use crate::configs::settings::{Latitude, Longitude};
use crate::errors::GeohashError;
use crate::logger;
use anyhow::Error;
use anyhow::Result;
use chrono::DateTime;
use chrono::TimeZone;
use resvg::tiny_skia;
use resvg::usvg;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use usvg::fontdb;

/// Converts an SVG file to a PNG file.
///
/// # Arguments
///
/// * `input_path` - Path to the input SVG file.
/// * `output_path` - Path to save the output PNG file.
/// * `scale_factor` - The scale factor to apply to the SVG.
///
/// # Returns
///
/// * `Result<(), Error>` - Ok(()) if successful, or an error message.
pub fn convert_svg_to_png(
    input_path: &PathBuf,
    output_path: &PathBuf,
    scale_factor: f32,
) -> Result<(), Error> {
    // Read the SVG file
    let svg_data = fs::read_to_string(input_path)
        .map_err(|e| Error::msg(format!("Failed to read SVG file: {e}")))?;

    // Uses the same cached database as `measure_stacked_label_dx`, so a
    // label's measured width always matches what actually gets rendered
    // here — if these ever used separately built databases, a system font
    // shadowing a bundled one (see `load_fonts`) could make one resolve a
    // different face than the other.
    let opts = usvg::Options {
        fontdb: shared_font_db(),
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(&svg_data, &opts)
        .map_err(|e| Error::msg(format!("Failed to parse SVG: {e}")))?;

    // Create a higher resolution canvas
    let pixmap_size = tree.size().to_int_size();
    let width = (pixmap_size.width() as f32 * scale_factor) as u32;
    let height = (pixmap_size.height() as f32 * scale_factor) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| Error::msg("Failed to create pixmap"))?;

    // Create a transform that scales the SVG
    let transform = tiny_skia::Transform::from_scale(scale_factor, scale_factor);

    // Render SVG onto the canvas with scaling
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Save the PNG file
    pixmap
        .save_png(output_path)
        .map_err(|e| Error::msg(format!("Failed to save PNG: {e}")))?;

    Ok(())
}

/// Loads fonts into the provided font database.
///
/// # Arguments
///
/// * `font_db` - A mutable reference to a `fontdb::Database` to load fonts into.
fn load_fonts(font_db: &mut fontdb::Database) {
    font_db.load_system_fonts();

    // print current path
    let current_path = std::env::current_dir().unwrap();

    let font_files = [
        "static/fonts/Roboto-VariableFont_wdth,wght.ttf",
        "static/fonts/Roboto-Italic-VariableFont_wdth,wght.ttf",
        "static/fonts/Roboto-Regular-Dashed.ttf",
        // Subset covering only the kanji used by the Japanese locale strings in
        // src/i18n.rs; Roboto has no CJK glyphs, so resvg falls back to this font.
        "static/fonts/NotoSansJP-Weather-Regular.ttf",
    ];

    for file in &font_files {
        match font_db.load_font_file(current_path.join(file)) {
            Ok(_) => {}
            Err(e) => logger::warning(format!("Failed to load font file: {e}")),
        }
    }
}

/// Lazily built, process-wide font database used for text measurement.
///
/// Loads the same fonts (and system-font fallback) as [`convert_svg_to_png`],
/// so widths measured with [`measure_text_width`] match what resvg actually
/// renders. Cached because `load_system_fonts` is a filesystem scan and
/// measurement can run once per label per render.
fn shared_font_db() -> Arc<fontdb::Database> {
    static FONT_DB: OnceLock<Arc<fontdb::Database>> = OnceLock::new();
    FONT_DB
        .get_or_init(|| {
            let mut font_db = fontdb::Database::new();
            load_fonts(&mut font_db);
            Arc::new(font_db)
        })
        .clone()
}

// =============================================================================
// TEMPORARY WORKAROUND — remove once resvg/usvg supports tspan `text-anchor`
// with `dx`/`dy` centering correctly: https://github.com/linebender/resvg/issues/583
//
// Until then, stacked-label centering (e.g. "Feels"/"Like") and the gap to
// whatever follows it are computed here by actually rendering and measuring
// pixels, rather than via native SVG anchoring.
// =============================================================================

fn escape_xml_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Computes the horizontal `dx` (SVG user units) that visually centers
/// `line2` directly beneath `line1`, when both are rendered as sibling
/// `tspan`s of one `<text font_family font_size>` element via relative
/// `dx`/`dy` (i.e. `<tspan>{line1}</tspan><tspan dx dy>{line2}</tspan>`,
/// the pattern used for the dashboard's stacked "Feels"/"Like"-style
/// labels).
///
/// Without an explicit `dx`, `line2` starts wherever the cursor was left
/// after `line1` — its own centre doesn't line up with `line1`'s. Rather
/// than a hand-tuned constant per language/font, this renders both lines
/// through the real resvg/usvg pipeline (same font DB as the final
/// dashboard, including glyph-fallback for scripts like Japanese) and
/// measures the actual ink centre of each line from the rasterized pixels,
/// so it stays correct for any wording, font, or font-size — no
/// recalibration needed when a language or font changes.
///
/// Renders the two lines far apart vertically (independent of whatever the
/// real, much smaller, line spacing will be) purely so their ink can't
/// overlap while measuring; only the horizontal centres are used. Returns
/// `0.0` if either line is empty or rendering fails.
pub fn measure_stacked_label_dx(
    line1: &str,
    line2: &str,
    font_family: &str,
    font_size: f32,
) -> f32 {
    if line1.is_empty() || line2.is_empty() {
        return 0.0;
    }

    let line1 = escape_xml_text(line1);
    let line2 = escape_xml_text(line2);
    // Mirrors the production markup's own x/text-anchor exactly (not just an
    // arbitrary centred canvas): text-anchor="middle" changes which pixel
    // column a glyph's antialiasing lands on (hinting is sensitive to
    // subpixel position), so matching it keeps this measurement's rounding
    // behaviour identical to what actually gets rendered.
    let cx = 246.0_f32;
    let width = 500.0_f32;
    let separation = font_size * 10.0;
    let height = separation + font_size * 4.0;
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" font-family="{font_family}">
            <text x="{cx}" y="{font_size}" text-anchor="middle" font-size="{font_size}">
                <tspan>{line1}</tspan>
                <tspan dx="0" dy="{separation}">{line2}</tspan>
            </text>
        </svg>"#
    );

    let opts = usvg::Options {
        fontdb: shared_font_db(),
        ..Default::default()
    };
    let tree = match usvg::Tree::from_str(&svg, &opts) {
        Ok(tree) => tree,
        Err(e) => {
            logger::warning(format!(
                "Failed to measure stacked label offset for {line1:?}/{line2:?}: {e}"
            ));
            return 0.0;
        }
    };

    let size = tree.size().to_int_size();
    let mut pixmap = match tiny_skia::Pixmap::new(size.width(), size.height()) {
        Some(pixmap) => pixmap,
        None => return 0.0,
    };
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );

    let ink_extent_x = |y_from: u32, y_to: u32| -> Option<(u32, u32)> {
        let mut min_x = u32::MAX;
        let mut max_x = 0;
        for y in y_from..=y_to.min(pixmap.height().saturating_sub(1)) {
            for x in 0..pixmap.width() {
                if pixmap.pixel(x, y).map(|p| p.alpha()).unwrap_or(0) > 0 {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                }
            }
        }
        (min_x <= max_x).then_some((min_x, max_x))
    };

    let split_y = (font_size + separation / 2.0) as u32;
    let (Some((r1_min, r1_max)), Some((r2_min, r2_max))) = (
        ink_extent_x(0, split_y),
        ink_extent_x(split_y + 1, pixmap.height() - 1),
    ) else {
        logger::warning(format!(
            "Failed to measure stacked label offset for {line1:?}/{line2:?}: no ink found"
        ));
        return 0.0;
    };

    let line1_centre = (r1_min + r1_max) as f32 / 2.0;
    let line2_centre = (r2_min + r2_max) as f32 / 2.0;
    line1_centre - line2_centre
}

fn render_svg_to_pixmap(svg: &str) -> Option<tiny_skia::Pixmap> {
    let opts = usvg::Options {
        fontdb: shared_font_db(),
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(svg, &opts).ok()?;
    let size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())?;
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    Some(pixmap)
}

/// x-extent (min, max) of near-opaque pixels close to `(r, g, b)`, or `None`
/// if no such pixel is found.
///
/// Only near-full alpha counts: at low alpha, `tiny_skia`'s premultiplied
/// channels shrink toward zero regardless of hue, so a partially
/// transparent antialiased edge of *any* colour would otherwise be
/// misclassified as matching every colour, including black.
fn ink_extent_x_of_colour(
    pixmap: &tiny_skia::Pixmap,
    (r, g, b): (u8, u8, u8),
) -> Option<(u32, u32)> {
    let close = |a: u8, b: u8| (a as i32 - b as i32).abs() < 40;
    let mut min_x = u32::MAX;
    let mut max_x = 0;
    for y in 0..pixmap.height() {
        for x in 0..pixmap.width() {
            if let Some(p) = pixmap.pixel(x, y) {
                if p.alpha() > 200 && close(p.red(), r) && close(p.green(), g) && close(p.blue(), b)
                {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                }
            }
        }
    }
    (min_x <= max_x).then_some((min_x, max_x))
}

/// Computes the `dx` (SVG user units) for the tspan that follows a
/// "Feels"/"Like"-style stacked label, so that tspan's ink starts exactly
/// `target_gap` after the label's own ink — regardless of how far right the
/// label's natural cursor position already landed (which is language- and
/// font-dependent, see [`measure_stacked_label_dx`]).
///
/// `label_dx`/`label_dy` must be the exact offsets already applied to
/// `line2` in production (from [`measure_stacked_label_dx`]), since they
/// change where the cursor sits by the time the following tspan starts.
/// `probe_text` stands in for whatever dynamic content the real tspan will
/// hold (e.g. a temperature reading) — only its left ink edge is used, so
/// any fixed placeholder of the same font/size works (a real value isn't
/// known at measurement time and doesn't materially change left-edge
/// position). Returns `0.0` if either label line is empty or rendering
/// fails.
#[allow(clippy::too_many_arguments)]
pub fn measure_label_to_number_gap_dx(
    line1: &str,
    line2: &str,
    label_dx: f32,
    label_dy: f32,
    label_font_family: &str,
    label_font_size: f32,
    number_font_family: &str,
    number_font_size: f32,
    probe_text: &str,
    target_gap: f32,
) -> f32 {
    if line1.is_empty() || line2.is_empty() {
        return 0.0;
    }

    let line1 = escape_xml_text(line1);
    let line2 = escape_xml_text(line2);
    let label_colour = (0, 0, 0);
    let probe_colour = (0, 255, 0);
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="500" height="{number_font_size}" font-family="{label_font_family}">
            <text x="246" y="{label_font_size}" text-anchor="middle" font-size="{label_font_size}" fill="rgb(0,0,0)">
                <tspan>{line1}</tspan>
                <tspan dx="{label_dx}" dy="{label_dy}">{line2}</tspan>
                <tspan font-family="{number_font_family}" dominant-baseline="middle" font-size="{number_font_size}" fill="rgb(0,255,0)" dx="0">{probe_text}</tspan>
            </text>
        </svg>"#
    );

    let Some(pixmap) = render_svg_to_pixmap(&svg) else {
        logger::warning(format!(
            "Failed to measure label-to-number gap for {line1:?}/{line2:?}: render failed"
        ));
        return 0.0;
    };
    let (Some((_, label_max)), Some((probe_min, _))) = (
        ink_extent_x_of_colour(&pixmap, label_colour),
        ink_extent_x_of_colour(&pixmap, probe_colour),
    ) else {
        logger::warning(format!(
            "Failed to measure label-to-number gap for {line1:?}/{line2:?}: no ink found"
        ));
        return 0.0;
    };

    let natural_gap = probe_min as f32 - label_max as f32;
    target_gap - natural_gap
}

// ============================= END WORKAROUND ==============================

/// Calculates the total value between two dates from a dataset.
///
/// # Arguments
///
/// * `data` - A slice of data items.
/// * `start_date` - The start date as `DateTime<TZ>`.
/// * `end_date` - The end date as `DateTime<TZ>`.
/// * `get_value` - A function to extract the value from a data item.
/// * `get_time` - A function to extract the time from a data item.
///
/// # Returns
///
/// * `V` - The total value between the specified dates.
pub fn total_between_dates<T, V, TZ: TimeZone>(
    data: &[T],
    start_date: &DateTime<TZ>,
    end_date: &DateTime<TZ>,
    get_value: impl Fn(&T) -> V,
    get_time: impl Fn(&T) -> DateTime<TZ>,
) -> V
where
    V: std::iter::Sum + Default,
{
    data.iter()
        .filter_map(|item| {
            let item_date = &get_time(item);
            if item_date >= start_date && item_date < end_date {
                Some(get_value(item))
            } else {
                None
            }
        })
        .sum()
}

/// Finds the maximum value between two dates from a dataset.
///
/// # Arguments
///
/// * `data` - A slice of data items.
/// * `start_date` - The start date as `DateTime<TZ>`.
/// * `end_date` - The end date as `DateTime<TZ>`, not inclusive.
/// * `get_value` - A function to extract the value from a data item.
/// * `get_time` - A function to extract the time from a data item.
///
/// # Returns
///
/// * `None` if no item falls in `[start_date, end_date)` — distinct from "the
///   maximum value present is zero" — so callers can render "no data"
///   instead of a misleading `0`.
pub fn find_max_item_between_dates<T, V, TZ: TimeZone>(
    data: &[T],
    start_date: &DateTime<TZ>,
    end_date: &DateTime<TZ>,
    get_value: impl Fn(&T) -> V,
    get_time: impl Fn(&T) -> DateTime<TZ>,
) -> Option<V>
where
    V: PartialOrd + Copy,
{
    data.iter()
        .filter_map(|item| {
            let date = &get_time(item);
            if date >= start_date && date < end_date {
                Some(get_value(item))
            } else {
                None
            }
        })
        .fold(None, |acc, x| match acc {
            Some(acc) if acc > x => Some(acc),
            _ => Some(x),
        })
}

// Below code was adopted from Geohash crate
// https://github.com/georust/geohash/blob/main/src/core.rs

// the alphabet for the base32 encoding used in geohashing
#[rustfmt::skip]
const BASE32_CODES: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

// bit shifting functions used in encoding and decoding

// spread takes a u32 and deposits its bits into the evenbit positions of a u64
#[inline]
fn spread(x: u32) -> u64 {
    let mut new_x = x as u64;
    new_x = (new_x | (new_x << 16)) & 0x0000ffff0000ffff;
    new_x = (new_x | (new_x << 8)) & 0x00ff00ff00ff00ff;
    new_x = (new_x | (new_x << 4)) & 0x0f0f0f0f0f0f0f0f;
    new_x = (new_x | (new_x << 2)) & 0x3333333333333333;
    new_x = (new_x | (new_x << 1)) & 0x5555555555555555;

    new_x
}

// spreads the inputs, then shifts the y input and does a bitwise or to fill the remaining bits in x
#[inline]
fn interleave(x: u32, y: u32) -> u64 {
    spread(x) | (spread(y) << 1)
}

/// Encode a validated coordinate to a geohash with length `len`.
///
/// # Arguments
///
/// * `lon_x` - The longitude, guaranteed in [-180, 180] by the `Longitude` type
/// * `lat_y` - The latitude, guaranteed in [-90, 90] by the `Latitude` type
/// * `len` - The desired length of the geohash string (1-12)
pub fn encode(lon_x: Longitude, lat_y: Latitude, len: usize) -> Result<String, GeohashError> {
    if !(1..=12).contains(&len) {
        return Err(GeohashError::InvalidLength(len));
    }

    let lon_x = lon_x.into_inner();
    let lat_y = lat_y.into_inner();

    // divides the latitude by 180, then adds 1.5 to give a value between 1 and 2
    // then we take the first 32 bits of the significand as a u32
    let lat32 = ((lat_y * 0.005555555555555556 + 1.5).to_bits() >> 20) as u32;
    // same as latitude, but a division by 360 instead of 180
    let lon32 = ((lon_x * 0.002777777777777778 + 1.5).to_bits() >> 20) as u32;

    let mut interleaved_int = interleave(lat32, lon32);

    let mut out = String::with_capacity(len);
    // loop through and take the first 5 bits of the interleaved value ech iteration
    for _ in 0..len {
        // shifts so that the high 5 bits are now the low five bits, then masks to get their value
        let code = (interleaved_int >> 59) as usize & (0x1f);
        // uses that value to index into the array of base32 codes
        out.push(BASE32_CODES[code]);
        // shifts the interleaved bits left by 5, so we get the next 5 bits on the next iteration
        interleaved_int <<= 5;
    }
    Ok(out)
}

// Finish Geohash crate code

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Renders the full production-shaped construct (stacked label + a
    /// following coloured tspan) with the `dx`s [`measure_stacked_label_dx`]
    /// and [`measure_label_to_number_gap_dx`] compute, then verifies the
    /// tspan's ink actually lands `target_gap` after the label's ink, across
    /// several scripts/word-length combinations.
    fn assert_label_to_number_gap(line1: &str, line2: &str, target_gap: f32) {
        let label_font_size = 18.0;
        let number_font_size = 55.0;
        let label_dx =
            measure_stacked_label_dx(line1, line2, "Roboto, sans-serif", label_font_size);
        let label_dy = 15.5;
        let gap_dx = measure_label_to_number_gap_dx(
            line1,
            line2,
            label_dx,
            label_dy,
            "Roboto, sans-serif",
            label_font_size,
            "Roboto-Regular-Dashed",
            number_font_size,
            "16",
            target_gap,
        );
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="500" height="250" font-family="Roboto, sans-serif">
                <text x="246" y="158" text-anchor="middle" font-size="{label_font_size}" fill="rgb(0,0,0)">
                    <tspan>{line1}</tspan>
                    <tspan dx="{label_dx}" dy="{label_dy}">{line2}</tspan>
                    <tspan font-family="Roboto-Regular-Dashed" dominant-baseline="middle" font-size="{number_font_size}" fill="rgb(0,255,0)" dx="{gap_dx}">16</tspan>
                </text>
            </svg>"#
        );
        let pixmap = render_svg_to_pixmap(&svg).unwrap();
        let (_, label_max) = ink_extent_x_of_colour(&pixmap, (0, 0, 0)).unwrap();
        let (probe_min, _) = ink_extent_x_of_colour(&pixmap, (0, 255, 0)).unwrap();
        let gap = probe_min as f32 - label_max as f32;
        assert!(
            (gap - target_gap).abs() <= 1.0,
            "{line1:?}/{line2:?}: gap not at target, got {gap} want {target_gap}"
        );
    }

    #[test]
    fn label_to_number_gap_matches_target_for_every_locale_pair() {
        for (line1, line2) in [
            ("Feels", "Like"),
            ("Ress.", "comme"),
            ("Gef.", "wie"),
            ("Se", "siente"),
            ("体感", "温度"),
        ] {
            assert_label_to_number_gap(line1, line2, 12.0);
        }
    }

    /// Renders `line1`/`line2` stacked with the `dx` computed by
    /// [`measure_stacked_label_dx`] and verifies the two lines actually end
    /// up ink-centered on one another (within half a pixel), across several
    /// scripts/word-length combinations — i.e. that the measurement is
    /// self-consistent with what it's meant to produce, not just idempotent.
    fn assert_stacked_label_dx_centers(line1: &str, line2: &str) {
        let font_size: f32 = 18.0;
        let dx = measure_stacked_label_dx(line1, line2, "Roboto, sans-serif", font_size);
        let dy: f32 = 15.5;
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="500" height="250" font-family="Roboto, sans-serif">
                <text x="246" y="158" text-anchor="middle" font-size="{font_size}">
                    <tspan>{line1}</tspan>
                    <tspan dx="{dx}" dy="{dy}">{line2}</tspan>
                </text>
            </svg>"#
        );
        let opts = usvg::Options {
            fontdb: shared_font_db(),
            ..Default::default()
        };
        let tree = usvg::Tree::from_str(&svg, &opts).unwrap();
        let size = tree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
        resvg::render(
            &tree,
            tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );

        let ink_extent_x = |y_from: u32, y_to: u32| -> (u32, u32) {
            let mut min_x = u32::MAX;
            let mut max_x = 0;
            for y in y_from..=y_to {
                for x in 0..pixmap.width() {
                    if pixmap.pixel(x, y).map(|p| p.alpha()).unwrap_or(0) > 0 {
                        min_x = min_x.min(x);
                        max_x = max_x.max(x);
                    }
                }
            }
            assert!(min_x <= max_x, "no ink found in y range {y_from}..={y_to}");
            (min_x, max_x)
        };
        let split_y = (158.0 + dy / 2.0).round() as u32;
        let (r1_min, r1_max) = ink_extent_x(0, split_y);
        let (r2_min, r2_max) = ink_extent_x(split_y + 1, pixmap.height() - 1);
        let c1 = (r1_min + r1_max) as f32 / 2.0;
        let c2 = (r2_min + r2_max) as f32 / 2.0;
        // A couple of pixels of slop is expected: the two renders place the
        // glyphs at different absolute x positions, and font hinting can
        // shift ink by a pixel depending on where a glyph lands on the pixel
        // grid. What matters is this is imperceptible at label size, far
        // smaller than the tens-of-pixels a wrong-by-a-half-width formula
        // would produce.
        assert!(
            (c1 - c2).abs() <= 0.5,
            "{line1:?}/{line2:?}: line centres not aligned, dx={dx} c1={c1} c2={c2}"
        );
    }

    #[test]
    fn stacked_label_dx_centers_every_locale_pair() {
        for (line1, line2) in [
            ("Feels", "Like"),
            ("Ress.", "comme"),
            ("Gef.", "wie"),
            ("Se", "siente"),
            ("体感", "温度"),
        ] {
            assert_stacked_label_dx_centers(line1, line2);
        }
    }

    #[test]
    fn stacked_label_dx_is_zero_for_empty_line() {
        assert_eq!(
            measure_stacked_label_dx("", "Like", "Roboto, sans-serif", 18.0),
            0.0
        );
        assert_eq!(
            measure_stacked_label_dx("Feels", "", "Roboto, sans-serif", 18.0),
            0.0
        );
    }

    struct Point {
        time: DateTime<Utc>,
        value: f64,
    }

    fn points(pairs: &[(&str, f64)]) -> Vec<Point> {
        pairs
            .iter()
            .map(|(t, v)| Point {
                time: t.parse().unwrap(),
                value: *v,
            })
            .collect()
    }

    mod total_between_dates_tests {
        use super::*;

        #[test]
        fn empty_data_sums_to_zero() {
            let data: Vec<Point> = Vec::new();
            let start = "2024-01-01T00:00:00Z".parse().unwrap();
            let end = "2024-01-02T00:00:00Z".parse().unwrap();
            let total = total_between_dates(&data, &start, &end, |p| p.value, |p| p.time);
            assert_eq!(total, 0.0);
        }

        #[test]
        fn single_item_in_range_is_included() {
            let data = points(&[("2024-01-01T12:00:00Z", 5.0)]);
            let start = "2024-01-01T00:00:00Z".parse().unwrap();
            let end = "2024-01-02T00:00:00Z".parse().unwrap();
            let total = total_between_dates(&data, &start, &end, |p| p.value, |p| p.time);
            assert_eq!(total, 5.0);
        }

        #[test]
        fn sums_multiple_items_in_range() {
            let data = points(&[
                ("2024-01-01T01:00:00Z", 1.0),
                ("2024-01-01T02:00:00Z", 2.0),
                ("2024-01-01T03:00:00Z", 3.0),
            ]);
            let start = "2024-01-01T00:00:00Z".parse().unwrap();
            let end = "2024-01-02T00:00:00Z".parse().unwrap();
            let total = total_between_dates(&data, &start, &end, |p| p.value, |p| p.time);
            assert_eq!(total, 6.0);
        }

        #[test]
        fn start_date_is_inclusive_end_date_is_exclusive() {
            let data = points(&[
                ("2024-01-01T00:00:00Z", 10.0),  // == start, included
                ("2024-01-02T00:00:00Z", 100.0), // == end, excluded
            ]);
            let start = "2024-01-01T00:00:00Z".parse().unwrap();
            let end = "2024-01-02T00:00:00Z".parse().unwrap();
            let total = total_between_dates(&data, &start, &end, |p| p.value, |p| p.time);
            assert_eq!(total, 10.0);
        }
    }

    mod find_max_item_between_dates_tests {
        use super::*;

        #[test]
        fn empty_data_returns_none() {
            let data: Vec<Point> = Vec::new();
            let start = "2024-01-01T00:00:00Z".parse().unwrap();
            let end = "2024-01-02T00:00:00Z".parse().unwrap();
            let max = find_max_item_between_dates(&data, &start, &end, |p| p.value, |p| p.time);
            // `None`, not `0.0` — "no data in range" is distinct from "the max
            // value present happens to be zero" (see src/utils.rs's doc comment).
            assert_eq!(max, None);
        }

        #[test]
        fn single_item_in_range_is_the_max() {
            let data = points(&[("2024-01-01T12:00:00Z", 5.0)]);
            let start = "2024-01-01T00:00:00Z".parse().unwrap();
            let end = "2024-01-02T00:00:00Z".parse().unwrap();
            let max = find_max_item_between_dates(&data, &start, &end, |p| p.value, |p| p.time);
            assert_eq!(max, Some(5.0));
        }

        #[test]
        fn finds_max_among_multiple_items() {
            let data = points(&[
                ("2024-01-01T01:00:00Z", 3.0),
                ("2024-01-01T02:00:00Z", 9.0),
                ("2024-01-01T03:00:00Z", 4.0),
            ]);
            let start = "2024-01-01T00:00:00Z".parse().unwrap();
            let end = "2024-01-02T00:00:00Z".parse().unwrap();
            let max = find_max_item_between_dates(&data, &start, &end, |p| p.value, |p| p.time);
            assert_eq!(max, Some(9.0));
        }

        #[test]
        fn end_date_is_exclusive() {
            let data = points(&[("2024-01-02T00:00:00Z", 100.0)]);
            let start = "2024-01-01T00:00:00Z".parse().unwrap();
            let end = "2024-01-02T00:00:00Z".parse().unwrap();
            let max = find_max_item_between_dates(&data, &start, &end, |p| p.value, |p| p.time);
            assert_eq!(max, None); // out of range -> no data, not a fallback zero
        }

        #[test]
        fn all_negative_values_report_the_actual_max_not_zero() {
            // Regression: the previous `V::default()`-seeded fold reported 0
            // as the max whenever every real value was negative, since
            // nothing ever exceeded the zero seed. Not reachable through
            // today's callers (wind speed / UV / humidity are all
            // non-negative), but the function is generic — this pins the
            // fold itself, independent of what callers happen to pass.
            let data = points(&[
                ("2024-01-01T01:00:00Z", -10.0),
                ("2024-01-01T02:00:00Z", -3.0),
                ("2024-01-01T03:00:00Z", -7.0),
            ]);
            let start = "2024-01-01T00:00:00Z".parse().unwrap();
            let end = "2024-01-02T00:00:00Z".parse().unwrap();
            let max = find_max_item_between_dates(&data, &start, &end, |p| p.value, |p| p.time);
            assert_eq!(max, Some(-3.0));
        }
    }

    mod spread_and_interleave {
        use super::*;

        #[test]
        fn spread_zero_is_zero() {
            assert_eq!(spread(0), 0);
        }

        #[test]
        fn spread_deposits_bits_into_even_positions() {
            // 0b11 -> bits land at positions 0 and 2 (0b101 = 5)
            assert_eq!(spread(0b11), 0b101);
        }

        #[test]
        fn interleave_combines_x_into_even_and_y_into_odd_bits() {
            // x=0b1 (bit 0) -> even position 0; y=0b1 (bit 0) -> odd position 1
            assert_eq!(interleave(0b1, 0b1), 0b11);
        }

        #[test]
        fn interleave_zero_and_zero_is_zero() {
            assert_eq!(interleave(0, 0), 0);
        }
    }

    mod encode_tests {
        use super::*;
        use proptest::prelude::*;

        #[test]
        fn known_coordinate_produces_expected_geohash() {
            // Sydney Opera House, well-known reference coordinate.
            let lat = Latitude::try_new(-33.8568).unwrap();
            let lon = Longitude::try_new(151.2153).unwrap();
            let hash = encode(lon, lat, 6).unwrap();
            assert_eq!(hash, "r3gx2u");
        }

        #[test]
        fn zero_zero_produces_expected_geohash() {
            let lat = Latitude::try_new(0.0).unwrap();
            let lon = Longitude::try_new(0.0).unwrap();
            let hash = encode(lon, lat, 5).unwrap();
            assert_eq!(hash, "s0000");
        }

        #[test]
        fn length_zero_is_invalid() {
            let lat = Latitude::try_new(0.0).unwrap();
            let lon = Longitude::try_new(0.0).unwrap();
            assert!(encode(lon, lat, 0).is_err());
        }

        #[test]
        fn length_above_twelve_is_invalid() {
            let lat = Latitude::try_new(0.0).unwrap();
            let lon = Longitude::try_new(0.0).unwrap();
            assert!(encode(lon, lat, 13).is_err());
        }

        #[test]
        fn output_length_matches_requested_length() {
            let lat = Latitude::try_new(45.0).unwrap();
            let lon = Longitude::try_new(-122.0).unwrap();
            let hash = encode(lon, lat, 9).unwrap();
            assert_eq!(hash.len(), 9);
        }

        proptest! {
            #[test]
            fn output_length_always_equals_requested_length(
                lon in -180.0f64..=180.0,
                lat in -90.0f64..=90.0,
                len in 1usize..=12,
            ) {
                let lon = Longitude::try_new(lon).unwrap();
                let lat = Latitude::try_new(lat).unwrap();
                let hash = encode(lon, lat, len).unwrap();
                prop_assert_eq!(hash.len(), len);
            }
        }
    }
}

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

    let mut font_db = fontdb::Database::new();
    load_fonts(&mut font_db);

    // Parse the SVG
    let opts = usvg::Options {
        fontdb: font_db.into(),
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
    ];

    for file in &font_files {
        match font_db.load_font_file(current_path.join(file)) {
            Ok(_) => {}
            Err(e) => logger::warning(format!("Failed to load font file: {e}")),
        }
    }
}

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

use crate::{
    clock::Clock, constants::DEFAULT_AXIS_LABEL_FONT_SIZE, logger, weather::icons::UVIndexIcon,
};
use anyhow::Error;
use std::fmt;
use strum_macros::Display;

#[derive(Clone, Debug, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn to_svg(self) -> String {
        format!("L {} {}", self.x, self.y)
    }
}

#[derive(Clone, Debug)]
pub struct Curve {
    pub c1: Point,
    pub c2: Point,
    pub end: Point,
}

impl Curve {
    pub fn to_svg(&self) -> String {
        format!(
            "C {:.4} {:.4}, {:.4} {:.4}, {:.4} {:.4}",
            self.c1.x, self.c1.y, self.c2.x, self.c2.y, self.end.x, self.end.y
        )
    }
}

#[derive(Clone, Debug)]
pub enum PrecipitationPattern {
    Snow,
    Rain,
}

impl fmt::Display for PrecipitationPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrecipitationPattern::Snow => write!(f, "snow"),
            PrecipitationPattern::Rain => write!(f, "rain"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrecipitationBlock {
    pub path: String,
    pub pattern: PrecipitationPattern,
    pub x_start: f32,
    pub x_end: f32,
    pub height_left: f32,
    pub height_right: f32,
    pub max_height: f32,
    pub chance: f32,
}

#[derive(Clone, Debug)]
pub struct GraphData {
    pub points: Vec<Point>,
    pub smooth: bool,
}

#[derive(Clone, Debug, Copy)]
pub struct PrecipitationPoint {
    pub x: f32,
    pub chance: f32,
    pub is_primarily_snow: bool,
}

#[derive(Clone, Debug)]
pub struct PrecipitationData {
    pub points: Vec<PrecipitationPoint>,
}

#[derive(Clone, Debug)]
pub enum CurveType {
    ActualTemp(GraphData),
    TempFeelLike(GraphData),
    PrecipitationChance(PrecipitationData),
}

impl GraphData {
    pub fn add_point(&mut self, x: f32, y: f32) {
        self.points.push(Point { x, y })
    }
}

impl PrecipitationData {
    pub fn add_point(&mut self, x: f32, chance: f32, is_primarily_snow: bool) {
        self.points.push(PrecipitationPoint {
            x,
            chance,
            is_primarily_snow,
        })
    }
}
pub struct HourlyForecastGraph {
    pub curves: Vec<CurveType>,
    pub uv_data: [u16; 24],
    pub height: f32,
    pub width: f32,
    pub starting_x: f32,
    pub ending_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub x_ticks: u16,
    pub y_left_ticks: u16,
    pub y_right_ticks: u16,
    pub x_axis_always_at_min: bool,
    pub text_colour: String,
    /// Display timezone for time-dependent labels (e.g. the "tomorrow" day name).
    pub tz: chrono_tz::Tz,
}

// TODO: use the builder pattern to create the graph
impl Default for HourlyForecastGraph {
    fn default() -> Self {
        Self {
            curves: vec![
                CurveType::ActualTemp(GraphData {
                    points: vec![],
                    smooth: true,
                }),
                CurveType::TempFeelLike(GraphData {
                    points: vec![],
                    smooth: true,
                }),
                CurveType::PrecipitationChance(PrecipitationData { points: vec![] }),
            ],
            uv_data: [0; 24],
            height: 300.0,
            width: 600.0,
            starting_x: 0.0,
            ending_x: 23.0,
            min_y: f32::INFINITY,
            max_y: -f32::INFINITY,
            // Number of ticks, +1 because of the fencepost problem
            x_ticks: 6,
            y_left_ticks: 5,
            y_right_ticks: 5,
            x_axis_always_at_min: false,
            text_colour: "black".to_string(),
            tz: chrono_tz::UTC,
        }
    }
}

pub enum GraphDataPath {
    Temp(String),
    TempFeelLike(String),
    Precipitation(Vec<PrecipitationBlock>),
}

#[derive(Debug, Display)]
pub enum FontStyle {
    #[strum(to_string = "normal")]
    Normal,
    #[strum(to_string = "italic")]
    Italic,
}

#[derive(Debug, Display)]
pub enum ElementVisibility {
    #[strum(to_string = "visible")]
    Visible,
    #[strum(to_string = "hidden")]
    Hidden,
}

fn lcg_next(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

/// Builds one unified SVG fragment for all precipitation blocks (rain and snow mixed):
/// - one `<clipPath>` covering every block regardless of type
/// - one `<linearGradient>` whose stop-colour and stop-opacity vary per block
///   (rain_colour/snow_colour, opacity scaled between opacity_min and opacity_max by chance)
/// - a single LCG placement pass with a shared `placed` list so the seed never
///   resets at block boundaries; glyph type is chosen per block from `pattern`
///
/// Separation uses the larger snowflake threshold whenever either the new or an
/// existing glyph is a snowflake, keeping glyphs of different types from colliding.
pub(crate) fn generate_unified_precipitation_svg(
    blocks: &[PrecipitationBlock],
    rain_colour: &str,
    snow_colour: &str,
    graph_height: f32,
    opacity_min: f32,
    opacity_max: f32,
) -> String {
    let x_start = blocks.first().map(|b| b.x_start).unwrap_or(0.0);
    let x_end = blocks.last().map(|b| b.x_end).unwrap_or(0.0);
    let x_range = (x_end - x_start).max(1.0);

    // --- clip path: union of all blocks ---
    let clip_paths: String = blocks
        .iter()
        .map(|b| format!(r#"<path d="{}"/>"#, b.path))
        .collect();

    // --- gradient: one stop per hour, colour and opacity vary by type ---
    let mut gradient_stops = String::new();
    for block in blocks.iter() {
        let offset = (block.x_start - x_start) / x_range * 100.0;
        let colour = match &block.pattern {
            PrecipitationPattern::Snow => snow_colour,
            PrecipitationPattern::Rain => rain_colour,
        };
        let stop_opacity =
            (opacity_min + (block.chance / 100.0) * (opacity_max - opacity_min)).clamp(0.0, 1.0);
        gradient_stops.push_str(&format!(
            r#"<stop offset="{offset:.2}%" stop-color="{colour}" stop-opacity="{stop_opacity:.3}"/>"#
        ));
    }
    if let Some(last) = blocks.last() {
        let colour = match &last.pattern {
            PrecipitationPattern::Snow => snow_colour,
            PrecipitationPattern::Rain => rain_colour,
        };
        let stop_opacity =
            (opacity_min + (last.chance / 100.0) * (opacity_max - opacity_min)).clamp(0.0, 1.0);
        gradient_stops.push_str(&format!(
            r#"<stop offset="100%" stop-color="{colour}" stop-opacity="{stop_opacity:.3}"/>"#
        ));
    }

    // --- single placement pass across all blocks ---
    // placed: (x, y, is_snow) — type tracked so we can use the right separation threshold.
    let mut seed: u64 = 2654435761;
    let mut placed: Vec<(f32, f32, bool)> = Vec::new();
    let mut glyphs = String::new();

    for (idx, block) in blocks.iter().enumerate() {
        if block.chance == 0.0 || block.max_height == 0.0 {
            continue;
        }
        let is_snow = match &block.pattern {
            PrecipitationPattern::Snow => true,
            PrecipitationPattern::Rain => false,
        };

        let width = block.x_end - block.x_start;

        let (density_div, max_count) = if is_snow { (150.0, 25) } else { (80.0, 30) };
        let count =
            ((width * block.max_height / density_div * block.chance / 100.0) as u32).min(max_count);

        let r_x: f32 = 4.0;
        let r_y: f32 = if is_snow { 4.0 } else { 9.0 };

        // Only pad x where there is no adjacent non-zero block to extend the clip region.
        let left_open =
            idx == 0 || blocks[idx - 1].chance == 0.0 || blocks[idx - 1].max_height == 0.0;
        let right_open = idx + 1 >= blocks.len()
            || blocks[idx + 1].chance == 0.0
            || blocks[idx + 1].max_height == 0.0;
        let x_lo = block.x_start + if left_open { r_x } else { 0.0 };
        let x_hi = (block.x_end - if right_open { r_x } else { 0.0 }).max(x_lo);
        let y_lo = r_y;
        // y_hi is loose; the slope test below rejects candidates above the sloped edge.
        let y_hi = block.max_height;
        let block_width = (block.x_end - block.x_start).max(1.0);

        'outer: for _ in 0..count {
            for _ in 0..20 {
                seed = lcg_next(seed);
                let rx = x_lo + (seed as f32 / u64::MAX as f32) * (x_hi - x_lo);
                seed = lcg_next(seed);
                let ry = y_lo + (seed as f32 / u64::MAX as f32) * (y_hi - y_lo);

                // Reject if glyph top would exceed the trapezoid's sloped ceiling at this x.
                let t = (rx - block.x_start) / block_width;
                let local_ceil = block.height_left + (block.height_right - block.height_left) * t;
                if ry + r_y > local_ceil {
                    seed = lcg_next(seed); // consume extra entropy to avoid degenerate retry patterns
                    continue;
                }

                // Use the larger snowflake separation whenever either glyph is a snowflake.
                let overlaps = placed.iter().any(|&(px, py, placed_snow)| {
                    let (sx, sy) = if is_snow || placed_snow {
                        (8.0_f32, 8.0_f32)
                    } else {
                        (7.0_f32, 14.0_f32)
                    };
                    (rx - px).abs() < sx && (ry - py).abs() < sy
                });

                if !overlaps {
                    placed.push((rx, ry, is_snow));
                    glyphs.push_str(&if is_snow {
                        seed = lcg_next(seed);
                        let radius = 2.0 + (seed as f32 / u64::MAX as f32) * 1.5;
                        format!(
                            r#"<circle cx="{rx:.2}" cy="{ry:.2}" r="{radius:.1}" fill="white" fill-opacity="0.85"/>"#
                        )
                    } else {
                        format!(
                            r#"<path d="M-1,1 C-1,0.45 -0.55,0 0,0 C0.55,0 1,0.45 1,1 L1,8 C1,8.55 0.55,9 0,9 C-0.55,9 -1,8.55 -1,8 Z" fill="white" fill-opacity="0.8" transform="translate({rx:.2},{ry:.2}) rotate(-15)"/>"#
                        )
                    });
                    continue 'outer;
                }
            }
        }
    }

    format!(
        r#"<defs>
            <linearGradient id="precipBg" gradientUnits="userSpaceOnUse" x1="{x_start}" y1="0" x2="{x_end}" y2="0">
                {gradient_stops}
            </linearGradient>
            <clipPath id="precipClip">
                {clip_paths}
            </clipPath>
        </defs>
        <g clip-path="url(#precipClip)">
            <rect x="{x_start}" y="0" width="{x_range}" height="{graph_height}" fill="url(#precipBg)"/>
            {glyphs}
        </g>"#
    )
}

/// Convert a list of points to a list of Bézier curves
/// using the Catmull-Rom to Bézier conversion
///
/// # Arguments
///
/// * `points` - A list of points to convert to Bézier curves
///
/// # Returns
///
/// A list of Bézier curves
pub fn catmull_rom_to_bezier(points: Vec<Point>) -> Vec<Curve> {
    if points.len() < 2 {
        return Vec::new();
    }

    let mut curves = Vec::with_capacity(points.len() - 1);

    let last_point = points.len() - 1;
    const TENSION: f32 = 6.0; // Catmull_Rom_standard, adjust to make curves more or less smooth

    for i in 0..last_point {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };

        let p1 = points[i];

        let p2 = points[i + 1];

        let p3 = if i + 2 > last_point {
            points[i + 1]
        } else {
            points[i + 2]
        };

        // Calculate control points using Catmull-Rom to Bézier conversion
        let c1 = Point {
            x: if i == 0 {
                // this is needed to make the first curve start at x=0
                // makes the curve tangent to the x axis for the first point
                0.0
            } else {
                (-p0.x + TENSION * p1.x + p2.x) / TENSION
            },
            y: (-p0.y + TENSION * p1.y + p2.y) / TENSION,
        };

        let c2 = Point {
            x: ((p1.x + TENSION * p2.x - p3.x) / TENSION),
            y: ((p1.y + TENSION * p2.y - p3.y) / TENSION),
        };

        let end = p2;
        curves.push(Curve { c1, c2, end });
    }

    curves
}

/// Collect all axis paths and labels into one struct
pub struct AxisPaths {
    pub x_axis_path: String,
    pub x_axis_guideline_path: String,
    pub y_left_axis_path: String,
    pub y_right_axis_path: String,
    pub x_labels: String,
    pub y_left_labels: String,
    pub y_right_labels: String,
}

/// Create the axis paths and labels for the graph
impl HourlyForecastGraph {
    pub fn create_axis_with_labels(&self, current_hour: f32, clock: &dyn Clock) -> AxisPaths {
        let range_x = self.ending_x - self.starting_x + 1.0; // +1 because last hour is 23
        let range_y_left = self.max_y - self.min_y;
        let range_y_right = 100.0; // Rain data is in percentage

        // Mapping functions from data space to SVG space
        // x data domain maps to [0, width]
        // y data domain maps to [height, 0] (SVG y goes down)
        let map_x = |x: f32| (x - self.starting_x) * (self.width / range_x);
        let map_y_left = |y: f32| self.height - ((y - self.min_y) * (self.height / range_y_left));
        // For the right axis, we assume 0 to 100% maps directly onto the height.
        let map_y_right = |y: f32| self.height - (y * (self.height / range_y_right));

        // Determine where to place the x-axis (shared between both left and right data)
        // If 0 is within the y range, place x-axis at y=0.
        // Otherwise, place it at the min or max y boundary.
        let x_axis_y = if self.x_axis_always_at_min || self.min_y > 0.0 && self.max_y > 0.00 {
            map_y_left(self.min_y) // min and max are both positive, so place it at min
        } else if self.min_y <= 0.0 && self.max_y >= 0.0 {
            map_y_left(0.0) // place x axis in between min and max
        } else {
            map_y_left(self.max_y) // min and max are both negative, so place it at max
        };

        // Determine where to place the y-axis
        // If 0 is within the x range, place y-axis at x=0.
        // Otherwise, place it at the starting_x or ending_x boundary.
        let y_axis_x = if self.starting_x <= 0.0 && self.ending_x >= 0.0 {
            map_x(0.0)
        } else if self.starting_x > 0.0 {
            map_x(self.starting_x)
        } else {
            map_x(self.ending_x)
        };

        // Right axis will be placed at the right side of the chart
        let y_right_axis_x = self.width;

        // Axis paths
        let mut x_axis_path = format!("M 0 {} L {} {}", x_axis_y, self.width, x_axis_y);
        let mut x_axis_guideline_path = format!("M 0 {} L {} {}", x_axis_y, self.width, x_axis_y);
        let mut y_left_axis_path = format!("M {} 0 L {} {}", y_axis_x, y_axis_x, self.height);
        let mut y_right_axis_path = format!(
            "M {} 0 L {} {}",
            y_right_axis_x, y_right_axis_x, self.height
        );

        let x_step = range_x / self.x_ticks as f32;
        let y_left_step = range_y_left / self.y_left_ticks as f32;
        let y_right_step = range_y_right / self.y_right_ticks as f32;

        // println!(
        //     "X step: {}, Y step (left): {}, Y step (right): {}",
        //     x_step, y_left_step, y_right_step
        // );

        // X-axis ticks and labels
        let x_labels = self.generate_x_axis_labels(
            current_hour,
            map_x,
            x_axis_y,
            &mut x_axis_path,
            &mut x_axis_guideline_path,
            x_step,
            clock,
        );

        // Y-axis ticks and labels (left)
        let y_left_labels =
            self.generate_y_axis_ticks(map_y_left, y_axis_x, &mut y_left_axis_path, y_left_step);

        // Y-axis ticks and labels (right - 0 to 100%)
        let y_right_labels = self.generate_right_axis_ticks(
            map_y_right,
            y_right_axis_x,
            &mut y_right_axis_path,
            y_right_step,
        );

        AxisPaths {
            x_axis_path,
            x_axis_guideline_path,
            y_left_axis_path,
            x_labels,
            y_left_labels,
            y_right_axis_path,
            y_right_labels,
        }
    }

    fn generate_right_axis_ticks(
        &self,
        map_y_right: impl Fn(f32) -> f32,
        y_right_axis_x: f32,
        y_right_axis_path: &mut String,
        y_right_step: f32,
    ) -> String {
        let mut y_right_labels = String::new();
        for k in 0..=self.y_right_ticks {
            let y_val = k as f32 * y_right_step; // percentage step
            if y_val > 100.0 {
                break;
            }
            let ys = map_y_right(y_val);
            // Tick mark on the right axis
            y_right_axis_path.push_str(&format!(
                " M {} {} L {} {}",
                y_right_axis_x - 5.0,
                ys,
                y_right_axis_x + 5.0,
                ys
            ));

            // Label (align to the start since it's on the right side)
            let label_x = y_right_axis_x + 10.0;
            let label_str = format!("{y_val:.0}%");
            y_right_labels.push_str(&format!(
                r#"<text x="{x}" y="{y}" fill="{colour}"  font-size="{DEFAULT_AXIS_LABEL_FONT_SIZE}" text-anchor="start" dy="4">{text}</text>"#,
                x = label_x,
                y = ys,
                colour = self.text_colour,
                text = label_str,
            ));
        }
        y_right_labels
    }

    fn generate_y_axis_ticks(
        &self,
        map_y_left: impl Fn(f32) -> f32,
        y_axis_x: f32,
        y_left_axis_path: &mut String,
        y_left_step: f32,
    ) -> String {
        let mut y_left_labels = String::new();
        for j in 0..=self.y_left_ticks {
            let y_val = self.min_y + j as f32 * y_left_step;
            // Use small tolerance to handle floating point precision issues
            // const EPSILON: f32 = 0.001;
            // this is just defensive - should not happen due to loop condition
            // if y_val > self.max_y + EPSILON {
            //     break;
            // }
            let ys = map_y_left(y_val);
            // Tick mark
            y_left_axis_path.push_str(&format!(
                " M {} {} L {} {}",
                y_axis_x - 5.0,
                ys,
                y_axis_x + 5.0,
                ys
            ));

            // Label: placed to the left of the y-axis
            let label_x = y_axis_x - 10.0;
            let mut label_str = format!("{y_val:.1}°");
            let mut font_size = DEFAULT_AXIS_LABEL_FONT_SIZE;
            if j == 0 || j == self.y_left_ticks {
                // Normalize negative zero when rounding to integer (e.g., -0.1 → 0, not -0)
                let display_val = if y_val.abs() < 0.5 { 0.0 } else { y_val };
                label_str = format!("{display_val:.0}°");
                font_size = 35;
            }
            y_left_labels.push_str(&format!(
                r#"<text x="{x}" y="{y}"  fill="{colour}" font-size="{font_size}" text-anchor="end" dx="8" dy="4">{text}</text>"#,
                x = label_x,
                y = ys,
                colour = self.text_colour,
                font_size = font_size,
                text = label_str
            ));
        }
        y_left_labels
    }

    #[allow(clippy::too_many_arguments)]
    fn generate_x_axis_labels(
        &self,
        current_hour: f32,
        map_x: impl Fn(f32) -> f32,
        x_axis_y: f32,
        x_axis_path: &mut String,
        x_axis_guideline_path: &mut String,
        x_step: f32,
        clock: &dyn Clock,
    ) -> String {
        let mut x_val: f32 = 0.0;
        let mut x_labels = String::new();
        for i in 0..=self.x_ticks {
            if x_val > self.ending_x {
                break;
            }
            x_val = self.starting_x + i as f32 * x_step;

            let xs = map_x(x_val);
            // Tick mark
            x_axis_path.push_str(&format!(
                " M {} {} L {} {}",
                xs,
                x_axis_y - 5.0,
                xs,
                x_axis_y + 5.0
            ));

            let x_guideline_len = self.height;
            // do not draw guideline if it overlaps with tomorrow's line
            if x_val != (24.0 - current_hour) {
                x_axis_guideline_path.push_str(&format!(
                    r#" M {xs} {x_guideline_len} v -{x_guideline_len} m 0 2 v -2"#
                ));
            }
            // Label: placed below the x-axis line
            let label_x = xs;
            let label_y = self.height + 20.0;
            let hour = (current_hour + x_val) % 24.0;
            let period = if hour < 12.0 { "am" } else { "pm" };
            let display_hour = if hour == 0.0 && period == "am" {
                12.0
            } else if hour > 12.0 {
                hour - 12.0
            } else {
                hour
            };
            let label_str = format!("{display_hour:.0}{period}");

            x_labels.push_str(&format!(
                r#"<text x="{x}" y="{y}" fill="{colour}" font-size="{DEFAULT_AXIS_LABEL_FONT_SIZE}" text-anchor="middle">{text}</text>"#,
                x = label_x,
                y = label_y,
                colour = self.text_colour,
                text = label_str
            ));
        }

        // Add tomorrow day name vertically in the graph just like the guidelines
        if current_hour != 0.0 {
            x_labels.push_str(
                self.draw_tomorrow_line(map_x(24.0 - current_hour), clock)
                    .as_str(),
            );
        }
        x_labels
    }

    fn draw_tomorrow_line(&self, x_coor: f32, clock: &dyn Clock) -> String {
        let tomorrow_day_name = clock
            .now_local(self.tz)
            .checked_add_days(chrono::Days::new(1))
            .map(|d| d.format("%A").to_string())
            .unwrap_or_else(|| "Tomorrow".to_string());

        format!(
            r#"<line x1="{x}" y1="0" x2="{x}" y2="{chart_height}" stroke="{colour}" stroke-width="2" stroke-dasharray="3,3" />
                   <text x="{x_text}" y="{y_text}" fill="{colour}" font-size="{DEFAULT_AXIS_LABEL_FONT_SIZE}" font-style="{font_style}"  transform="rotate(-90, {rotate_x_text}, {rotate_y_text})" text-anchor="start">{tomorrow_day_name}</text>"#,
            x = x_coor,
            chart_height = self.height,
            x_text = x_coor + 10.0,
            y_text = (self.height / 2.0) + 20.0,
            font_style = FontStyle::Italic,
            rotate_x_text = x_coor + 10.0 - 30.0,
            rotate_y_text = (self.height / 2.0) - 15.0,
            colour = self.text_colour,
            tomorrow_day_name = tomorrow_day_name
        )
    }

    fn initialize_x_y_bounds(&mut self) {
        for curve in &self.curves {
            match curve {
                CurveType::ActualTemp(data) | CurveType::TempFeelLike(data) => {
                    let min_y_data = data.points.iter().map(|val| val.y).fold(f32::NAN, f32::min);
                    let max_y_data = data.points.iter().map(|val| val.y).fold(f32::NAN, f32::max);
                    let starting_x_data = data.points.first().map(|val| val.x).unwrap_or(0.0);
                    let ending_x_data = data.points.last().map(|val| val.x).unwrap_or(0.0);

                    self.min_y = self.min_y.min(min_y_data);
                    self.max_y = self.max_y.max(max_y_data);
                    self.starting_x = starting_x_data;
                    self.ending_x = ending_x_data;
                }
                CurveType::PrecipitationChance(data) => {
                    let starting_x_data = data
                        .points
                        .first()
                        .map(|val: &PrecipitationPoint| val.x)
                        .unwrap_or(0.0);
                    let ending_x_data = data.points.last().map(|val| val.x).unwrap_or(0.0);
                    self.starting_x = starting_x_data;
                    self.ending_x = ending_x_data;
                }
            }
        }

        // println!(
        //     "starting x: {}, ending x: {}",
        //     self.starting_x, self.ending_x
        // );
        logger::detail(format!(
            "24h forecast range: Min {}°, Max {}°",
            self.min_y, self.max_y
        ));
    }

    pub fn draw_uv_gradient_over_time(&self) -> String {
        let mut gradient = String::new();

        for (i, &uv) in self.uv_data.iter().enumerate() {
            let offset = (i as f32 / 23.0) * 100.0;
            let colour = UVIndexIcon::from(uv).to_colour();
            gradient.push_str(&format!(
                r#"<stop offset="{offset:.2}%" stop-color="{colour}"/>"#
            ));
        }

        gradient
    }

    /// Select precipitation pattern based on chance percentage and whether the hour is
    /// primarily snow (pre-computed from `Precipitation::is_primarily_snow()`).
    pub(crate) fn select_precipitation_pattern(
        _chance: f32,
        is_snow: bool,
    ) -> PrecipitationPattern {
        if is_snow {
            PrecipitationPattern::Snow
        } else {
            PrecipitationPattern::Rain
        }
    }

    pub fn draw_graph(&mut self) -> Result<Vec<GraphDataPath>, Error> {
        // Calculate the minimum and maximum x values from the points
        let mut data_path = vec![];

        self.initialize_x_y_bounds();
        for curve in &self.curves {
            // println!("Data: {:?}", data);
            // Calculate scaling factors for x and y to fit the graph within the given width and height
            let xfactor = self.width / self.ending_x;
            let yfactor = match curve {
                CurveType::PrecipitationChance(_) => self.height / 100.0, // Rain data is in percentage
                CurveType::ActualTemp(_) | CurveType::TempFeelLike(_) => {
                    if self.max_y >= 0.0 && self.min_y < 0.0 {
                        self.height / (self.max_y + self.min_y.abs())
                    } else if self.min_y < 0.0 && self.max_y < 0.0 {
                        // both are negative - use the absolute difference
                        self.height / (self.min_y.abs() - self.max_y.abs())
                    } else {
                        // when both are positive
                        self.height / (self.max_y - self.min_y)
                    }
                }
            };

            // println!("X factor: {}, Y factor: {}", xfactor, yfactor);

            match curve {
                CurveType::PrecipitationChance(precipitation_data) => {
                    // Scale precipitation points according to the calculated factors.
                    let scaled_points: Vec<Point> = precipitation_data
                        .points
                        .iter()
                        .map(|val| Point {
                            x: val.x * xfactor,
                            y: val.chance * yfactor,
                        })
                        .collect();

                    // Generate individual blocks for each hour with stepped path segments
                    let mut blocks = Vec::new();

                    for i in 0..scaled_points.len() {
                        let current = scaled_points[i];
                        let next = if i + 1 < scaled_points.len() {
                            scaled_points[i + 1]
                        } else {
                            // Last point - extrapolate one step, but clamp to graph width
                            // so the final block does not overflow past the right Y-axis.
                            let step = current.x - if i > 0 { scaled_points[i - 1].x } else { 0.0 };
                            Point {
                                x: (current.x + step).min(self.width),
                                y: current.y,
                            }
                        };

                        // Get original precipitation chance percentage and snow flag for pattern selection
                        let precip_chance = precipitation_data.points[i].chance;
                        let is_snow = precipitation_data.points[i].is_primarily_snow;
                        let pattern = Self::select_precipitation_pattern(precip_chance, is_snow);

                        // Interpolated block: top-left -> bottom-left -> bottom-right -> top-right
                        // Bottom edge tapers from current.y to next.y, smoothly blending between hours.
                        let block_path = format!(
                            "M {:.4} 0 L {:.4} {:.4} L {:.4} {:.4} L {:.4} 0 Z",
                            current.x, current.x, current.y, next.x, next.y, next.x
                        );

                        blocks.push(PrecipitationBlock {
                            path: block_path,
                            pattern,
                            x_start: current.x,
                            x_end: next.x,
                            height_left: current.y,
                            height_right: next.y,
                            max_height: current.y.max(next.y),
                            chance: precip_chance,
                        });
                    }

                    data_path.push(GraphDataPath::Precipitation(blocks));
                }
                CurveType::ActualTemp(data) | CurveType::TempFeelLike(data) => {
                    // Scale the points according to the calculated factors.
                    let scaled_points: Vec<Point> = data
                        .points
                        .iter()
                        .map(|val| Point {
                            x: (val.x * xfactor),
                            y: if self.min_y < 0.0 {
                                (val.y + self.min_y.abs()) * yfactor
                            } else {
                                (val.y - self.min_y) * yfactor
                            },
                        })
                        .collect();

                    // Generate the SVG path data for temperature curves
                    let path = if data.smooth {
                        catmull_rom_to_bezier(scaled_points)
                            .iter()
                            .enumerate()
                            .map(|(i, val)| {
                                if i == 0 {
                                    format!("M {:.4} {:.4}", val.c1.x, val.c1.y)
                                } else {
                                    val.to_svg()
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("")
                    } else {
                        scaled_points
                            .iter()
                            .enumerate()
                            .map(|(i, val)| {
                                if i == 0 {
                                    format!("M {:.4} {:.4}", val.x, val.y)
                                } else {
                                    val.to_svg()
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("")
                    };

                    match curve {
                        CurveType::ActualTemp(_) => {
                            data_path.push(GraphDataPath::Temp(path));
                        }
                        CurveType::TempFeelLike(_) => {
                            data_path.push(GraphDataPath::TempFeelLike(path));
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
        Ok(data_path)
    }
}

use crate::{constants::DEFAULT_AXIS_LABEL_FONT_SIZE, weather::icons::UVIndexIcon};
use anyhow::Error;
use strum_macros::Display;

#[derive(Clone, Debug, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
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
pub struct GraphData {
    pub points: Vec<Point>,
    pub smooth: bool,
}

#[derive(Clone, Debug)]
pub enum CurveType {
    ActualTemp(GraphData),
    TempFeelLike(GraphData),
    RainChance(GraphData),
}

impl CurveType {
    fn data(&self) -> &GraphData {
        match self {
            Self::ActualTemp(data) | Self::TempFeelLike(data) | Self::RainChance(data) => data,
        }
    }

    pub fn get_points(&self) -> &Vec<Point> {
        &self.data().points
    }

    pub fn get_smooth(&self) -> bool {
        self.data().smooth
    }
}

impl GraphData {
    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push(Point { x, y })
    }
}
pub struct HourlyForecastGraph {
    pub curves: Vec<CurveType>,
    pub uv_data: [usize; 24],
    pub height: f64,
    pub width: f64,
    pub starting_x: f64,
    pub ending_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub x_ticks: usize,
    pub y_left_ticks: usize,
    pub y_right_ticks: usize,
    pub x_axis_always_at_min: bool,
    pub text_colour: String,
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
                CurveType::RainChance(GraphData {
                    points: vec![],
                    smooth: false,
                }),
            ],
            uv_data: [0; 24],
            height: 300.0,
            width: 600.0,
            starting_x: 0.0,
            ending_x: 23.0,
            min_y: f64::INFINITY,
            max_y: -f64::INFINITY,
            // Number of ticks, +1 because of the fencepost problem
            x_ticks: 6,
            y_left_ticks: 5,
            y_right_ticks: 5,
            x_axis_always_at_min: false,
            text_colour: "black".to_string(),
        }
    }
}

pub enum GraphDataPath {
    Temp(String),
    TempFeelLike(String),
    Rain(String),
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

type UVIndexCategory = UVIndexIcon;

impl UVIndexCategory {
    pub fn from_u8(value: usize) -> Self {
        match value {
            0 => UVIndexCategory::None,
            1..=2 => UVIndexCategory::Low,
            3..=5 => UVIndexCategory::Moderate,
            6..=7 => UVIndexCategory::High,
            8..=10 => UVIndexCategory::VeryHigh,
            11.. => UVIndexCategory::Extreme,
        }
    }

    pub fn to_colour(self) -> &'static str {
        match self {
            UVIndexCategory::None => "transparent",
            UVIndexCategory::Low => "green",
            UVIndexCategory::Moderate => "yellow",
            UVIndexCategory::High => "orange",
            UVIndexCategory::VeryHigh => "red",
            UVIndexCategory::Extreme => "purple",
        }
    }
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
    const TENSION: f64 = 6.0; // Catmull_Rom_standard, adjust to make curves more or less smooth

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
    pub fn create_axis_with_labels(&self, current_hour: f64) -> AxisPaths {
        let range_x = self.ending_x - self.starting_x + 1.0; // +1 because last hour is 23
        let range_y_left = self.max_y - self.min_y;
        let range_y_right = 100.0; // Rain data is in percentage

        // Mapping functions from data space to SVG space
        // x data domain maps to [0, width]
        // y data domain maps to [height, 0] (SVG y goes down)
        let map_x = |x: f64| (x - self.starting_x) * (self.width / range_x);
        let map_y_left = |y: f64| self.height - ((y - self.min_y) * (self.height / range_y_left));
        // For the right axis, we assume 0 to 100% maps directly onto the height.
        let map_y_right = |y: f64| self.height - (y * (self.height / range_y_right));

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

        let x_step = range_x / self.x_ticks as f64;
        let y_left_step = range_y_left / self.y_left_ticks as f64;
        let y_right_step = range_y_right / self.y_right_ticks as f64;

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
        map_y_right: impl Fn(f64) -> f64,
        y_right_axis_x: f64,
        y_right_axis_path: &mut String,
        y_right_step: f64,
    ) -> String {
        let mut y_right_labels = String::new();
        for k in 0..=self.y_right_ticks {
            let y_val = k as f64 * y_right_step; // percentage step
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
            let label_str = format!("{:.0}%", y_val);
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
        map_y_left: impl Fn(f64) -> f64,
        y_axis_x: f64,
        y_left_axis_path: &mut String,
        y_left_step: f64,
    ) -> String {
        let mut y_left_labels = String::new();
        for j in 0..=self.y_left_ticks {
            let y_val = self.min_y + j as f64 * y_left_step;
            if y_val > self.max_y {
                break;
            }
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
            let mut label_str = format!("{:.1}°", y_val);
            let mut font_size = DEFAULT_AXIS_LABEL_FONT_SIZE;
            if j == 0 || j == self.y_left_ticks {
                label_str = format!("{:.0}°", y_val);
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

    fn generate_x_axis_labels(
        &self,
        current_hour: f64,
        map_x: impl Fn(f64) -> f64,
        x_axis_y: f64,
        x_axis_path: &mut String,
        x_axis_guideline_path: &mut String,
        x_step: f64,
    ) -> String {
        let mut x_val: f64 = 0.0;
        let mut x_labels = String::new();
        for i in 0..=self.x_ticks {
            if x_val > self.ending_x {
                break;
            }
            x_val = self.starting_x + i as f64 * x_step;

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
                    r#" M {} {} v -{} m 0 2 v -2"#,
                    xs, x_guideline_len, x_guideline_len
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
            let label_str = format!("{:.0}{}", display_hour, period);

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
            x_labels.push_str(self.draw_tomorrow_line(map_x(24.0 - current_hour)).as_str());
        }
        x_labels
    }

    fn draw_tomorrow_line(&self, x_coor: f64) -> String {
        let tomorrow_day_name = chrono::Local::now()
            .checked_add_days(chrono::Days::new(1))
            .map(|d| d.format("%A").to_string())
            .unwrap_or_else(|| "Tomorrow".to_string());

        format!(
            r#"<line x1="{x}" y1="0" x2="{x}" y2="{chart_height}" stroke="{colour}" stroke-width="2" stroke-dasharray="3,3" />
                   <text x="{x_text}" y="{y_text}" fill="{colour}" font-size="{DEFAULT_AXIS_LABEL_FONT_SIZE}" font-style="{font_style}"  transform="rotate(-90, {rotate_x_text}, {rotate_y_text})" text-anchor="start">{tomorrow_day_name}</text>"#,
            x = x_coor,
            chart_height = self.height,
            x_text = x_coor + 10.0,
            y_text = self.height / 2.0,
            font_style = FontStyle::Italic,
            rotate_x_text = x_coor + 10.0 - 30.0,
            rotate_y_text = (self.height / 2.0) - 35.0,
            colour = self.text_colour,
            tomorrow_day_name = tomorrow_day_name
        )
    }

    fn initialize_x_y_bounds(&mut self) {
        for curve in &self.curves {
            let min_y_data = curve
                .get_points()
                .iter()
                .map(|val| val.y)
                .fold(f64::NAN, f64::min);
            let max_y_data = curve
                .get_points()
                .iter()
                .map(|val| val.y)
                .fold(f64::NAN, f64::max);

            let starting_x_data = curve.get_points().first().map(|val| val.x).unwrap_or(0.0);
            let ending_x_data = curve.get_points().last().map(|val| val.x).unwrap_or(0.0);

            match curve {
                CurveType::RainChance(_) => {}
                CurveType::ActualTemp(_) | CurveType::TempFeelLike(_) => {
                    self.min_y = self.min_y.min(min_y_data);
                    self.max_y = self.max_y.max(max_y_data);
                }
            }
            self.starting_x = starting_x_data;
            self.ending_x = ending_x_data;
        }

        // println!(
        //     "starting x: {}, ending x: {}",
        //     self.starting_x, self.ending_x
        // );
        println!(
            "24h forecast Global Min y: {}, Max y: {}",
            self.min_y, self.max_y
        );
    }

    pub fn draw_uv_gradient_over_time(&self) -> String {
        // println!("UV data: {:?}", self.uv_data);
        let mut gradient = String::new();

        for (i, &uv) in self.uv_data.iter().enumerate() {
            let offset = (i as f64 / 23.0) * 100.0;
            let colour = UVIndexCategory::from_u8(uv).to_colour();
            gradient.push_str(&format!(
                r#"<stop offset="{:.2}%" stop-color="{}"/>"#,
                offset, colour
            ));
        }

        gradient
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
                CurveType::RainChance(_) => self.height / 100.0, // Rain data is in percentage
                CurveType::ActualTemp(_) | CurveType::TempFeelLike(_) => {
                    if self.max_y >= 0.0 && self.min_y < 0.0 {
                        self.height / (self.max_y + self.min_y.abs())
                    } else if self.min_y < 0.0 {
                        // it's possible for both to be negative
                        self.height / (self.max_y.abs() - self.min_y.abs())
                    } else {
                        // when both are positive
                        self.height / (self.max_y - self.min_y)
                    }
                }
            };

            // println!("X factor: {}, Y factor: {}", xfactor, yfactor);

            // Scale the points according to the calculated factors
            let scaled_points: Vec<Point> = curve
                .get_points()
                .iter()
                .map(|val| Point {
                    x: (val.x * xfactor), // x always start from 0 so no need to adjust the x value
                    y: match curve {
                        CurveType::RainChance(_) => val.y * yfactor,
                        CurveType::ActualTemp(_) | CurveType::TempFeelLike(_) => {
                            // If the minimum y value is negative, we need to adjust the y value
                            // to ensure it's correctly placed on the graph
                            if self.min_y < 0.0 {
                                (val.y + self.min_y.abs()) * yfactor
                            } else {
                                (val.y - self.min_y) * yfactor
                            }
                        }
                    },
                })
                .collect();

            // Generate the SVG path data
            let path = if curve.get_smooth() {
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
                CurveType::RainChance(_) => {
                    let bounding_area_path = format!("{} L {} 0 L 0 0Z", path, self.width);
                    data_path.push(GraphDataPath::Rain(bounding_area_path));
                }
            }
        }
        Ok(data_path)
    }
}

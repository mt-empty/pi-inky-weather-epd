// mod apis;
mod bom;
mod chart;
pub mod context;
mod errors;
mod pimironi_image_py;
mod settings;
mod update;
mod utils;

// #[cfg(debug_assertions)]
// mod dev;

// #[cfg(debug_assertions)]
// use dev::create_striped_png;

use anyhow::Error;
use bom::*;
use chart::{catmull_rom_to_bezier, Point};
use chrono::{Datelike, Timelike};
use context::Context;
use errors::*;
use lazy_static::lazy_static;
use pimironi_image_py::invoke_pimironi_image_script;
use serde::Deserialize;
use settings::DashboardSettings;
use std::io::Write;
use std::{fs, path::PathBuf};
use strum_macros::Display;
use tinytemplate::{format_unescaped, TinyTemplate};
use update::update_app;
pub use utils::*;

pub const NOT_AVAILABLE_ICON: &str = "not-available.svg";
pub const DEFAULT_AXIS_LABEL_FONT_SIZE: usize = 19;

lazy_static! {
    pub static ref CONFIG: DashboardSettings =
        DashboardSettings::new().expect("Failed to load config");
}

#[derive(Clone, Debug)]
pub enum DataType {
    Temp,
    TempFeelLike,
    Rain,
}

#[derive(Clone, Debug)]
pub struct GraphData {
    pub graph_type: DataType,
    pub points: Vec<Point>,
    pub smooth: bool,
}

impl GraphData {
    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push(Point { x, y })
    }
}
pub struct DailyForecastGraph {
    pub name: String,
    pub data: Vec<GraphData>,
    pub height: f64,
    pub width: f64,
    pub starting_x: f64,
    pub ending_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub x_ticks: usize,
    pub y_left_ticks: usize,
    pub y_right_ticks: usize,
}

impl Default for DailyForecastGraph {
    fn default() -> Self {
        Self {
            name: "Hourly Forecast".to_string(),
            data: vec![],
            height: 300.0,
            width: 600.0,
            starting_x: 0.0,
            ending_x: 23.0,
            min_y: f64::INFINITY,
            max_y: -f64::INFINITY,
            // Number of ticks, +1 to each because of the fencepost problem
            x_ticks: 6,
            y_left_ticks: 5,
            y_right_ticks: 5,
        }
    }
}

pub enum GraphDataPath {
    Temp(String),
    TempFeelLike(String),
    Rain(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UVIndexCategory {
    None,
    Low,
    Moderate,
    High,
    VeryHigh,
    Extreme,
    Hazardous,
}

impl UVIndexCategory {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => UVIndexCategory::None,
            1..=2 => UVIndexCategory::Low,
            3..=5 => UVIndexCategory::Moderate,
            6..=7 => UVIndexCategory::High,
            8..=10 => UVIndexCategory::VeryHigh,
            11..=12 => UVIndexCategory::Extreme,
            _ => UVIndexCategory::Hazardous,
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
            UVIndexCategory::Hazardous => "black",
        }
    }
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

impl DailyForecastGraph {
    fn create_axis_with_labels(&self, current_hour: f64) -> AxisPaths {
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
        let x_axis_y = if CONFIG.render_options.x_axis_always_at_min
            || self.min_y > 0.0 && self.max_y > 0.00
        {
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
                colour = CONFIG.colours.text_colour,
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
                colour = CONFIG.colours.text_colour,
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
                colour = CONFIG.colours.text_colour,
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
            .date_naive()
            .checked_add_days(chrono::Days::new(1))
            .map(|d| d.format("%A").to_string())
            .unwrap_or_else(|| "Tomorrow".to_string());

        format!(
            r#"<line x1="{x}" y1="0" x2="{x}" y2="{chart_height}" stroke="{colour}" stroke-width="2" stroke-dasharray="3,3" />
                   <text x="{x_text}" y="{y_text}" fill="{colour}" font-size="{DEFAULT_AXIS_LABEL_FONT_SIZE}" font-style="italic"  transform="rotate(-90, {rotate_x_text}, {rotate_y_text})" text-anchor="start">{tomorrow_day_name}</text>"#,
            x = x_coor,
            chart_height = self.height,
            x_text = x_coor + 10.0,
            y_text = self.height / 2.0,
            rotate_x_text = x_coor + 10.0 - 30.0,
            rotate_y_text = (self.height / 2.0) - 35.0,
            colour = CONFIG.colours.text_colour,
            tomorrow_day_name = tomorrow_day_name
        )
    }

    fn initialize_x_y_bounds(&mut self) {
        for data in &self.data {
            let min_y_data = data.points.iter().map(|val| val.y).fold(f64::NAN, f64::min);
            let max_y_data = data.points.iter().map(|val| val.y).fold(f64::NAN, f64::max);

            let starting_x_data = data.points.first().map(|val| val.x).unwrap_or(0.0);
            let ending_x_data = data.points.last().map(|val| val.x).unwrap_or(0.0);

            match data.graph_type {
                DataType::Rain => {}
                DataType::TempFeelLike | DataType::Temp => {
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

    fn draw_uv_gradient_over_time(&self, uv_data: [usize; 24]) -> String {
        // println!("UV data: {:?}", uv_data);
        let mut gradient = String::new();

        for (i, &uv) in uv_data.iter().enumerate() {
            let offset = (i as f64 / 23.0) * 100.0;
            let colour = UVIndexCategory::from_u8(uv as u8).to_colour();
            gradient.push_str(&format!(
                r#"<stop offset="{:.2}%" stop-color="{}"/>"#,
                offset, colour
            ));
        }

        gradient
    }

    fn draw_graph(&mut self) -> Result<Vec<GraphDataPath>, Error> {
        // Calculate the minimum and maximum x values from the points
        let mut data_path = vec![];

        self.initialize_x_y_bounds();
        for data in &self.data {
            // println!("Data: {:?}", data);
            // Calculate scaling factors for x and y to fit the graph within the given width and height
            let xfactor = self.width / self.ending_x;
            let yfactor = match data.graph_type {
                DataType::Rain => self.height / 100.0, // Rain data is in percentage
                DataType::Temp | DataType::TempFeelLike => {
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
            let points: Vec<Point> = data
                .points
                .iter()
                .map(|val| Point {
                    x: (val.x * xfactor), // x always start from 0 so no need to adjust the x value
                    y: match data.graph_type {
                        DataType::Rain => val.y * yfactor,
                        DataType::Temp | DataType::TempFeelLike => {
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
            let path = if data.smooth {
                catmull_rom_to_bezier(points)
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
                points
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

            match data.graph_type {
                DataType::Temp => {
                    data_path.push(GraphDataPath::Temp(path));
                }
                DataType::TempFeelLike => {
                    data_path.push(GraphDataPath::TempFeelLike(path));
                }
                DataType::Rain => {
                    let bounding_area_path = format!("{} L {} 0 L 0 0Z", path, self.width);
                    data_path.push(GraphDataPath::Rain(bounding_area_path));
                }
            }
        }
        Ok(data_path)
    }
}

#[derive(Deserialize, Debug, Display)]
enum RainChanceName {
    #[strum(to_string = "clear")]
    Clear,
    #[strum(to_string = "partly-cloudy")]
    PartlyCloudy,
    #[strum(to_string = "overcast")]
    Overcast,
    #[strum(to_string = "extreme")]
    Extreme,
}

#[derive(Deserialize, Debug, Display)]
enum RainAmountName {
    #[strum(to_string = "")]
    None,
    #[strum(to_string = "-drizzle")]
    Drizzle,
    #[strum(to_string = "-rain")]
    Rain,
}

#[derive(Deserialize, Debug, Display)]
enum DayNight {
    #[strum(to_string = "-day")]
    Day,
    #[strum(to_string = "-night")]
    Night,
}

/// A trait representing an icon with methods to get its name and path.
///
/// # Methods
///
/// - `get_icon_name(&self) -> String`
///
///   Returns the name of the icon as a `String`.
///
/// - `get_icon_path(&self) -> String`
///
///   Returns the full path to the icon as a `String`. The path is constructed
///   by concatenating the `svg_icons_directory` from the `CONFIG` with the icon name.
///
/// - `rain_amount_to_name(amount: u32) -> String`
///
///   Converts a given rain amount (in millimetres) to a corresponding name.
///   This method should not be part of this trait and needs to be refactored.
///
///   - `amount: u32` - The amount of rain in millimetres.
///   - Returns a `String` representing the rain amount name:
///     - `""` for 0 to 2 mm
///     - `"Drizzle"` for 3 to 20 mm
///     - `"Rain"` for 21 mm and above
///
/// - `rain_chance_to_name(chance: u32) -> String`
///
///   Converts a given rain chance (in percentage) to a corresponding name.
///   This method should not be part of this trait and needs to be refactored.
///
///   - `chance: u32` - The chance of rain in percentage.
///   - Returns a `String` representing the rain chance name:
///     - `"Clear"` for 0 to 25%
///     - `"PartlyCloudy"` for 26 to 50%
///     - `"Overcast"` for 51 to 75%
///     - `"Extreme"` for 76% and above
trait Icon {
    /// Returns the name of the icon as a `String`.
    fn get_icon_name(&self) -> String;

    /// Returns the path of the icon as a `String`.
    /// The path is constructed using the `svg_icons_directory` from the configuration
    /// and the icon name obtained from `get_icon_name`.
    fn get_icon_path(&self) -> String {
        format!(
            "{}{}",
            CONFIG.misc.svg_icons_directory,
            self.get_icon_name()
        )
    }

    /// Converts a given rain amount (in millimetres) to a corresponding name.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount of rain.
    ///
    /// # Returns
    ///
    /// * A `String` representing the name corresponding to the rain amount.
    fn rain_amount_to_name(mut amount: u32, is_hourly: bool) -> String {
        // TODO: this should be moved to a more appropriate place
        if is_hourly {
            amount *= 24;
        }
        match amount {
            0..=2 => RainAmountName::None.to_string(),
            3..=20 => RainAmountName::Drizzle.to_string(),
            21.. => RainAmountName::Rain.to_string(),
        }
    }

    /// Converts a given rain chance (in percentage) to a corresponding name as a `String`.
    ///
    /// # Arguments
    ///
    /// * `chance` - The chance of rain.
    ///
    /// # Returns
    ///
    /// * A `String` representing the name corresponding to the rain chance.
    fn rain_chance_to_name(chance: u32) -> String {
        // TODO: this should be moved to a more appropriate place
        match chance {
            0..=25 => RainChanceName::Clear.to_string(),
            26..=50 => RainChanceName::PartlyCloudy.to_string(),
            51..=75 => RainChanceName::Overcast.to_string(),
            76.. => RainChanceName::Extreme.to_string(),
        }
    }
}

impl Icon for Wind {
    fn get_icon_name(&self) -> String {
        let icon = match self.speed_kilometre {
            0.0..=20.0 => "wind.svg",
            20.1..=40.0 => "umbrella-wind.svg",
            40.1.. => "umbrella-wind-alt.svg",
            _ => NOT_AVAILABLE_ICON,
        };
        icon.to_string()
    }
}

impl Icon for RainAmount {
    fn get_icon_name(&self) -> String {
        "raindrop-measure.svg".to_string()
    }
}

impl Icon for UV {
    fn get_icon_name(&self) -> String {
        match self.max_index {
            Some(_index) => "uv-index.svg".to_string(),
            // Some(index) => match index {
            //     0.. => "uv-index.svg".to_string(),
            //     // 1..=11 => format!("uv-index-{}.svg", index),
            //     _ => NOT_AVAILABLE_ICON.to_string(),
            // },
            None => NOT_AVAILABLE_ICON.to_string(),
        }
    }
}

impl Icon for DailyEntry {
    fn get_icon_name(&self) -> String {
        // TODO: this might generate invalid icon names, like clear-day/night-drzzile/rain
        let temp = format!(
            "{}{}{}.svg",
            DailyEntry::rain_chance_to_name(self.rain.as_ref().unwrap().chance.unwrap_or(0)),
            DayNight::Day,
            DailyEntry::rain_amount_to_name(
                self.rain
                    .as_ref()
                    .unwrap()
                    .amount
                    .min
                    .unwrap_or(0.0)
                    .round() as u32,
                false
            )
        );
        temp
    }
}

type RelativeHumidity = f64;

impl Icon for RelativeHumidity {
    fn get_icon_name(&self) -> String {
        "humidity.svg".to_string()
    }
}

impl Icon for HourlyForecast {
    fn get_icon_name(&self) -> String {
        let temp = format!(
            "{}{}{}.svg",
            HourlyForecast::rain_chance_to_name(self.rain.chance.unwrap_or(0)),
            if self.is_night {
                DayNight::Night.to_string()
            } else {
                DayNight::Day.to_string()
            },
            HourlyForecast::rain_amount_to_name(
                self.rain.amount.min.unwrap_or(0.0).round() as u32,
                true
            )
        );
        temp
    }
}

pub enum FetchOutcome<T> {
    Fresh(T),
    Stale { data: T, error: DashboardError },
}

fn load_cached<T: for<'de> Deserialize<'de>>(file_path: &PathBuf) -> Result<T, Error> {
    let cached = fs::read_to_string(file_path).map_err(|_| {
        Error::msg(
            "Weather data JSON file not found. If this is your first time running the application. Please ensure 'disable_network_requests' is set to false in the configuration so data can be cached.",
        )
    }).map_err(|_| {
        DashboardError::NoInternet {
            details: "No hourly forecast data available".to_string(),
        }
    })?;
    let data = serde_json::from_str(&cached).map_err(Error::msg)?;
    Ok(data)
}

fn fallback<T: for<'de> Deserialize<'de>>(
    file_path: &PathBuf,
    dashboard_error: DashboardError,
) -> Result<FetchOutcome<T>, Error> {
    let data = load_cached(file_path)?;
    Ok(FetchOutcome::Stale {
        data,
        error: dashboard_error,
    })
}

fn fetch_data<T: for<'de> Deserialize<'de>>(
    endpoint: &str,
    file_path: &PathBuf,
) -> Result<FetchOutcome<T>, Error> {
    if !file_path.exists() {
        fs::create_dir_all(file_path.parent().unwrap())?;
    }

    if !CONFIG.debugging.disable_network_requests {
        let client = reqwest::blocking::Client::new();
        let response = match client.get(endpoint).send() {
            Ok(res) => res,
            Err(e) => {
                eprintln!("API request failed: {}", e);

                return fallback(
                    file_path,
                    DashboardError::NoInternet {
                        details: e.to_string(),
                    },
                );
            }
        };

        let body = response.text().map_err(Error::msg)?;

        if let Ok(api_error) = serde_json::from_str::<BomError>(&body) {
            if let Some(first_error) = api_error.errors.first() {
                eprintln!("Warning: API request failed, double check your api configs, trying to load cached data");
                let first_error_detail = first_error.detail.clone();
                for (i, error) in api_error.errors.iter().enumerate() {
                    eprintln!("API Errors, {}: {}", i + 1, error.detail);
                }

                return fallback(file_path, DashboardError::ApiError(first_error_detail));
            }
        }

        fs::write(file_path, &body)?;
        let data = serde_json::from_str(&body).map_err(Error::msg)?;
        Ok(FetchOutcome::Fresh(data))
    } else {
        Ok(FetchOutcome::Fresh(load_cached(file_path)?))
    }
}

fn fetch_daily_forecast_data() -> Result<FetchOutcome<DailyForecastResponse>, Error> {
    let file_path =
        std::path::Path::new(&CONFIG.misc.weather_data_store_path).join("daily_forecast.json");
    fetch_data(&DAILY_FORECAST_ENDPOINT, &file_path)
}

fn fetch_hourly_forecast_data() -> Result<FetchOutcome<HourlyForecastResponse>, Error> {
    let file_path =
        std::path::Path::new(&CONFIG.misc.weather_data_store_path).join("hourly_forecast.json");
    fetch_data(&HOURLY_FORECAST_ENDPOINT, &file_path)
}

fn update_current_hour_data(current_hour: &HourlyForecast, context: &mut Context) {
    let mut current_hour_weather_icon = current_hour.get_icon_path();
    if CONFIG.render_options.use_moon_phase_instead_of_clear_night
        && current_hour_weather_icon.ends_with(&format!(
            "{}{}.svg",
            RainChanceName::Clear,
            DayNight::Night
        ))
    {
        println!("'use_moon_phase_instead_of_clear_night' is set to true, using moon phase icon instead of clear night");
        current_hour_weather_icon = get_moon_phase_icon_path().to_string();
    }
    context.current_hour_temp = current_hour.temp.to_string();
    context.current_hour_weather_icon = current_hour_weather_icon;
    context.current_hour_feels_like = current_hour.temp_feels_like.to_string();
    context.current_hour_wind_speed = current_hour.wind.speed_kilometre.to_string();
    context.current_hour_wind_icon = current_hour.wind.get_icon_path();
    context.current_hour_uv_index = current_hour.uv.to_string();
    context.current_hour_relative_humidity = current_hour.relative_humidity.to_string();
    context.current_hour_relative_humidity_icon = current_hour.relative_humidity.get_icon_path();
    context.current_day_name = chrono::Local::now().format("%A, %d %b").to_string();
    context.current_hour_rain_amount = (current_hour.rain.amount.min.unwrap_or(0.0)
        + current_hour.rain.amount.min.unwrap_or(0.0))
    .to_string();
    context.rain_measure_icon = current_hour.rain.amount.get_icon_path();
}

// Extrusion Pattern: force everything through one function until it resembles spaghetti
fn update_daily_forecast_data(context: &mut Context) -> Result<(), Error> {
    let daily_forecast_data = match fetch_daily_forecast_data() {
        Ok(FetchOutcome::Fresh(data)) => data.data,
        Ok(FetchOutcome::Stale { data, error }) => {
            eprintln!("Warning: Using stale data");
            handle_errors(context, error);
            data.data
        }
        Err(e) => return Err(e),
    };
    let current_date = chrono::Local::now().date_naive();
    let mut i = 1;

    for day in daily_forecast_data {
        if let Some(naive_date) = day.date {
            if naive_date.date() < current_date {
                // If the date is in the past, skip it
                continue;
            } else if naive_date.date() > current_date + chrono::Duration::days(7) {
                // If the date is more than 7 days in the future, skip it
                break;
            }
        }

        let min_temp_value = day
            .temp_min
            .map_or("NA".to_string(), |temp| temp.to_string());
        let max_temp_value = day
            .temp_max
            .map_or("NA".to_string(), |temp| temp.to_string());
        let icon_value = day.get_icon_path();
        let day_name_value = day
            .date
            .map_or("NA".to_string(), |date| date.format("%a").to_string());

        println!(
            "{} - Max {} Min {}",
            day_name_value, max_temp_value, min_temp_value
        );
        match i {
            1 => {
                context.sunrise_time = day
                    .astronomical
                    .unwrap_or_default()
                    .sunrise_time
                    .unwrap_or_default()
                    .format("%H:%M")
                    .to_string();
                context.sunset_time = day
                    .astronomical
                    .unwrap_or_default()
                    .sunset_time
                    .unwrap_or_default()
                    .format("%H:%M")
                    .to_string();
            }
            2 => {
                context.day2_mintemp = min_temp_value;
                context.day2_maxtemp = max_temp_value;
                context.day2_icon = icon_value;
                context.day2_name = day_name_value;
            }
            3 => {
                context.day3_mintemp = min_temp_value;
                context.day3_maxtemp = max_temp_value;
                context.day3_icon = icon_value;
                context.day3_name = day_name_value;
            }
            4 => {
                context.day4_mintemp = min_temp_value;
                context.day4_maxtemp = max_temp_value;
                context.day4_icon = icon_value;
                context.day4_name = day_name_value;
            }
            5 => {
                context.day5_mintemp = min_temp_value;
                context.day5_maxtemp = max_temp_value;
                context.day5_icon = icon_value;
                context.day5_name = day_name_value;
            }
            6 => {
                context.day6_mintemp = min_temp_value;
                context.day6_maxtemp = max_temp_value;
                context.day6_icon = icon_value;
                context.day6_name = day_name_value;
            }
            7 => {
                context.day7_mintemp = min_temp_value;
                context.day7_maxtemp = max_temp_value;
                context.day7_icon = icon_value;
                context.day7_name = day_name_value;
            }
            _ => {}
        }

        i += 1;
    }

    if i < 8 {
        println!("Warning: Less than 7 days of daily forecast data, Using stale data");
        handle_errors(
            context,
            DashboardError::NoInternet {
                details: "Warning: Less than 7 days of daily forecast data, Using stale data"
                    .to_string(),
            },
        );
    }

    Ok(())
}

fn update_hourly_forecast_data(context: &mut Context) -> Result<(), Error> {
    let hourly_forecast_data = match fetch_hourly_forecast_data() {
        Ok(FetchOutcome::Fresh(data)) => data.data,
        Ok(FetchOutcome::Stale { data, error }) => {
            eprintln!("Warning: Using stale data");
            handle_errors(context, error);
            data.data
        }
        Err(e) => return Err(e),
    };

    let mut graph = DailyForecastGraph::default();

    let mut temp_data = GraphData {
        graph_type: DataType::Temp,
        points: vec![],
        smooth: true,
    };

    let mut feels_like_data = GraphData {
        graph_type: DataType::TempFeelLike,
        points: vec![],
        smooth: true,
    };

    let mut rain_data = GraphData {
        graph_type: DataType::Rain,
        points: vec![],
        smooth: false,
    };

    let current_uv = UV {
        category: None,
        end_time: None,
        max_index: Some(hourly_forecast_data[0].uv as u32),
        start_time: None,
    };

    let current_date = chrono::Local::now()
        .naive_local()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    println!("Current time: {:?}", current_date);
    // we only want to display the next 24 hours
    let first_date = hourly_forecast_data.iter().find_map(
        // find the first time
        |forecast| {
            if forecast.time >= current_date {
                Some(forecast.time)
            } else {
                None
            }
        },
    );
    let forecast_window_start = match first_date {
        Some(date) => date,
        None => {
            handle_errors(
                context,
                DashboardError::NoInternet {
                    details: "No hourly forecast data available, Could Not find a date later than the current date".to_string(),
                },
            );
            return Ok(());
        }
    };

    let forecast_window_end = forecast_window_start + chrono::Duration::hours(24);

    println!("24h forecast window start: {:?}", forecast_window_start);
    println!("24h forecast window end: {:?}", forecast_window_end);

    let mut x = 0.0;

    let mut uv_data = [0; 24];

    hourly_forecast_data
        .iter()
        .filter(|forecast| {
            forecast.time >= forecast_window_start && forecast.time < forecast_window_end
        })
        .for_each(|forecast| {
            if x == 0.0 {
                // update current hour
                update_current_hour_data(forecast, context);
            }
            // we won't push the actual hour right now
            // we can calculate it later
            // we push this index to make scaling graph easier
            temp_data.add_point(x, forecast.temp);
            feels_like_data.add_point(x, forecast.temp_feels_like);
            rain_data.add_point(x, forecast.rain.chance.unwrap_or(0).into());
            uv_data[x as usize] = forecast.uv as usize;
            x += 1.0;
        });

    graph
        .data
        .extend(vec![feels_like_data, temp_data, rain_data]);

    let svg_result = graph.draw_graph().unwrap();

    let (temp_curve_data, feel_like_curve_data, rain_curve_data): (String, String, String) =
        svg_result.iter().fold(
            (String::new(), String::new(), String::new()),
            |(mut temp_acc, mut feel_like_acc, mut rain_acc), path| {
                match path {
                    GraphDataPath::Temp(data) => temp_acc.push_str(data),
                    GraphDataPath::TempFeelLike(data) => feel_like_acc.push_str(data),
                    GraphDataPath::Rain(data) => rain_acc.push_str(data),
                }
                (temp_acc, feel_like_acc, rain_acc)
            },
        );

    let day_start = forecast_window_start
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    let day_end = day_start + chrono::Duration::days(1);

    // println!("day_start: {:?}", day_start);
    // println!("day_end: {:?}", day_end);

    context.graph_height = graph.height.to_string();
    context.graph_width = graph.width.to_string();
    context.temp_curve_data = temp_curve_data;
    context.feel_like_curve_data = feel_like_curve_data;
    context.rain_curve_data = rain_curve_data;
    context.uv_index = hourly_forecast_data[0].uv.to_string();
    context.uv_index_icon = current_uv.get_icon_path().to_string();
    context.wind_speed = hourly_forecast_data[0].wind.speed_kilometre.to_string();

    let max_wind_today = find_max_item_between_dates(
        &hourly_forecast_data,
        &forecast_window_start,
        &day_end,
        |item| item.wind.gust_speed_kilometre.unwrap_or(0.0),
        |item| &item.time,
    );

    let max_wind_tomorrow = find_max_item_between_dates(
        &hourly_forecast_data,
        &day_end,
        &forecast_window_end,
        |item| item.wind.gust_speed_kilometre.unwrap_or(0.0),
        |item| &item.time,
    );

    if max_wind_today > max_wind_tomorrow {
        context.max_gust_speed = max_wind_today.to_string();
    } else {
        context.max_gust_speed = max_wind_tomorrow.to_string();
        context.max_gust_speed_font_style = "italic".to_string();
    }

    let max_uv_index = find_max_item_between_dates(
        &hourly_forecast_data,
        &forecast_window_start,
        &day_end,
        |item| item.uv,
        |item| &item.time,
    );

    let max_uv_index_tomorrow = find_max_item_between_dates(
        &hourly_forecast_data,
        &day_end,
        &forecast_window_end,
        |item| item.uv,
        |item| &item.time,
    );

    if max_uv_index > max_uv_index_tomorrow {
        context.max_uv_index = max_uv_index.to_string();
    } else {
        context.max_uv_index = max_uv_index_tomorrow.to_string();
        context.max_uv_index_font_style = "italic".to_string();
    }

    let max_relative_humidity = find_max_item_between_dates(
        &hourly_forecast_data,
        &forecast_window_start,
        &day_end,
        |item| item.relative_humidity,
        |item| &item.time,
    );

    let max_relative_humidity_tomorrow = find_max_item_between_dates(
        &hourly_forecast_data,
        &day_end,
        &forecast_window_end,
        |item| item.relative_humidity,
        |item| &item.time,
    );

    if max_relative_humidity > max_relative_humidity_tomorrow {
        context.max_relative_humidity = max_relative_humidity.to_string();
    } else {
        context.max_relative_humidity = max_relative_humidity_tomorrow.to_string();
        context.max_relative_humidity_font_style = "italic".to_string();
    }

    context.total_rain_today = (get_total_between_dates(
        &hourly_forecast_data,
        &forecast_window_start,
        &forecast_window_end,
        |item: &HourlyForecast| {
            (item.rain.amount.min.unwrap_or(0.0) + item.rain.amount.max.unwrap_or(0.0)) / 2.0
        },
        |item| &item.time,
    ) as usize)
        .to_string();

    context.wind_icon = hourly_forecast_data[0].wind.get_icon_path();

    let axis_data_path = graph.create_axis_with_labels(forecast_window_start.hour() as f64);

    context.x_axis_path = axis_data_path.x_axis_path;
    context.y_left_axis_path = axis_data_path.y_left_axis_path;
    context.x_labels = axis_data_path.x_labels;
    context.y_left_labels = axis_data_path.y_left_labels;
    context.y_right_axis_path = axis_data_path.y_right_axis_path;
    context.y_right_labels = axis_data_path.y_right_labels;
    context.x_axis_guideline_path = axis_data_path.x_axis_guideline_path;

    context.uv_gradient = graph.draw_uv_gradient_over_time(uv_data);
    Ok(())
}

fn get_moon_phase_icon_path() -> String {
    let now = chrono::Local::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();

    // Calculate the approximate age of the moon in days since the last new moon
    let mut moon_age_days = ((year as f64 - 2000.0) * 365.25 + month as f64 * 30.6 + day as f64
        - 2451550.1)
        % 29.530588;
    if moon_age_days < 0.0 {
        moon_age_days += 29.530588; // Ensure positive values
    }

    // Determine the moon phase icon based on the moon age
    let icon_name = match moon_age_days {
        age if age < 1.84566 => "moon-new.svg",
        age if age < 5.53699 => "moon-waxing-crescent.svg",
        age if age < 9.22831 => "moon-first-quarter.svg",
        age if age < 12.91963 => "moon-waxing-gibbous.svg",
        age if age < 16.61096 => "moon-full.svg",
        age if age < 20.30228 => "moon-waning-gibbous.svg",
        age if age < 23.99361 => "moon-last-quarter.svg",
        _ => "moon-waning-crescent.svg",
    };

    format!("{}{}", CONFIG.misc.svg_icons_directory, icon_name)
}

fn update_forecast_context(context: &mut Context) -> Result<(), Error> {
    println!("Daily forecast");
    match update_daily_forecast_data(context) {
        Ok(context) => context,
        Err(e) => {
            eprintln!("Failed to update daily forecast");
            return Err(e);
        }
    };
    println!("Hourly forecast");
    match update_hourly_forecast_data(context) {
        Ok(context) => context,
        Err(e) => {
            eprintln!("Failed to update hourly forecast");
            return Err(e);
        }
    };
    Ok(())
}

fn render_dashboard_template(context: &mut Context, dashboard_svg: String) -> Result<(), Error> {
    let mut tt = TinyTemplate::new();
    let tt_name = "dashboard";

    if let Err(e) = tt.add_template(tt_name, &dashboard_svg) {
        println!("Failed to add template: {}", e);
        return Err(e.into());
    }
    tt.set_default_formatter(&format_unescaped);
    // Attempt to render the template
    match tt.render(tt_name, &context) {
        Ok(rendered) => {
            let mut output = fs::File::create(CONFIG.misc.modified_template_name.clone())?;
            output.write_all(rendered.as_bytes())?;
            Ok(())
        }
        Err(e) => {
            println!("Failed to render template: {}", e);
            Err(e.into())
        }
    }
}

pub fn generate_weather_dashboard() -> Result<(), Error> {
    let current_dir = std::env::current_dir()?;
    let template_path = current_dir.join(&CONFIG.misc.template_path);
    let mut context = Context::default();

    let template_svg = match fs::read_to_string(&template_path) {
        Ok(svg) => svg,
        Err(e) => {
            println!("Current directory: {:?}", current_dir);
            println!("Template path: {:?}", &template_path);
            println!("Failed to read template file: {}", e);
            return Err(e.into());
        }
    };
    update_forecast_context(&mut context)?;

    render_dashboard_template(&mut context, template_svg)?;
    println!(
        "SVG has been modified and saved successfully at {}",
        current_dir
            .join(&CONFIG.misc.modified_template_name)
            .display()
    );

    if !CONFIG.debugging.disable_png_output {
        convert_svg_to_png(
            &CONFIG.misc.modified_template_name,
            &CONFIG.misc.modified_template_name.replace(".svg", ".png"),
            2.0,
        )?;

        println!(
            "PNG has been generated successfully at {}",
            current_dir
                .join(CONFIG.misc.modified_template_name.replace(".svg", ".png"))
                .display()
        );
    }
    Ok(())
}

pub fn run_weather_dashboard() -> Result<(), anyhow::Error> {
    generate_weather_dashboard()?;
    if !CONFIG.debugging.disable_png_output && !CONFIG.debugging.disable_drawing_on_epd {
        invoke_pimironi_image_script()?;
    }
    if CONFIG.release.auto_update {
        update_app()?;
    };
    Ok(())
}

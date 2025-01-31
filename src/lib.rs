// mod apis;
mod bom;
mod chart;
mod config;
mod context;
mod utils;

// #[cfg(debug_assertions)]
// mod dev;

// #[cfg(debug_assertions)]
// use dev::create_striped_png;

use ::config::{Config, File};
use anyhow::Error;
use bom::*;
use chart::{catmull_bezier, Point};
use chrono::{Datelike, Timelike};
use config::DashboardConfig;
use context::Context;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::io::Write;
use std::{fs, path::PathBuf};
use strum_macros::Display;
use tinytemplate::{format_unescaped, TinyTemplate};
pub use utils::*;

pub const NOT_AVAILABLE_ICON: &str = "not-available.svg";
const CONFIG_NAME: &str = "config.toml";

lazy_static! {
    pub static ref CONFIG: DashboardConfig =
        load_dashboard_config().expect("Failed to load config");
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
    pub height: usize,
    pub width: usize,
    pub starting_x: f64,
    pub ending_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

impl DailyForecastGraph {
    const HEIGHT: usize = 300;
    const WIDTH: usize = 600;

    fn default() -> Self {
        Self {
            name: "Hourly Forecast".to_string(),
            data: vec![],
            height: Self::HEIGHT,
            width: Self::WIDTH,
            starting_x: 0.0,
            ending_x: 23.0,
            min_y: f64::INFINITY,
            max_y: -f64::INFINITY,
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

    pub fn to_color(self) -> &'static str {
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

impl DailyForecastGraph {
    fn create_axis_with_labels(
        &self,
        current_hour: f64,
    ) -> (String, String, String, String, String, String, String) {
        let width = self.width as f64;
        let height = self.height as f64;

        let range_x = self.ending_x - self.starting_x + 1.0; // +1 because last hour is 23
        let range_y_left = self.max_y - self.min_y;
        let range_y_right = 100.0; // Rain data is in percentage

        // Mapping functions from data space to SVG space
        // x data domain maps to [0, width]
        // y data domain maps to [height, 0] (SVG y goes down)
        let map_x = |x: f64| (x - self.starting_x) * (width / range_x);
        let map_y_left = |y: f64| height - ((y - self.min_y) * (height / range_y_left));
        // For the right axis, we assume 0 to 100% maps directly onto the height.
        let map_y_right = |y: f64| height - (y * (height / range_y_right));

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
        let y_right_axis_x = width;

        // Axis paths
        let mut x_axis_path = format!("M 0 {} L {} {}", x_axis_y, width, x_axis_y);
        let mut x_axis_guideline_path = format!("M 0 {} L {} {}", x_axis_y, width, x_axis_y);
        let mut y_left_axis_path = format!("M {} 0 L {} {}", y_axis_x, y_axis_x, height);
        let mut y_right_axis_path =
            format!("M {} 0 L {} {}", y_right_axis_x, y_right_axis_x, height);

        // Number of ticks, +1 because of the fencepost problem
        let x_ticks = 6;
        let y_left_ticks = 5;
        let y_right_ticks = 5;

        let x_step = range_x / x_ticks as f64;
        let y_left_step = range_y_left / y_left_ticks as f64;
        let y_right_step = range_y_right / y_right_ticks as f64;

        println!(
            "X step: {}, Y step (left): {}, Y step (right): {}",
            x_step, y_left_step, y_right_step
        );

        // Labels storage
        let mut x_labels = String::new();
        let mut y_left_labels = String::new();
        let mut y_right_labels = String::new();

        let mut x_val: f64 = 0.0;
        // X-axis ticks and labels
        for i in 0..=x_ticks {
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

            let x_guideline_len = if CONFIG.render_options.x_axis_always_at_min {
                height
            } else {
                x_axis_y
            };
            x_axis_guideline_path.push_str(&format!(
                r#" M {} {} v -{} m 0 2 v -2"#,
                xs, x_axis_y, x_guideline_len
            ));

            // Label: placed below the x-axis line
            let label_y = x_axis_y + 20.0;
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
            // slight offset for the first label if the min_y is negative
            let x_offset = if !CONFIG.render_options.x_axis_always_at_min && self.min_y < 0.0 {
                if i == 0 {
                    22.0
                } else if i == x_ticks {
                    -22.0
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let label_x = xs + x_offset;
            x_labels.push_str(&format!(
                r#"<text x="{x}" y="{y}" fill="{colour}" font-size="17" text-anchor="middle">{text}</text>"#,
                x = label_x,
                y = label_y,
                colour = CONFIG.colours.text_colour,
                text = label_str
            ));
        }

        // Y-axis ticks and labels (left)
        for j in 0..=y_left_ticks {
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
            let label_str = format!("{:.1}", y_val);
            let font_size = if j == 0 || j == y_left_ticks { 20 } else { 17 };
            y_left_labels.push_str(&format!(
                r#"<text x="{x}" y="{y}"  fill="{colour}" font-size="{font_size}" text-anchor="end" dy="4">{text}</text>"#,
                x = label_x,
                y = ys,
                colour = CONFIG.colours.text_colour,
                font_size = font_size,
                text = label_str
            ));
        }

        // Y-axis ticks and labels (right - 0 to 100%)
        for k in 0..=y_right_ticks {
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
                r#"<text x="{x}" y="{y}" fill="{colour}"  font-size="17" text-anchor="start" dy="4">{text}</text>"#,
                x = label_x,
                y = ys,
                colour = CONFIG.colours.text_colour,
                text = label_str
            ));
        }

        // Return all axis paths and labels, now including the right axis
        (
            x_axis_path,
            y_left_axis_path,
            x_labels,
            y_left_labels,
            y_right_axis_path,
            y_right_labels,
            x_axis_guideline_path,
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

        println!(
            "starting x: {}, ending x: {}",
            self.starting_x, self.ending_x
        );
        println!("Global Min y: {}, Max y: {}", self.min_y, self.max_y);
    }

    fn draw_uv_gradient_over_time(&self, uv_data: [usize; 24]) -> String {
        println!("UV data: {:?}", uv_data);
        let mut gradient = String::new();

        for (i, &uv) in uv_data.iter().enumerate() {
            let offset = (i as f64 / 23.0) * 100.0;
            let color = UVIndexCategory::from_u8(uv as u8).to_color();
            gradient.push_str(&format!(
                r#"<stop offset="{:.2}%" stop-color="{}"/>"#,
                offset, color
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
            let xfactor = self.width as f64 / self.ending_x;
            let yfactor = match data.graph_type {
                DataType::Rain => self.height as f64 / 100.0, // Rain data is in percentage
                DataType::Temp | DataType::TempFeelLike => {
                    if self.max_y >= 0.0 && self.min_y < 0.0 {
                        self.height as f64 / (self.max_y + self.min_y.abs())
                    } else if self.min_y < 0.0 {
                        // it's possible for both to be negative
                        self.height as f64 / (self.max_y.abs() - self.min_y.abs())
                    } else {
                        // when both are positive
                        self.height as f64 / (self.max_y - self.min_y)
                    }
                }
            };

            println!("X factor: {}, Y factor: {}", xfactor, yfactor);

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
                catmull_bezier(points)
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
                    let bounding_area_path =
                        format!("{} L {} 0 L 0 0Z", path, DailyForecastGraph::WIDTH);
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

trait Icon {
    fn get_icon_name(&self) -> String;
    fn get_icon_path(&self) -> String {
        format!(
            "{}{}",
            CONFIG.misc.svg_icons_directory,
            self.get_icon_name()
        )
    }
    fn rain_amount_to_name(amount: u32) -> String {
        match amount {
            0..=2 => "".to_string(),
            3..=20 => RainAmountName::Drizzle.to_string(),
            21.. => RainAmountName::Rain.to_string(),
        }
    }

    fn rain_chance_to_name(chance: u32) -> String {
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
                    .round() as u32
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
            HourlyForecast::rain_amount_to_name(self.rain.amount.min.unwrap_or(0.0).round() as u32)
        );
        temp
    }
}

fn fetch_data<T: for<'de> Deserialize<'de>>(
    endpoint: &str,
    file_path: &PathBuf,
) -> Result<T, Error> {
    if !file_path.exists() {
        fs::create_dir_all(file_path.parent().unwrap())?;
    }

    if CONFIG.debugging.use_online_data {
        let client = reqwest::blocking::Client::new();
        let response = client.get(endpoint).send()?;
        let body = response.text().map_err(Error::msg)?;

        if let Ok(api_error) = serde_json::from_str::<BomError>(&body) {
            for error in api_error.errors {
                eprintln!("API Error: {}", error.detail);
            }
            return Err(Error::msg(
                "API request failed, double check your api configs.",
            ));
        }

        fs::write(file_path, &body)?;
        serde_json::from_str(&body).map_err(Error::msg)
    } else {
        let body = fs::read_to_string(file_path)?;
        serde_json::from_str(&body).map_err(Error::msg)
    }
}

fn fetch_daily_forecast_data() -> Result<DailyForcastResponse, Error> {
    let file_path =
        std::path::Path::new(&CONFIG.misc.weather_data_store_path).join("daily_forecast.json");
    fetch_data(&DAILY_FORECAST_ENDPOINT, &file_path)
}

fn fetch_hourly_forecast_data() -> Result<HourlyForcastResponse, Error> {
    let file_path =
        std::path::Path::new(&CONFIG.misc.weather_data_store_path).join("hourly_forecast.json");
    fetch_data(&HOURLY_FORECAST_ENDPOINT, &file_path)
}

fn update_current_hour_data(current_hour: &HourlyForecast, context: &mut Context) {
    let mut curret_icon = current_hour.get_icon_path();
    if CONFIG.render_options.use_moon_phase_instead_of_clear_night
        && curret_icon.ends_with(&format!("{}{}.svg", RainChanceName::Clear, DayNight::Night))
    {
        println!("Using moon phase icon instead of clear night");
        curret_icon = get_moon_phase_icon_path().to_string();
    }
    context.current_temp = current_hour.temp.to_string();
    context.current_icon = curret_icon;
    context.current_feels_like = current_hour.temp_feels_like.to_string();
    context.current_wind_speed = current_hour.wind.speed_kilometre.to_string();
    context.current_wind_icon = current_hour.wind.get_icon_path();
    context.current_uv_index = current_hour.uv.to_string();
    context.current_relative_humidity = current_hour.relative_humidity.to_string();
    context.current_relative_humidity_icon = current_hour.relative_humidity.get_icon_path();
    context.current_day_name = chrono::Local::now().format("%A, %d %b").to_string();
    context.current_rain_amount = (current_hour.rain.amount.min.unwrap_or(0.0)
        + current_hour.rain.amount.min.unwrap_or(0.0))
    .to_string();
    context.rain_measure_icon = current_hour.rain.amount.get_icon_path();
}

// Extrusion Pattern: force everything through one function until it resembles spaghetti
fn update_daily_forecast_data(context: &mut Context) -> Result<(), Error> {
    let daily_forecast_data = fetch_daily_forecast_data()?.data;
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
            "{} - min {} max {} temp",
            day_name_value, min_temp_value, max_temp_value
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
        println!("Less than 7 days of forecast data");
    }

    Ok(())
}

fn update_hourly_forecast_data(context: &mut Context) -> Result<(), Error> {
    let hourly_forecast = fetch_hourly_forecast_data()?;

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
        max_index: Some(hourly_forecast.data[0].uv as u32),
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
    let first_date = hourly_forecast
        .data
        .iter()
        .find_map(
            // find the first time
            |forecast| {
                if forecast.time >= current_date {
                    Some(forecast.time)
                } else {
                    None
                }
            },
        )
        .unwrap_or_else(|| chrono::Local::now().naive_local());

    let end_date = first_date + chrono::Duration::hours(24);

    println!("First date: {:?}", first_date);
    println!("End date: {:?}", end_date);

    let mut x = 0.0;

    let mut uv_data = [0; 24];

    hourly_forecast
        .data
        .iter()
        .filter(|forecast| forecast.time >= first_date && forecast.time < end_date)
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

    let day_start = first_date
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    let day_end = day_start + chrono::Duration::days(1);

    println!("Day start: {:?}", day_start);
    println!("Day end: {:?}", day_end);

    context.graph_height = graph.height.to_string();
    context.graph_width = graph.width.to_string();
    context.temp_curve_data = temp_curve_data;
    context.feel_like_curve_data = feel_like_curve_data;
    context.rain_curve_data = rain_curve_data;
    context.uv_index = hourly_forecast.data[0].uv.to_string();
    context.uv_index_icon = current_uv.get_icon_path().to_string();
    context.wind_speed = hourly_forecast.data[0].wind.speed_kilometre.to_string();
    context.max_wind_gust_today = find_max_item_between_dates(
        &hourly_forecast.data,
        &day_start,
        &day_end,
        |item| item.wind.gust_speed_kilometre.unwrap_or(0.0),
        |item| &item.time,
    )
    .to_string();
    // There is a discrepancy in max uv between hourly forecast and daily forecast
    context.uv_max_today = find_max_item_between_dates(
        &hourly_forecast.data,
        &day_start,
        &day_end,
        |item| item.uv,
        |item| &item.time,
    )
    .to_string();
    context.max_relative_humidity_today = find_max_item_between_dates(
        &hourly_forecast.data,
        &day_start,
        &day_end,
        |item| item.relative_humidity,
        |item| &item.time,
    )
    .to_string();

    context.total_rain_today = (get_total_between_dates(
        &hourly_forecast.data,
        &day_start,
        &day_end,
        |item| (item.rain.amount.min.unwrap_or(0.0) + item.rain.amount.max.unwrap_or(0.0)) / 2.0,
        |item| &item.time,
    ) as usize)
        .to_string();

    context.wind_icon = hourly_forecast.data[0].wind.get_icon_path();

    let axis_data_path = graph.create_axis_with_labels(first_date.hour() as f64);

    context.x_axis_path = axis_data_path.0;
    context.y_left_axis_path = axis_data_path.1;
    context.x_labels = axis_data_path.2;
    context.y_left_labels = axis_data_path.3;
    context.y_right_axis_path = axis_data_path.4;
    context.y_right_labels = axis_data_path.5;
    context.x_axis_guideline_path = axis_data_path.6;

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

fn load_dashboard_config() -> Result<DashboardConfig, Error> {
    let root = std::env::current_dir()?;
    let config_path = root.join(CONFIG_NAME);
    let settings = Config::builder()
        .add_source(File::with_name(config_path.to_str().unwrap()))
        .build()?;

    settings.try_deserialize().map_err(Error::msg)
}

pub fn update_forecast_context(context: &mut Context) -> Result<(), Error> {
    match update_daily_forecast_data(context) {
        Ok(context) => context,
        Err(e) => {
            println!("Failed to update daily forecast: {}", e);
            return Err(e);
        }
    };
    match update_hourly_forecast_data(context) {
        Ok(context) => context,
        Err(e) => {
            println!("Failed to update hourly forecast: {}", e);
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
            println!(
                "SVG has been modified and saved successfully at {}",
                CONFIG.misc.modified_template_name
            );
            Ok(())
        }
        Err(e) => {
            println!("Failed to render template: {}", e);
            Err(e.into())
        }
    }
}

pub fn generate_weather_dashboard() -> Result<(), Error> {
    // print current directory
    let current_dir = std::env::current_dir()?;
    println!("Current directory: {:?}", current_dir);

    //print current dir + template path
    let template_path = current_dir.join(&CONFIG.misc.template_path);
    println!("Template path: {:?}", template_path);

    let template_svg = match fs::read_to_string(template_path) {
        Ok(svg) => svg,
        Err(e) => {
            println!("Failed to read template file: {}", e);
            return Err(e.into());
        }
    };
    let mut context = Context::default();
    update_forecast_context(&mut context)?;
    render_dashboard_template(&mut context, template_svg)?;

    convert_svg_to_png(
        &CONFIG.misc.modified_template_name,
        &CONFIG.misc.modified_template_name.replace(".svg", ".png"),
    )
    .map_err(Error::msg)?;

    println!(
        "PNG has been generated successfully at {}",
        CONFIG.misc.modified_template_name.replace(".svg", ".png")
    );
    Ok(())
}

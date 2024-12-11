#![allow(unused_imports)]
#![allow(dead_code)]
pub mod charts;
use anyhow::Error;
use chrono::{NaiveDateTime, Timelike};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use serde_json::{Error as SerdeError, Result as SerdeResult};
use std::io::{self, Write};
use std::{env, fs};
use strum_macros::Display;

const WEATHER_PROVIDER: &str = "https://api.weather.bom.gov.au/v1/locations/";
const LOCATION: &str = "r283sf";
use lazy_static::lazy_static;

lazy_static! {
    static ref DAILY_FORECAST_ENDPOINT: String =
        format!("{}/{}/forecasts/daily", WEATHER_PROVIDER, LOCATION);
    static ref HOURLY_FORECAST_ENDPOINT: String =
        format!("{}/{}/forecasts/hourly", WEATHER_PROVIDER, LOCATION);
}

const UNIT: &str = "°C";
const TEMPLATE_PATH: &str = "./dashboard-template-min.svg";
const MODIFIED_TEMPLATE_PATH: &str = "./modified-dashboard.svg";
const ICON_PATH: &str = "./static/line-svg-static/";
const USE_ONLINE_DATA: bool = false;
const NOT_AVAILABLE_ICON: &str = "not-available.svg";
// const ICON_PATH: &str = "./static/fill-svg-static/";

#[derive(Deserialize, Clone, Debug, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn to_svg(&self) -> String {
        format!("L {} {}", self.x, self.y)
    }
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
    pub colour: String,
    pub smooth: bool,
}

pub struct DailyForcastGraph {
    pub name: String,
    pub data: Vec<GraphData>,
    pub height: usize,
    pub width: usize,
}

impl DailyForcastGraph {
    const HEIGHT: usize = 300;
    const WIDTH: usize = 600;
    // const HEIGHT: usize = 400;
    // const WIDTH: usize = 800;

    fn default() -> Self {
        Self {
            name: "Hourly Forcast".to_string(),
            data: vec![],
            height: Self::HEIGHT,
            width: Self::WIDTH,
        }
    }
}

impl GraphData {
    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push(Point { x, y })
    }
}

pub enum GraphDataPath {
    Temp(String),
    TempFeelLike(String),
    Rain(String),
}

impl DailyForcastGraph {
    pub fn draw_graph(&self) -> Result<Vec<GraphDataPath>, Error> {
        // Calculate the minimum and maximum x values from the points
        let mut data_path = vec![];
        for data in &self.data {
            let min_x = data.points.first().map(|val| val.x).unwrap_or(0.0);
            let max_x = data
                .points
                .iter()
                .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
                .unwrap()
                .x;

            // println!("{:?}", data.points);

            // Calculate the minimum and maximum y values from the points
            let min_y = data.points.iter().map(|val| val.y).fold(f64::NAN, f64::min);
            let max_y = match data.graph_type {
                DataType::Rain => 100.0,
                DataType::TempFeelLike | DataType::Temp => {
                    data.points
                        .iter()
                        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
                        .unwrap()
                        .y
                }
            };

            println!("Min x: {}, Max x: {}", min_x, max_x);
            println!("Min y: {}, Max y: {}", min_y, max_y);

            // Calculate scaling factors for x and y to fit the graph within the given width and height
            let xfactor = self.width as f64 / max_x;
            let yfactor = self.height as f64 / max_y;

            println!("X factor: {}, Y factor: {}", xfactor, yfactor);

            // Scale the points according to the calculated factors
            let points: Vec<Point> = data
                .points
                .iter()
                .map(|val| Point {
                    x: (val.x * xfactor),
                    y: (val.y * yfactor),
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
                    data_path.push(GraphDataPath::Rain(path));
                }
            }
        }
        Ok(data_path)
    }
}

#[derive(Deserialize, Debug)]
struct Wind {
    speed_kilometre: f64,
    speed_knot: f64,
    direction: String,
    gust_speed_knot: Option<f64>,
    gust_speed_kilometre: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct Gust {
    speed_kilometre: f64,
    speed_knot: f64,
    time: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Temp {
    time: String,
    value: f64,
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
        format!("{}{}", ICON_PATH, self.get_icon_name())
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
            0.0..=20.0 => "wind-beaufort-0.svg",
            20.1..=40.0 => "umbrella-wind.svg",
            40.1.. => "umbrella-wind-alt.svg",
            _ => NOT_AVAILABLE_ICON,
        };
        icon.to_string()
    }
}

#[derive(Deserialize, Debug)]
struct Metadata {
    response_timestamp: String,
    issue_time: String,
    observation_time: Option<String>,
    copyright: String,
}

#[derive(Deserialize, Debug)]
struct RainAmount {
    min: Option<f64>,
    max: Option<f64>,
    lower_range: Option<f64>,
    upper_range: Option<f64>,
    units: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Rain {
    amount: RainAmount,
    chance: Option<u32>,
    chance_of_no_rain_category: Option<String>,
    precipitation_amount_25_percent_chance: Option<f64>,
    precipitation_amount_50_percent_chance: Option<f64>,
    precipitation_amount_75_percent_chance: Option<f64>,
}
#[derive(Deserialize, Debug)]
struct UV {
    category: Option<String>,
    end_time: Option<String>,
    max_index: Option<u32>,
    start_time: Option<String>,
}

impl Icon for UV {
    fn get_icon_name(&self) -> String {
        match self.max_index {
            Some(index) => match index {
                0 => "uv-index.svg".to_string(),
                1..=11 => format!("uv-index-{}.svg", index),
                _ => NOT_AVAILABLE_ICON.to_string(),
            },
            None => NOT_AVAILABLE_ICON.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct Astronomical {
    sunrise_time: Option<String>,
    sunset_time: Option<String>,
}
#[derive(Deserialize, Debug)]
struct FireDangerCategory {
    text: Option<String>,
    default_colour: Option<String>,
    dark_mode_colour: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Now {
    is_night: Option<bool>,
    now_label: Option<String>,
    later_label: Option<String>,
    temp_now: Option<f64>,
    temp_later: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct DailyEntry {
    rain: Option<Rain>,
    uv: Option<UV>,
    astronomical: Option<Astronomical>,
    date: Option<String>,
    temp_max: Option<f64>,
    temp_min: Option<f64>,
    extended_text: Option<String>,
    icon_descriptor: Option<String>,
    short_text: Option<String>,
    surf_danger: Option<String>,
    fire_danger: Option<String>,
    fire_danger_category: Option<FireDangerCategory>,
    now: Option<Now>,
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
        println!("Temp: {:?}", temp);
        temp
    }
}

#[derive(Deserialize, Debug)]
struct HourlyMetadata {
    response_timestamp: String,
    issue_time: String,
    next_issue_time: String,
    forecast_region: String,
    forecast_type: String,
    copyright: String,
}

#[derive(Deserialize, Debug)]
struct HourlyForecast {
    rain: Rain,
    temp: f64,
    temp_feels_like: f64,
    dew_point: f64,
    wind: Wind,
    relative_humidity: f64,
    uv: f64,
    icon_descriptor: String,
    next_three_hourly_forecast_period: String,
    time: String,
    is_night: bool,
    next_forecast_period: String,
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
        println!("Temp: {:?}", temp);
        temp
    }
}

#[derive(Deserialize, Debug)]
struct HourlyForcastResponse {
    metadata: Metadata,
    data: Vec<HourlyForecast>,
}

#[derive(Deserialize, Debug)]
struct DailyForcastResponse {
    metadata: HourlyMetadata,
    data: Vec<DailyEntry>,
}

fn fetch<T: for<'de> Deserialize<'de>>(endpoint: &str, file_path: &str) -> SerdeResult<T> {
    if USE_ONLINE_DATA {
        let client = reqwest::blocking::Client::new();
        let response = client.get(endpoint).send();
        let body = response.unwrap().text();
        // write them to a file
        let file = fs::File::create(file_path);
        if let Err(e) = file.unwrap().write_all(body.unwrap().as_bytes()) {
            eprintln!("Failed to write to file: {}", e);
        }
    }

    let body = fs::read_to_string(file_path);
    serde_json::from_str(&body.unwrap())
}

fn fetch_daily_forecast() -> SerdeResult<DailyForcastResponse> {
    fetch(&DAILY_FORECAST_ENDPOINT, "./test/daily_forcast.json")
}

fn fetch_hourly_forecast() -> SerdeResult<HourlyForcastResponse> {
    fetch(&HOURLY_FORECAST_ENDPOINT, "./test/hourly_forcast.json")
}

fn update_current_hour(current_hour: &HourlyForecast, template: String) -> String {
    template
        .replace("{{current_temp}}", &current_hour.temp.to_string())
        .replace("{{current_icon}}", &current_hour.get_icon_path())
        .replace(
            "{{current_feels_like}}",
            &current_hour.temp_feels_like.to_string(),
        )
        .replace(
            "{{current_wind_speed}}",
            &current_hour.wind.speed_kilometre.to_string(),
        )
        .replace("{{current_wind_icon}}", &current_hour.wind.get_icon_path())
        .replace("{{current_uv_index}}", &current_hour.uv.to_string())
        .replace(
            "{{current_relative_humidity}}",
            &current_hour.relative_humidity.to_string(),
        )
        .replace(
            "{{current_relative_humidity_icon}}",
            "./static/line-svg-static/humidity.svg",
        )
        .replace(
            "{{day1_name}}",
            &chrono::Local::now().format("%A, %d %b").to_string(),
        )
        .replace(
            "{{time}}",
            &chrono::Local::now().format("%I:%M%p").to_string(),
        )
}

fn update_daily_forecast(template: String) -> Result<String, Error> {
    let daily_forecast_data = fetch_daily_forecast()?.data;
    // todo check length of daily_forecast_data
    let mut updated_template = template.to_string();
    let mut i = 2;
    for day in daily_forecast_data {
        if let Some(date_str) = &day.date {
            if let Ok(date) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%SZ")
                .map(|datetime| datetime.date())
            {
                let current_date = chrono::Local::now().date_naive();

                // If the date is today or in the past, skip it
                if date <= current_date {
                    continue;
                }
            }
        } else if day.date
            == Some(
                (chrono::Local::now() + chrono::Duration::days(7))
                    .format("%Y-%m-%d")
                    .to_string(),
            )
        {
            // If the date is more than 7 days in the future, skip it
            break;
        }

        let min_temp_key = format!("{{{{day{}_mintemp}}}}", i);
        let max_temp_key = format!("{{{{day{}_maxtemp}}}}", i);
        let icon_key = format!("{{{{day{}_icon}}}}", i);
        let day_name_key = format!("{{{{day{}_name}}}}", i);

        // println!("Day: {:?}", day);

        let min_temp_value = day
            .temp_min
            .map(|temp| temp.to_string())
            .unwrap_or_else(|| "NA".to_string());

        let max_temp_value = day
            .temp_max
            .map(|temp| temp.to_string())
            .unwrap_or_else(|| "NA".to_string());

        let icon_value = day.get_icon_path();

        println!("min {} max {} temp", min_temp_value, max_temp_value);

        let day_name_value = day
            .date
            .as_ref()
            .map(|date| {
                chrono::NaiveDate::parse_from_str(date, "%Y-%m-%dT%H:%M:%SZ")
                    .map(|parsed_date| parsed_date.format("%A").to_string())
                    .map(|day_name| day_name.chars().take(3).collect::<String>())
                    .unwrap_or_else(|_| "NA".to_string())
            })
            .unwrap_or_else(|| "NA".to_string());

        updated_template = updated_template
            .replace(&min_temp_key, &min_temp_value)
            .replace(&max_temp_key, &max_temp_value)
            .replace(&icon_key, &icon_value)
            .replace(&day_name_key, &day_name_value);
        i += 1;
    }

    // if i < 8, this means that we have less than 7 days of forecast data
    // so we need to NA the remaining days
    while i < 8 {
        let min_temp_key = format!("{{{{day{}_mintemp}}}}", i);
        let max_temp_key = format!("{{{{day{}_maxtemp}}}}", i);
        let icon_key = format!("{{{{day{}_icon}}}}", i);
        let day_name_key = format!("{{{{day{}_name}}}}", i);

        updated_template = updated_template
            .replace(&min_temp_key, "NA")
            .replace(&max_temp_key, "NA")
            .replace(&icon_key, &format!("{}{}", ICON_PATH, NOT_AVAILABLE_ICON))
            .replace(&day_name_key, "NA");

        i += 1;
    }

    Ok(updated_template)
}

fn update_hourly_forecast(template: String) -> Result<String, Error> {
    let mut hourly_forecast = fetch_hourly_forecast()?;

    let mut graph = DailyForcastGraph::default();

    let mut temp_data = GraphData {
        graph_type: DataType::Temp,
        points: vec![],
        colour: "blue".to_string(),
        smooth: true,
    };

    let mut feels_like_data = GraphData {
        graph_type: DataType::TempFeelLike,
        points: vec![],
        colour: "green".to_string(),
        smooth: true,
    };

    let mut rain_data = GraphData {
        graph_type: DataType::Rain,
        points: vec![],
        colour: "red".to_string(),
        smooth: false,
    };

    let mut updated_template = template
        .replace("{{temp_colour}}", &temp_data.colour)
        .replace("{{feels_like_colour}}", &feels_like_data.colour)
        .replace("{{rain_colour}}", &rain_data.colour);

    hourly_forecast.data.sort_by(|a, b| a.time.cmp(&b.time));

    updated_template = update_current_hour(&hourly_forecast.data[0], updated_template);

    // we only want to display the next 24 hours
    let first_date =
        NaiveDateTime::parse_from_str(&hourly_forecast.data[0].time, "%Y-%m-%dT%H:%M:%SZ")
            .unwrap_or_else(|_| chrono::Local::now().naive_local());

    let end_date = first_date + chrono::Duration::hours(24);

    println!("First date: {:?}", first_date);
    println!("End date: {:?}", end_date);

    let mut x = 0.0;

    hourly_forecast
        .data
        .iter()
        .filter(|forcast| {
            match NaiveDateTime::parse_from_str(&forcast.time, "%Y-%m-%dT%H:%M:%SZ") {
                Ok(datetime) => datetime < end_date,
                Err(_) => false,
            }
        })
        .for_each(|forcast| {
            // println!("{:?} <= {:?}", forcast.time, forcast.temp);
            temp_data.add_point(x, forcast.temp);
            feels_like_data.add_point(x, forcast.temp_feels_like);
            rain_data.add_point(x, forcast.rain.chance.unwrap_or(0).into());
            x += 1.0;
        });

    graph
        .data
        .extend(vec![temp_data, rain_data, feels_like_data]);

    let svg_result = graph.draw_graph().unwrap();

    let current_uv = UV {
        category: None,
        end_time: None,
        max_index: Some(hourly_forecast.data[0].uv as u32),
        start_time: None,
    };

    // println!("\n{:?}\n", svg_result);
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

    updated_template = updated_template
        .replace("{{graph_hieght}}", &graph.height.to_string())
        .replace("{{graph_width}}", &graph.width.to_string())
        .replace("{{temp_curve_data}}", &temp_curve_data)
        .replace("{{feel_like_curve_data}}", &feel_like_curve_data)
        .replace("{{rain_curve_data}}", &rain_curve_data)
        .replace("{{uv_index}}", &hourly_forecast.data[0].uv.to_string())
        .replace("{{uv_index_icon}}", &current_uv.get_icon_path().to_string())
        .replace(
            "{{relative_humidity}}",
            &hourly_forecast.data[0].relative_humidity.to_string(),
        )
        .replace(
            "{{relative_humidity_icon}}",
            "./static/line-svg-static/humidity.svg",
        )
        .replace(
            "{{wind_speed}}",
            &hourly_forecast.data[0].wind.speed_kilometre.to_string(),
        )
        .replace(
            "{{wind_icon}}",
            &hourly_forecast.data[0].wind.get_icon_path(),
        );

    Ok(updated_template)
}
pub fn generate_weather_dashboard() -> io::Result<()> {
    let dashboard_svg = fs::read_to_string(TEMPLATE_PATH)?;
    let mut updated_svg = update_daily_forecast(dashboard_svg);
    updated_svg = update_hourly_forecast(updated_svg.unwrap());

    let mut output = fs::File::create(MODIFIED_TEMPLATE_PATH)?;
    let unwraped_svg: String = updated_svg.unwrap();
    output.write_all(unwraped_svg.as_bytes())?;

    println!(
        "SVG has been modified and saved successfully at {}",
        MODIFIED_TEMPLATE_PATH
    );
    Ok(())
}

#[derive(Deserialize, Clone, Debug)]
struct Curve {
    c1: Point,
    c2: Point,
    end: Point,
}

impl Curve {
    fn to_svg(&self) -> String {
        format!(
            "C {:.4} {:.4}, {:.4} {:.4}, {:.4} {:.4}",
            self.c1.x, self.c1.y, self.c2.x, self.c2.y, self.end.x, self.end.y
        )
    }
}

fn catmull_bezier(points: Vec<Point>) -> Vec<Curve> {
    let mut res = Vec::new();

    let last = points.len() - 1;

    for i in 0..last {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };

        let p1 = points[i];

        let p2 = points[i + 1];

        let p3 = if i + 2 > last {
            points[i + 1]
        } else {
            points[i + 2]
        };

        let c1 = Point {
            x: ((-p0.x + 6.0 * p1.x + p2.x) / 6.0),
            y: ((-p0.y + 6.0 * p1.y + p2.y) / 6.0),
        };

        let c2 = Point {
            x: ((p1.x + 6.0 * p2.x - p3.x) / 6.0),
            y: ((p1.y + 6.0 * p2.y - p3.y) / 6.0),
        };

        let end = p2;

        res.push(Curve { c1, c2, end });
    }

    res
}

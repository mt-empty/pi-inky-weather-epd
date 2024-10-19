pub mod charts;
use anyhow::Error;
use chrono::NaiveDateTime;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Error as SerdeError, Result as SerdeResult};
use std::io::{self, Write};
use std::{env, fs};

const WEATHER_PROVIDER: &str = "https://api.weather.bom.gov.au/v1/locations/";
const LOCATION: &str = "r1r0z4";
use lazy_static::lazy_static;

lazy_static! {
    static ref OBSERVATION_ENDOPINT: String =
        format!("{}/{}/observations", WEATHER_PROVIDER, LOCATION);
    static ref DAILY_FORECAST_ENDPOINT: String =
        format!("{}/{}/forecasts/daily", WEATHER_PROVIDER, LOCATION);
    static ref HOURLY_FORECAST_ENDPOINT: String =
        format!("{}/{}/forecasts/hourly", WEATHER_PROVIDER, LOCATION);
}
const UNIT: &str = "°C";
const TEMPLATE_PATH: &str = "./dashboard-template-min.svg";
const MODIFIED_TEMPLATE_PATH: &str = "./modified-dashboard.svg";
const ICON_PATH: &str = "./static/line-svg-static/";
// const ICON_PATH: &str = "./static/fill-svg-static/";

#[derive(Deserialize, Debug)]
struct Wind {
    speed_kilometre: f64,
    speed_knot: f64,
    direction: String,
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

trait Value {
    fn to_string(&self) -> f64;
}

impl Value for Temp {
    fn to_string(&self) -> f64 {
        self.value
    }
}

impl Value for Wind {
    fn to_string(&self) -> f64 {
        self.speed_kilometre
    }
}

impl Value for Gust {
    fn to_string(&self) -> f64 {
        self.speed_kilometre
    }
}

trait Icon {
    fn get_icon_name(&self) -> &str;
    fn get_icon_path(&self) -> String {
        format!("{}{}", ICON_PATH, self.get_icon_name())
    }
}

impl Icon for Temp {
    fn get_icon_name(&self) -> &str {
        match self.value {
            0.0..=10.0 => "thermometer-celsius.svg",
            10.1..=20.0 => "thermometer-colder.svg",
            20.1..=30.0 => "thermometer-fahrenheit.svg",
            30.1..=40.0 => "thermometer-glass-celsius.svg",
            40.1..=50.0 => "thermometer-glass-fahrenheit.svg",
            50.1..=60.0 => "thermometer-glass.svg",
            60.1..=70.0 => "thermometer-mercury-cold.svg",
            70.1..=80.0 => "thermometer-mercury.svg",
            80.1..=90.0 => "thermometer-moon.svg",
            90.1..=100.0 => "thermometer-raindrop.svg",
            _ => "not-available.svg",
        }
    }
}

impl Icon for Wind {
    fn get_icon_name(&self) -> &str {
        match self.speed_kilometre {
            0.0..=10.0 => "wind-beaufort-0.svg",
            10.1..=20.0 => "wind-beaufort-1.svg",
            20.1..=30.0 => "wind-beaufort-2.svg",
            30.1..=40.0 => "wind-beaufort-3.svg",
            40.1..=50.0 => "wind-beaufort-4.svg",
            50.1..=60.0 => "wind-beaufort-5.svg",
            60.1..=70.0 => "wind-beaufort-6.svg",
            70.1..=80.0 => "wind-beaufort-7.svg",
            80.1..=90.0 => "wind-beaufort-8.svg",
            90.1..=100.0 => "wind-beaufort-9.svg",
            100.1..=110.0 => "wind-beaufort-10.svg",
            110.1..=120.0 => "wind-beaufort-11.svg",
            120.1..=130.0 => "wind-beaufort-12.svg",
            _ => "not-available.svg",
        }
    }
}

impl Icon for Gust {
    fn get_icon_name(&self) -> &str {
        match self.speed_kilometre {
            0.0..=10.0 => "windsock-weak.svg",
            10.1..=20.0 => "windsock.svg",
            20.1..=30.0 => "wind-onshore.svg",
            30.1..=40.0 => "wind-offshore.svg",
            40.1..=50.0 => "wind-snow.svg",
            50.1..=60.0 => "wind-alert.svg",
            _ => "not-available.svg",
        }
    }
}

#[derive(Deserialize, Debug)]
struct Station {
    bom_id: String,
    name: String,
    distance: u32,
}

#[derive(Deserialize, Debug)]
struct ObservationData {
    temp: f64,
    temp_feels_like: f64,
    wind: Wind,
    gust: Gust,
    max_gust: Gust,
    max_temp: Temp,
    min_temp: Temp,
    rain_since_9am: f64,
    humidity: u32,
    station: Station,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    response_timestamp: String,
    issue_time: String,
    observation_time: String,
    copyright: String,
}

#[derive(Deserialize, Debug)]
struct ObservationResponse {
    metadata: Metadata,
    data: ObservationData,
}

// fn get_icon_for_element<T>(
//     element: T,
//     value_range_to_icon: Vec<(f64, f64, &str)>,
//     compare_fn: fn(T) -> bool,
// ) -> &str {
//     for (min, max, icon) in value_range_to_icon {
//         if compare_fn(element) {
//             return icon;
//         }
//     }
//     "not-available.svg"
// }

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
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
pub struct ForcastGraph {
    pub name: String,
    pub points: Vec<Point>,
    pub colour: String,
    pub smooth: bool,
}

impl ForcastGraph {
    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push(Point { x, y })
    }
    pub fn draw_graph(&self, width: usize, height: usize) -> Result<String, Error> {
        // if self.points.len() == 0 {
        //     todo!()
        // }

        let min_x = self.points.first().map(|val| val.x).unwrap_or(0.0);
        let max_x = self
            .points
            .iter()
            .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
            .unwrap()
            .x;
        let min_y = self.points.iter().map(|val| val.y).fold(f64::NAN, f64::min);
        let max_y = self
            .points
            .iter()
            .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
            .unwrap()
            .y;

        let xfactor = width as f64 / max_x;
        let yfactor = height as f64 / max_y;

        let points: Vec<Point> = self
            .points
            .iter()
            .map(|val| Point {
                x: (val.x * xfactor),
                y: (val.y * yfactor),
            })
            .collect();

        let path = if self.smooth {
            catmull_bezier(points)
                .iter()
                .map(|val| val.to_svg())
                .collect::<Vec<String>>()
                .join("")
        } else {
            points
                .iter()
                .map(|val| val.to_svg())
                .collect::<Vec<String>>()
                .join("")
        };
        Ok(path)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RainAmount {
    min: Option<f64>,
    max: Option<f64>,
    lower_range: Option<f64>,
    upper_range: Option<f64>,
    units: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rain {
    amount: RainAmount,
    chance: Option<u32>,
    chance_of_no_rain_category: Option<String>,
    precipitation_amount_25_percent_chance: Option<f64>,
    precipitation_amount_50_percent_chance: Option<f64>,
    precipitation_amount_75_percent_chance: Option<f64>,
}
#[derive(Serialize, Deserialize, Debug)]
struct UV {
    category: Option<String>,
    end_time: Option<String>,
    max_index: Option<u32>,
    start_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Astronomical {
    sunrise_time: Option<String>,
    sunset_time: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct FireDangerCategory {
    text: Option<String>,
    default_colour: Option<String>,
    dark_mode_colour: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Now {
    is_night: Option<bool>,
    now_label: Option<String>,
    later_label: Option<String>,
    temp_now: Option<f64>,
    temp_later: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
struct Metadata2 {
    response_timestamp: String,
    issue_time: String,
    next_issue_time: String,
    forecast_region: String,
    forecast_type: String,
    copyright: String,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    metadata: Metadata2,
    data: Vec<DailyEntry>,
}

fn fetch_observation() -> SerdeResult<ObservationResponse> {
    // let client = reqwest::blocking::Client::new();
    // let response = client.get(&*OBSERVATION_ENDOPINT).send();
    // let body = response.unwrap().text();
    // print!("{:?}", body);
    // serde_json::from_str(&body.unwrap())

    let body = fs::read_to_string("./observation.json");
    serde_json::from_str(&body.unwrap())
}

fn fetch_daily_forecast() -> SerdeResult<WeatherResponse> {
    // let client = reqwest::blocking::Client::new();
    // let response = client.get(&*DAILY_FORECAST_ENDPOINT).send();
    // let body = response.unwrap().text();
    // // write them to a file
    // let mut file = fs::File::create(
    //     "./daily_forcast.json",
    // );
    // file.unwrap().write_all(body.unwrap().as_bytes());

    let body = fs::read_to_string("./daily_forcast.json");
    // print!("{:?}", body);
    serde_json::from_str(&body.unwrap())
}

// fn fetch_hourly_forecast() -> SerdeResult< WeatherData , Error> {
//     let client = reqwest::blocking::Client::new();
//     let response = client.get(&*HOURLY_FORECAST_ENDPOINT).send()?;
//     let body = response.text()?;
//     println!("{:?}", body);
//     Ok(())
// }day2_maxtemp

fn update_observation(template: String) -> Result<String, Error> {
    let observation = fetch_observation()?;
    let observation_data = observation.data;
    let issue_date = observation.metadata.issue_time;
    let temp: Temp = Temp {
        time: "now".to_string(),
        value: observation_data.temp,
    };
    let feels_like = observation_data.temp_feels_like;
    let wind = Wind {
        speed_kilometre: observation_data.wind.speed_kilometre,
        speed_knot: observation_data.wind.speed_knot,
        direction: observation_data.wind.direction,
    };
    let gust = Gust {
        speed_kilometre: observation_data.gust.speed_kilometre,
        speed_knot: observation_data.gust.speed_knot,
        time: observation_data.gust.time,
    };

    // find and replace {{keyword}} with the value
    let updated_template = template
        .replace("{{day1_temp}}", &temp.value.to_string())
        .replace("{{day1_icon}}", &temp.get_icon_path())
        .replace("{{day1_feels_like}}", &feels_like.to_string())
        .replace(
            "{{day1_maxtemp}}",
            &observation_data.max_temp.value.to_string(),
        )
        .replace(
            "{{day1_mintemp}}",
            &observation_data.min_temp.value.to_string(),
        )
        .replace("{{wind_speed}}", &wind.speed_kilometre.to_string())
        .replace("{{wind_icon}}", &wind.get_icon_path())
        .replace("{{gust_speed}}", &gust.speed_kilometre.to_string())
        .replace("{{gust_icon}}", &gust.get_icon_path())
        .replace("{{max_temp}}", &observation_data.max_temp.value.to_string())
        .replace("{{min_temp}}", &observation_data.min_temp.value.to_string())
        .replace(
            "{{day1_name}}",
            &chrono::Local::now().format("%A, %d %b").to_string(),
        );

    Ok(updated_template)
}
fn update_forecast(template: String) -> Result<String, Error> {
    let daily_forecast_data = fetch_daily_forecast()?.data;
    // todo check length of daily_forecast_data
    let mut updated_template = template.to_string();
    for i in 1..7 {
        let min_temp_key = format!("{{{{day{}_mintemp}}}}", i + 1);
        let max_temp_key = format!("{{{{day{}_maxtemp}}}}", i + 1);
        let icon_key = format!("{{{{day{}_icon}}}}", i + 1);
        let day_name_key = format!("{{{{day{}_name}}}}", i + 1);

        let min_temp_value = daily_forecast_data[i]
            .temp_min
            .map(|temp| temp.to_string())
            .unwrap_or_else(|| "NA".to_string());

        let max_temp_value = daily_forecast_data[i]
            .temp_max
            .map(|temp| temp.to_string())
            .unwrap_or_else(|| "NA".to_string());

        let icon_value = daily_forecast_data[i]
            .temp_max
            .map(|temp| {
                Temp {
                    value: temp,
                    time: "Now".to_string(),
                }
                .get_icon_path()
            })
            .unwrap_or_else(|| "NA".to_string());

        let day_name_value = daily_forecast_data[i]
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
    }

    Ok(updated_template)
}

fn update_hourly_forecast(template: String) -> Result<String, Error> {
    // let hourly_forecast = fetch_hourly_forecast()?;
    Ok(template)
}
fn main() -> io::Result<()> {
    let mut graph = ForcastGraph {
        name: "forcast".to_string(),
        points: vec![],
        colour: "red".to_string(),
        smooth: true,
    };

    graph.add_point(1.0, 11.0);
    graph.add_point(2.0, 13.0);
    graph.add_point(3.0, 12.5);
    graph.add_point(4.0, 16.0);
    graph.add_point(5.0, 23.0);
    graph.add_point(6.0, 21.0);
    graph.add_point(7.0, 20.0);
    graph.add_point(8.0, 19.0);
    graph.add_point(9.0, 18.0);
    graph.add_point(10.0, 17.0);

    let svg_result = graph.draw_graph(600, 300).unwrap();

    println!("\n{:?}\n", svg_result);

    let dashboard_svg = fs::read_to_string(TEMPLATE_PATH)?;
    let mut updated_svg = update_observation(dashboard_svg);
    updated_svg = update_forecast(updated_svg.unwrap());

    let mut output = fs::File::create(MODIFIED_TEMPLATE_PATH)?;
    let unwraped_svg: String = updated_svg.unwrap();
    output.write_all(unwraped_svg.as_bytes())?;

    println!(
        "SVG has been modified and saved successfully at {}",
        MODIFIED_TEMPLATE_PATH
    );
    Ok(())
}
// <?xml version="1.0" encoding="UTF-8"?>
fn strip_xml_tag(svg_icon: &str) -> String {
    let re = Regex::new(r"<\?xml.*?>").unwrap();
    re.replace_all(svg_icon, "").to_string()
}

// this function takes a sring stripped of new lines representing an svg template, an svg element, and
// a label inside the element. it embed the svg element under the label element
fn embed_svg_element<'a>(svg_template: &'a str, svg_element: &'a str, label: &'a str) -> String {
    // find element that contain the label
    println!("Label: {}", label);
    println!("SVG element: {}", svg_element);
    let mut new_svg = String::new();
    match svg_template.find(format!("label=\"{}\" />", label).as_str()) {
        Some(mut index) => {
            index += (format!("label=\"{}\" />", label).as_str()).len();
            println!("Label found in the SVG template. {} ", index);
            new_svg.push_str(&svg_template[..index]);
            new_svg.push_str(svg_element);
            new_svg.push_str(&svg_template[index..]);
        }
        None => {
            eprintln!("Label not found in the SVG template.");
        }
    }
    // println!("{}", new_svg);
    new_svg
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

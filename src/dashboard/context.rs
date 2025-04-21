use crate::{
    apis::bom::models::{DailyEntry, HourlyForecast, UV},
    constants::NOT_AVAILABLE_ICON,
    dashboard::chart::{DailyForecastGraph, DataType, GraphData, GraphDataPath},
    errors::{DashboardError, Description},
    utils::{find_max_item_between_dates, get_total_between_dates},
    weather::icons::Icon,
    CONFIG,
};
use chrono::{DateTime, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Context {
    pub background_colour: String,
    pub text_colour: String,
    pub x_axis_colour: String,
    pub y_left_axis_colour: String,
    pub y_right_axis_colour: String,
    pub actual_temp_colour: String,
    pub feels_like_colour: String,
    pub rain_colour: String,
    pub max_uv_index: String,
    pub max_uv_index_font_style: String,
    pub max_gust_speed: String,
    pub max_gust_speed_font_style: String,
    pub max_relative_humidity: String,
    pub max_relative_humidity_font_style: String,
    pub total_rain_today: String,
    pub temp_unit: String,
    pub current_hour_actual_temp: String,
    pub current_hour_weather_icon: String,
    pub current_hour_feels_like: String,
    pub current_hour_wind_speed: String,
    pub current_hour_wind_icon: String,
    pub current_hour_uv_index: String,
    pub current_hour_relative_humidity: String,
    pub current_hour_relative_humidity_icon: String,
    pub current_day_date: String,
    pub current_hour_rain_amount: String,
    pub rain_measure_icon: String,
    pub graph_height: String,
    pub graph_width: String,
    pub actual_temp_curve_data: String,
    pub feel_like_curve_data: String,
    pub rain_curve_data: String,
    pub uv_index: String,
    pub uv_index_icon: String,
    pub wind_speed: String,
    pub wind_icon: String,
    pub x_axis_path: String,
    pub x_axis_guideline_path: String,
    pub y_left_axis_path: String,
    pub x_labels: String,
    pub y_left_labels: String,
    pub y_right_axis_path: String,
    pub y_right_labels: String,
    pub uv_gradient: String,
    // daily forecast
    pub day2_mintemp: String,
    pub day2_maxtemp: String,
    pub day2_icon: String,
    pub day2_name: String,
    pub day3_mintemp: String,
    pub day3_maxtemp: String,
    pub day3_icon: String,
    pub day3_name: String,
    pub day4_mintemp: String,
    pub day4_maxtemp: String,
    pub day4_icon: String,
    pub day4_name: String,
    pub day5_mintemp: String,
    pub day5_maxtemp: String,
    pub day5_icon: String,
    pub day5_name: String,
    pub day6_mintemp: String,
    pub day6_maxtemp: String,
    pub day6_icon: String,
    pub day6_name: String,
    pub day7_mintemp: String,
    pub day7_maxtemp: String,
    pub day7_icon: String,
    pub day7_name: String,
    pub sunset_time: String,
    pub sunrise_time: String,
    pub sunset_icon: String,
    pub sunrise_icon: String,
    pub warning_message: String,
    pub warning_icon: String,
    pub warning_visibility: String,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            background_colour: CONFIG.colours.background_colour.clone(),
            text_colour: CONFIG.colours.text_colour.clone(),
            x_axis_colour: CONFIG.colours.x_axis_colour.clone(),
            y_left_axis_colour: CONFIG.colours.y_left_axis_colour.clone(),
            y_right_axis_colour: CONFIG.colours.y_right_axis_colour.clone(),
            actual_temp_colour: CONFIG.colours.temp_colour.clone(),
            feels_like_colour: CONFIG.colours.feels_like_colour.clone(),
            rain_colour: CONFIG.colours.rain_colour.clone(),
            max_uv_index: "NA".to_string(),
            max_uv_index_font_style: "normal".to_string(),
            max_gust_speed: "NA".to_string(),
            max_gust_speed_font_style: "normal".to_string(),
            max_relative_humidity: "NA".to_string(),
            max_relative_humidity_font_style: "normal".to_string(),
            total_rain_today: "NA".to_string(),
            temp_unit: CONFIG.render_options.temp_unit.clone(),
            current_hour_actual_temp: "NA".to_string(),
            current_hour_weather_icon: format!(
                "{}{}",
                CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON
            ),
            current_hour_feels_like: "NA".to_string(),
            current_hour_wind_speed: "NA".to_string(),
            current_hour_wind_icon: format!(
                "{}{}",
                CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON
            ),
            current_hour_uv_index: "NA".to_string(),
            current_hour_relative_humidity: "NA".to_string(),
            current_hour_relative_humidity_icon: format!(
                "{}{}",
                CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON
            ),
            current_day_date: "NA".to_string(),
            current_hour_rain_amount: "NA".to_string(),
            rain_measure_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            graph_height: "300".to_string(),
            graph_width: "600".to_string(),
            actual_temp_curve_data: "".to_string(),
            feel_like_curve_data: "".to_string(),
            rain_curve_data: "".to_string(),
            uv_index: "NA".to_string(),
            uv_index_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            wind_speed: "NA".to_string(),
            wind_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            x_axis_path: "".to_string(),
            x_axis_guideline_path: "".to_string(),
            y_left_axis_path: "".to_string(),
            x_labels: "".to_string(),
            y_left_labels: "".to_string(),
            y_right_axis_path: "".to_string(),
            y_right_labels: "".to_string(),
            uv_gradient: "".to_string(),
            day2_mintemp: "NA".to_string(),
            day2_maxtemp: "NA".to_string(),
            day2_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            day2_name: "NA".to_string(),
            day3_mintemp: "NA".to_string(),
            day3_maxtemp: "NA".to_string(),
            day3_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            day3_name: "NA".to_string(),
            day4_mintemp: "NA".to_string(),
            day4_maxtemp: "NA".to_string(),
            day4_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            day4_name: "NA".to_string(),
            day5_mintemp: "NA".to_string(),
            day5_maxtemp: "NA".to_string(),
            day5_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            day5_name: "NA".to_string(),
            day6_mintemp: "NA".to_string(),
            day6_maxtemp: "NA".to_string(),
            day6_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            day6_name: "NA".to_string(),
            day7_mintemp: "NA".to_string(),
            day7_maxtemp: "NA".to_string(),
            day7_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            day7_name: "NA".to_string(),
            sunrise_time: "NA".to_string(),
            sunset_time: "NA".to_string(),
            sunset_icon: format!("{}sunset.svg", CONFIG.misc.svg_icons_directory),
            sunrise_icon: format!("{}sunrise.svg", CONFIG.misc.svg_icons_directory),
            warning_message: "NA".to_string(),
            warning_icon: format!("{}{}", CONFIG.misc.svg_icons_directory, NOT_AVAILABLE_ICON),
            warning_visibility: "hidden".to_string(),
        }
    }
}

pub struct ContextBuilder {
    pub context: Context,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: Context::default(),
        }
    }

    pub fn with_current_hour_data(&mut self, current_hour: &HourlyForecast) -> &mut Self {
        self.context.current_hour_actual_temp = current_hour.temp.to_string();
        self.context.current_hour_weather_icon = current_hour.get_icon_path();
        self.context.current_hour_feels_like = current_hour.temp_feels_like.to_string();
        self.context.current_hour_wind_speed = current_hour.wind.speed_kilometre.to_string();
        self.context.current_hour_wind_icon = current_hour.wind.get_icon_path();
        self.context.current_hour_uv_index = current_hour.uv.to_string();
        self.context.current_hour_relative_humidity = current_hour.relative_humidity.to_string();
        self.context.current_hour_relative_humidity_icon =
            current_hour.relative_humidity.get_icon_path();
        self.context.current_day_date = chrono::Local::now().format("%A, %d %B").to_string();
        self.context.current_hour_rain_amount = (current_hour.rain.amount.min.unwrap_or(0.0)
            + current_hour.rain.amount.min.unwrap_or(0.0))
        .to_string();
        self.context.rain_measure_icon = current_hour.rain.amount.get_icon_path();
        self
    }

    pub fn with_daily_forecast_data(&mut self, daily_forecast_data: Vec<DailyEntry>) -> &mut Self {
        // The date returned by Bom api is x:14 utc, which translates to x+1:00 AEST time,
        // so we have to do some conversion
        let local_date_truncated = Local::now()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        println!("Local date truncated: {:?}", local_date_truncated);
        let utc_converted_date: DateTime<Utc> = local_date_truncated.with_timezone(&Utc);

        println!("UTC converted date  : {:?}", utc_converted_date);

        let mut day_index: i32 = 1;

        for day in daily_forecast_data {
            if let Some(naive_date) = day.date {
                if naive_date < utc_converted_date {
                    // If the date is in the past, skip it
                    continue;
                } else if naive_date > utc_converted_date + chrono::Duration::days(7) {
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

            // add a day here(or you can add AEST UTC delta), because of the way the API bom api returns the date
            let day_name_value = day.date.map_or("NA".to_string(), |date| {
                date.checked_add_signed(chrono::Duration::days(1))
                    .map(|d| d.format("%a").to_string())
                    .unwrap_or("NA".to_string())
            });

            println!(
                "{} - Max {} Min {}",
                day_name_value, max_temp_value, min_temp_value
            );
            match day_index {
                1 => {
                    self.context.sunrise_time = day
                        .astronomical
                        .unwrap_or_default()
                        .sunrise_time
                        .unwrap_or_default()
                        .format("%H:%M")
                        .to_string();
                    self.context.sunset_time = day
                        .astronomical
                        .unwrap_or_default()
                        .sunset_time
                        .unwrap_or_default()
                        .format("%H:%M")
                        .to_string();
                }
                2 => {
                    self.context.day2_mintemp = min_temp_value;
                    self.context.day2_maxtemp = max_temp_value;
                    self.context.day2_icon = icon_value;
                    self.context.day2_name = day_name_value;
                }
                3 => {
                    self.context.day3_mintemp = min_temp_value;
                    self.context.day3_maxtemp = max_temp_value;
                    self.context.day3_icon = icon_value;
                    self.context.day3_name = day_name_value;
                }
                4 => {
                    self.context.day4_mintemp = min_temp_value;
                    self.context.day4_maxtemp = max_temp_value;
                    self.context.day4_icon = icon_value;
                    self.context.day4_name = day_name_value;
                }
                5 => {
                    self.context.day5_mintemp = min_temp_value;
                    self.context.day5_maxtemp = max_temp_value;
                    self.context.day5_icon = icon_value;
                    self.context.day5_name = day_name_value;
                }
                6 => {
                    self.context.day6_mintemp = min_temp_value;
                    self.context.day6_maxtemp = max_temp_value;
                    self.context.day6_icon = icon_value;
                    self.context.day6_name = day_name_value;
                }
                7 => {
                    self.context.day7_mintemp = min_temp_value;
                    self.context.day7_maxtemp = max_temp_value;
                    self.context.day7_icon = icon_value;
                    self.context.day7_name = day_name_value;
                }
                _ => {}
            }

            day_index += 1;
        }

        if day_index < 8 {
            let details = "Warning: Less than 7 days of daily forecast data, Using Incomplete data"
                .to_string();
            self.set_errors(DashboardError::IncompleteData { details })
        } else {
            self
        }
    }

    pub fn with_hourly_forecast_data(
        &mut self,
        hourly_forecast_data: Vec<HourlyForecast>,
    ) -> &mut Self {
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

        let current_date = chrono::Utc::now()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        println!("Current UTC date          : {:?}", current_date);
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

        if let Some(forecast_window_start) = first_date {
            let forecast_window_end = forecast_window_start + chrono::Duration::hours(24);

            println!("24h forecast window start : {:?}", forecast_window_start);
            println!("24h forecast window end   : {:?}", forecast_window_end);

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
                        self.with_current_hour_data(forecast);
                    } else if x >= 24.0 {
                        eprintln!(
                        "Warning: More than 24 hours of hourly forecast data, this should not happen"
                    );
                        return;
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

            let mut graph = DailyForecastGraph {
                x_axis_always_at_min: CONFIG.render_options.x_axis_always_at_min,
                text_colour: CONFIG.colours.text_colour.clone(),
                ..Default::default()
            };
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

            self.context.graph_height = graph.height.to_string();
            self.context.graph_width = graph.width.to_string();
            self.context.actual_temp_curve_data = temp_curve_data;
            self.context.feel_like_curve_data = feel_like_curve_data;
            self.context.rain_curve_data = rain_curve_data;
            self.context.uv_index = hourly_forecast_data[0].uv.to_string();
            self.context.uv_index_icon = current_uv.get_icon_path().to_string();
            self.context.wind_speed = hourly_forecast_data[0].wind.speed_kilometre.to_string();

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
                self.context.max_gust_speed = max_wind_today.to_string();
            } else {
                self.context.max_gust_speed = max_wind_tomorrow.to_string();
                self.context.max_gust_speed_font_style = "italic".to_string();
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
                self.context.max_uv_index = max_uv_index.to_string();
            } else {
                self.context.max_uv_index = max_uv_index_tomorrow.to_string();
                self.context.max_uv_index_font_style = "italic".to_string();
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
                self.context.max_relative_humidity = max_relative_humidity.to_string();
            } else {
                self.context.max_relative_humidity = max_relative_humidity_tomorrow.to_string();
                self.context.max_relative_humidity_font_style = "italic".to_string();
            }

            self.context.total_rain_today = (get_total_between_dates(
                &hourly_forecast_data,
                &forecast_window_start,
                &forecast_window_end,
                |item: &HourlyForecast| {
                    (item.rain.amount.min.unwrap_or(0.0) + item.rain.amount.max.unwrap_or(0.0))
                        / 2.0
                },
                |item| &item.time,
            ) as usize)
                .to_string();

            self.context.wind_icon = hourly_forecast_data[0].wind.get_icon_path();

            let forecast_window_start_local = forecast_window_start.with_timezone(&chrono::Local);
            let axis_data_path =
                graph.create_axis_with_labels(forecast_window_start_local.hour() as f64);

            self.context.x_axis_path = axis_data_path.x_axis_path;
            self.context.y_left_axis_path = axis_data_path.y_left_axis_path;
            self.context.x_labels = axis_data_path.x_labels;
            self.context.y_left_labels = axis_data_path.y_left_labels;
            self.context.y_right_axis_path = axis_data_path.y_right_axis_path;
            self.context.y_right_labels = axis_data_path.y_right_labels;
            self.context.x_axis_guideline_path = axis_data_path.x_axis_guideline_path;

            self.context.uv_gradient = graph.draw_uv_gradient_over_time(uv_data);
            self
        } else {
            self.set_errors(
                DashboardError::IncompleteData {
                    details: "No hourly forecast data available, Could Not find a date later than the current date".to_string(),
                }
            )
        }
    }

    pub fn set_errors<E: Icon + Description + std::error::Error>(&mut self, error: E) -> &mut Self {
        self.context.warning_message = error.short_description().to_string();
        // TODO: at the moment the last error will overwrite the previous ones, so need to
        // display the errors in a list, front to back in cascading icons style
        self.context.warning_icon = error.get_icon_path().to_string();
        self.context.warning_visibility = "visible".to_string();
        eprintln!("Error: {}", error.long_description());
        self
    }
}

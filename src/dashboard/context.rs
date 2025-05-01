use crate::{
    apis::bom::models::{DailyEntry, HourlyForecast},
    constants::NOT_AVAILABLE_ICON_PATH,
    dashboard::chart::{GraphDataPath, HourlyForecastGraph},
    errors::{DashboardError, Description},
    utils::{find_max_item_between_dates, get_total_between_dates},
    weather::icons::{Icon, SunPositionIconName},
    CONFIG,
};
use chrono::{DateTime, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};

use super::chart::{CurveType, ElementVisibility, FontStyle};

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
        let na = "NA".to_string();
        let not_available_icon_path = NOT_AVAILABLE_ICON_PATH.to_string();
        let colours = CONFIG.colours.clone();
        let render_options = CONFIG.render_options.clone();
        let graph_height = "300".to_string();
        let graph_width = "600".to_string();
        Self {
            background_colour: colours.background_colour,
            text_colour: colours.text_colour,
            x_axis_colour: colours.x_axis_colour,
            y_left_axis_colour: colours.y_left_axis_colour,
            y_right_axis_colour: colours.y_right_axis_colour,
            actual_temp_colour: colours.actual_temp_colour,
            feels_like_colour: colours.feels_like_colour,
            rain_colour: colours.rain_colour,
            max_uv_index: na.clone(),
            max_uv_index_font_style: FontStyle::Normal.to_string(),
            max_gust_speed: na.clone(),
            max_gust_speed_font_style: FontStyle::Normal.to_string(),
            max_relative_humidity: na.clone(),
            max_relative_humidity_font_style: FontStyle::Normal.to_string(),
            total_rain_today: na.clone(),
            temp_unit: render_options.temp_unit,
            current_hour_actual_temp: na.clone(),
            current_hour_weather_icon: not_available_icon_path.clone(),
            current_hour_feels_like: na.clone(),
            current_hour_wind_speed: na.clone(),
            current_hour_wind_icon: not_available_icon_path.clone(),
            current_hour_uv_index: na.clone(),
            current_hour_relative_humidity: na.clone(),
            current_hour_relative_humidity_icon: not_available_icon_path.clone(),
            current_day_date: na.clone(),
            current_hour_rain_amount: na.clone(),
            rain_measure_icon: not_available_icon_path.clone(),
            graph_height,
            graph_width,
            actual_temp_curve_data: String::new(),
            feel_like_curve_data: String::new(),
            rain_curve_data: String::new(),
            uv_index: na.clone(),
            uv_index_icon: not_available_icon_path.clone(),
            wind_speed: na.clone(),
            wind_icon: not_available_icon_path.clone(),
            x_axis_path: String::new(),
            x_axis_guideline_path: String::new(),
            y_left_axis_path: String::new(),
            x_labels: String::new(),
            y_left_labels: String::new(),
            y_right_axis_path: String::new(),
            y_right_labels: String::new(),
            uv_gradient: String::new(),
            day2_mintemp: na.clone(),
            day2_maxtemp: na.clone(),
            day2_icon: not_available_icon_path.clone(),
            day2_name: na.clone(),
            day3_mintemp: na.clone(),
            day3_maxtemp: na.clone(),
            day3_icon: not_available_icon_path.clone(),
            day3_name: na.clone(),
            day4_mintemp: na.clone(),
            day4_maxtemp: na.clone(),
            day4_icon: not_available_icon_path.clone(),
            day4_name: na.clone(),
            day5_mintemp: na.clone(),
            day5_maxtemp: na.clone(),
            day5_icon: not_available_icon_path.clone(),
            day5_name: na.clone(),
            day6_mintemp: na.clone(),
            day6_maxtemp: na.clone(),
            day6_icon: not_available_icon_path.clone(),
            day6_name: na.clone(),
            day7_mintemp: na.clone(),
            day7_maxtemp: na.clone(),
            day7_icon: not_available_icon_path.clone(),
            day7_name: na.clone(),
            sunrise_time: na.clone(),
            sunset_time: na.clone(),
            sunset_icon: SunPositionIconName::Sunset.get_icon_path(),
            sunrise_icon: SunPositionIconName::Sunrise.get_icon_path(),
            warning_message: na,
            warning_icon: not_available_icon_path,
            warning_visibility: ElementVisibility::Hidden.to_string(),
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

    pub fn with_daily_forecast_data(&mut self, daily_forecast_data: Vec<DailyEntry>) -> &mut Self {
        // The date returned by Bom api is x:14 utc, which translates to x+10:00 AEST time,
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
        let (forecast_window_start, forecast_window_end) = match Self::find_forecast_window(
            &hourly_forecast_data,
        ) {
            Some((start, end)) => (start, end),
            None => {
                return self.set_errors(DashboardError::IncompleteData {
                        details: "No hourly forecast data available, Could Not find a date later than the current date".to_string(),
                    });
            }
        };

        println!("24h forecast window start : {:?}", forecast_window_start);
        println!("24h forecast window end   : {:?}", forecast_window_end);

        let mut graph = HourlyForecastGraph {
            x_axis_always_at_min: CONFIG.render_options.x_axis_always_at_min,
            text_colour: CONFIG.colours.text_colour.clone(),
            ..Default::default()
        };

        Self::populate_graph_data(
            self,
            &hourly_forecast_data,
            forecast_window_start,
            forecast_window_end,
            &mut graph,
        );

        let svg_result = graph.draw_graph().unwrap();
        let (temp_curve_data, feel_like_curve_data, rain_curve_data) =
            Self::extract_curve_data(&svg_result);

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
        self.context.uv_index = hourly_forecast_data[0].uv.0.to_string();
        self.context.uv_index_icon = hourly_forecast_data[0].uv.get_icon_path();
        self.context.wind_speed = hourly_forecast_data[0].wind.speed_kilometre.to_string();

        Self::set_max_values(
            self,
            &hourly_forecast_data,
            forecast_window_start,
            day_end,
            forecast_window_end,
        );

        self.context.total_rain_today = (get_total_between_dates(
            &hourly_forecast_data,
            &forecast_window_start,
            &forecast_window_end,
            |item: &HourlyForecast| item.rain.calculate_median_rain(),
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

        self.context.uv_gradient = graph.draw_uv_gradient_over_time();
        self
    }

    fn find_forecast_window(
        hourly_forecast_data: &[HourlyForecast],
    ) -> Option<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)> {
        let current_date = chrono::Utc::now()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        println!("Current UTC date          : {:?}", current_date);

        let first_date = hourly_forecast_data.iter().find_map(|forecast| {
            if forecast.time >= current_date {
                Some(forecast.time)
            } else {
                None
            }
        });

        if let Some(forecast_window_start) = first_date {
            let forecast_window_end = forecast_window_start + chrono::Duration::hours(24);
            Some((forecast_window_start, forecast_window_end))
        } else {
            None
        }
    }

    fn populate_graph_data(
        &mut self,
        hourly_forecast_data: &[HourlyForecast],
        forecast_window_start: chrono::DateTime<Utc>,
        forecast_window_end: chrono::DateTime<Utc>,
        graph: &mut HourlyForecastGraph,
    ) {
        let mut x = 0;
        hourly_forecast_data
            .iter()
            .filter(|forecast| {
                forecast.time >= forecast_window_start && forecast.time < forecast_window_end
            })
            .for_each(|forecast| {
                if x == 0 {
                    self.with_current_hour_data(forecast);
                } else if x >= 24 {
                    eprintln!(
                        "Warning: More than 24 hours of hourly forecast data, this should not happen"
                    );
                    return;
                }
                    // we won't push the actual hour right now
                    // we can calculate it later
                    // we push this index to make scaling graph easier
                for curve_type in &mut graph.curves.iter_mut() {
                    match curve_type {
                        CurveType::ActualTemp(curve) => curve.add_point(x as f64, forecast.temp as f64),
                        CurveType::TempFeelLike(curve) => curve.add_point(x as f64, forecast.temp_feels_like as f64),
                        CurveType::RainChance(curve) => curve.add_point(x as f64, forecast.rain.chance.unwrap_or(0).into()),
                    }
                }
                graph.uv_data[x] = forecast.uv.0 as usize;
                x += 1;
            });
    }

    pub fn with_current_hour_data(&mut self, current_hour: &HourlyForecast) -> &mut Self {
        self.context.current_hour_actual_temp = current_hour.temp.to_string();
        self.context.current_hour_weather_icon = current_hour.get_icon_path();
        self.context.current_hour_feels_like = current_hour.temp_feels_like.to_string();
        self.context.current_hour_wind_speed = current_hour.wind.speed_kilometre.to_string();
        self.context.current_hour_wind_icon = current_hour.wind.get_icon_path();
        self.context.current_hour_uv_index = current_hour.uv.0.to_string();
        self.context.current_hour_relative_humidity = current_hour.relative_humidity.0.to_string();
        self.context.current_hour_relative_humidity_icon =
            current_hour.relative_humidity.get_icon_path();
        self.context.current_day_date = chrono::Local::now().format("%A, %d %B").to_string();
        self.context.current_hour_rain_amount =
            current_hour.rain.calculate_median_rain().to_string();
        self.context.rain_measure_icon = current_hour.rain.amount.get_icon_path();
        self
    }

    fn extract_curve_data(svg_result: &[GraphDataPath]) -> (String, String, String) {
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
        )
    }

    fn set_max_values(
        &mut self,
        hourly_forecast_data: &[HourlyForecast],
        forecast_window_start: chrono::DateTime<Utc>,
        day_end: chrono::DateTime<Utc>,
        forecast_window_end: chrono::DateTime<Utc>,
    ) {
        let max_wind_today = find_max_item_between_dates(
            hourly_forecast_data,
            &forecast_window_start,
            &day_end,
            |item| item.wind.gust_speed_kilometre.unwrap_or(0),
            |item| &item.time,
        );

        let max_wind_tomorrow = find_max_item_between_dates(
            hourly_forecast_data,
            &day_end,
            &forecast_window_end,
            |item| item.wind.gust_speed_kilometre.unwrap_or(0),
            |item| &item.time,
        );

        if max_wind_today > max_wind_tomorrow {
            self.context.max_gust_speed = max_wind_today.to_string();
        } else {
            self.context.max_gust_speed = max_wind_tomorrow.to_string();
            self.context.max_gust_speed_font_style = FontStyle::Italic.to_string();
        }

        let max_uv_index = find_max_item_between_dates(
            hourly_forecast_data,
            &forecast_window_start,
            &day_end,
            |item| item.uv,
            |item| &item.time,
        );

        let max_uv_index_tomorrow = find_max_item_between_dates(
            hourly_forecast_data,
            &day_end,
            &forecast_window_end,
            |item| item.uv,
            |item| &item.time,
        );

        if max_uv_index > max_uv_index_tomorrow {
            self.context.max_uv_index = max_uv_index.0.to_string();
        } else {
            self.context.max_uv_index = max_uv_index_tomorrow.0.to_string();
            self.context.max_uv_index_font_style = FontStyle::Italic.to_string();
        }

        let max_relative_humidity = find_max_item_between_dates(
            hourly_forecast_data,
            &forecast_window_start,
            &day_end,
            |item| item.relative_humidity,
            |item| &item.time,
        );

        let max_relative_humidity_tomorrow = find_max_item_between_dates(
            hourly_forecast_data,
            &day_end,
            &forecast_window_end,
            |item| item.relative_humidity,
            |item| &item.time,
        );

        if max_relative_humidity > max_relative_humidity_tomorrow {
            self.context.max_relative_humidity = max_relative_humidity.0.to_string();
        } else {
            self.context.max_relative_humidity = max_relative_humidity_tomorrow.0.to_string();
            self.context.max_relative_humidity_font_style = FontStyle::Italic.to_string();
        }
    }

    pub fn set_errors<E: Icon + Description + std::error::Error>(&mut self, error: E) -> &mut Self {
        self.context.warning_message = error.short_description().to_string();
        // TODO: at the moment the last error will overwrite the previous ones, so need to
        // display the errors in a list, front to back in cascading icons style
        self.context.warning_icon = error.get_icon_path().to_string();
        self.context.warning_visibility = ElementVisibility::Visible.to_string();
        eprintln!("Error: {}", error.long_description());
        self
    }
}

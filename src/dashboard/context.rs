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
    // colours
    pub background_colour: String,
    pub text_colour: String,
    pub x_axis_colour: String,
    pub y_left_axis_colour: String,
    pub y_right_axis_colour: String,
    pub actual_temp_colour: String,
    pub feels_like_colour: String,
    pub rain_colour: String,
    // any weather element that is not graph
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
    pub current_hour_uv_index_icon: String,
    pub current_hour_relative_humidity: String,
    pub current_hour_relative_humidity_icon: String,
    pub current_day_date: String,
    pub current_hour_rain_amount: String,
    pub current_hour_rain_measure_icon: String,
    pub sunset_time: String,
    pub sunrise_time: String,
    pub sunset_icon: String,
    pub sunrise_icon: String,
    // these values might not be used
    pub graph_height: String,
    pub graph_width: String,
    // graph and curves
    pub actual_temp_curve_data: String,
    pub feel_like_curve_data: String,
    pub rain_curve_data: String,
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
    // warning message
    pub warning_message: String,
    pub warning_icon: String,
    pub warning_visibility: String,
}

impl Default for Context {
    fn default() -> Self {
        let na = "NA".to_string();
        let not_available_icon_path = NOT_AVAILABLE_ICON_PATH.to_string_lossy().to_string();
        let colours = CONFIG.colours.clone();
        let render_options = CONFIG.render_options.clone();
        let graph_height = "300".to_string();
        let graph_width = "600".to_string();
        Self {
            background_colour: colours.background_colour.to_string(),
            text_colour: colours.text_colour.to_string(),
            x_axis_colour: colours.x_axis_colour.to_string(),
            y_left_axis_colour: colours.y_left_axis_colour.to_string(),
            y_right_axis_colour: colours.y_right_axis_colour.to_string(),
            actual_temp_colour: colours.actual_temp_colour.to_string(),
            feels_like_colour: colours.feels_like_colour.to_string(),
            rain_colour: colours.rain_colour.to_string(),
            max_uv_index: na.clone(),
            max_uv_index_font_style: FontStyle::Normal.to_string(),
            max_gust_speed: na.clone(),
            max_gust_speed_font_style: FontStyle::Normal.to_string(),
            max_relative_humidity: na.clone(),
            max_relative_humidity_font_style: FontStyle::Normal.to_string(),
            total_rain_today: na.clone(),
            temp_unit: render_options.temp_unit.to_string(),
            current_hour_actual_temp: na.clone(),
            current_hour_weather_icon: not_available_icon_path.clone(),
            current_hour_feels_like: na.clone(),
            current_hour_wind_speed: na.clone(),
            current_hour_wind_icon: not_available_icon_path.clone(),
            current_hour_uv_index: na.clone(),
            current_hour_uv_index_icon: not_available_icon_path.clone(),
            current_hour_relative_humidity: na.clone(),
            current_hour_relative_humidity_icon: not_available_icon_path.clone(),
            current_day_date: na.clone(),
            current_hour_rain_amount: na.clone(),
            current_hour_rain_measure_icon: not_available_icon_path.clone(),
            sunrise_time: na.clone(),
            sunset_time: na.clone(),
            sunset_icon: SunPositionIconName::Sunset.get_icon_path(),
            sunrise_icon: SunPositionIconName::Sunrise.get_icon_path(),
            graph_height,
            graph_width,
            actual_temp_curve_data: String::new(),
            feel_like_curve_data: String::new(),
            rain_curve_data: String::new(),
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
        // The date returned by Bom api is UTC, for example x:14 UTC, which translates to x:14+10:00 AEST time,
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
                        .with_timezone(&Local)
                        .format("%H:%M")
                        .to_string();
                    self.context.sunset_time = day
                        .astronomical
                        .unwrap_or_default()
                        .sunset_time
                        .unwrap_or_default()
                        .with_timezone(&Local)
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

    // Extrusion Pattern: force everything through one function until it resembles spaghetti
    pub fn with_hourly_forecast_data(
        &mut self,
        hourly_forecast_data: Vec<HourlyForecast>,
    ) -> &mut Self {
        let (utc_forecast_window_start, utc_forecast_window_end) = match Self::find_forecast_window(
            &hourly_forecast_data,
        ) {
            Some((start, end)) => (start, end),
            None => {
                return self.set_errors(DashboardError::IncompleteData {
                        details: "No hourly forecast data available, Could Not find a date later than the current date".to_string(),
                    });
            }
        };

        println!(
            "24h UTC forecast window: start = {:?}, end = {:?}",
            utc_forecast_window_start, utc_forecast_window_end
        );

        let local_forecast_window_start: DateTime<Local> =
            utc_forecast_window_start.with_timezone(&Local);
        let local_forecast_window_end: DateTime<Local> =
            utc_forecast_window_end.with_timezone(&Local);
        let day_end = local_forecast_window_start
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            + chrono::Duration::days(1);

        println!(
            "Local forecast window: start = {:?}, end = {:?}",
            local_forecast_window_start, local_forecast_window_end
        );

        // println!("Day end: {:?}", day_end);

        let mut graph = HourlyForecastGraph {
            x_axis_always_at_min: CONFIG.render_options.x_axis_always_at_min,
            text_colour: CONFIG.colours.text_colour.to_string(),
            ..Default::default()
        };

        Self::populate_graph_data(
            self,
            &hourly_forecast_data,
            local_forecast_window_start,
            local_forecast_window_end,
            &mut graph,
        );

        let svg_result = graph.draw_graph().unwrap();
        let (temp_curve_data, feel_like_curve_data, rain_curve_data) =
            Self::extract_curve_data(&svg_result);
        self.context.graph_height = graph.height.to_string();
        self.context.graph_width = graph.width.to_string();
        self.context.actual_temp_curve_data = temp_curve_data;
        self.context.feel_like_curve_data = feel_like_curve_data;
        self.context.rain_curve_data = rain_curve_data;

        let axis_data_path =
            graph.create_axis_with_labels(local_forecast_window_start.hour() as f32);

        self.context.x_axis_path = axis_data_path.x_axis_path;
        self.context.y_left_axis_path = axis_data_path.y_left_axis_path;
        self.context.x_labels = axis_data_path.x_labels;
        self.context.y_left_labels = axis_data_path.y_left_labels;
        self.context.y_right_axis_path = axis_data_path.y_right_axis_path;
        self.context.y_right_labels = axis_data_path.y_right_labels;
        self.context.x_axis_guideline_path = axis_data_path.x_axis_guideline_path;

        self.context.uv_gradient = graph.draw_uv_gradient_over_time();

        Self::set_max_values_for_table(
            self,
            &hourly_forecast_data,
            local_forecast_window_start,
            day_end,
            local_forecast_window_end,
        );

        self.context.total_rain_today = (get_total_between_dates(
            &hourly_forecast_data,
            &local_forecast_window_start,
            &local_forecast_window_end,
            |item: &HourlyForecast| item.rain.calculate_median_rain(),
            |item| item.time.with_timezone(&Local),
        ))
        .to_string();

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
        println!("Current time (UTC, to the hour)     : {:?}", current_date);

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

    fn populate_graph_data(
        &mut self,
        hourly_forecast_data: &[HourlyForecast],
        forecast_window_start: chrono::DateTime<Local>,
        forecast_window_end: chrono::DateTime<Local>,
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
                    self.set_now_values_for_table(forecast)
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
                        CurveType::ActualTemp(curve) => curve.add_point(x as f32, *forecast.temp),
                        CurveType::TempFeelLike(curve) => curve.add_point(x as f32, *forecast.temp_feels_like),
                        CurveType::RainChance(curve) => curve.add_point(x as f32, forecast.rain.chance.unwrap_or(0) as f32),
                    }
                }
                graph.uv_data[x] = forecast.uv.0;
                x += 1;
            });
    }

    fn with_current_hour_data(&mut self, current_hour: &HourlyForecast) -> &mut Self {
        self.context.current_hour_actual_temp = current_hour.temp.to_string();
        self.context.current_hour_weather_icon = current_hour.get_icon_path();
        self.context.current_hour_feels_like = current_hour.temp_feels_like.to_string();
        self.context.current_day_date = chrono::Local::now().format("%A, %d %B").to_string();
        self.context.current_hour_rain_amount =
            current_hour.rain.calculate_median_rain().to_string();
        self.context.current_hour_rain_measure_icon = current_hour.rain.amount.get_icon_path();

        self
    }

    fn set_now_values_for_table(&mut self, current_hour: &HourlyForecast) {
        self.context.current_hour_wind_speed = current_hour.wind.get_speed().to_string();
        self.context.current_hour_wind_icon = current_hour.wind.get_icon_path();
        self.context.current_hour_uv_index = current_hour.uv.0.to_string();
        self.context.current_hour_uv_index_icon = current_hour.uv.get_icon_path();
        self.context.current_hour_relative_humidity = current_hour.relative_humidity.0.to_string();
        self.context.current_hour_relative_humidity_icon =
            current_hour.relative_humidity.get_icon_path();
    }

    fn set_max_values_for_table(
        &mut self,
        hourly_forecast_data: &[HourlyForecast],
        forecast_window_start: chrono::DateTime<Local>,
        day_end: chrono::DateTime<Local>,
        forecast_window_end: chrono::DateTime<Local>,
    ) {
        println!("### Calculating table Max24h...");
        let today_duration = day_end
            .signed_duration_since(forecast_window_start)
            .num_hours();
        println!(
            "Today's Forecast Window: start = {:?}, end = {:?}, duration = {} hours",
            forecast_window_start, day_end, today_duration
        );

        let tomorrow_duration = forecast_window_end
            .signed_duration_since(day_end)
            .num_hours();
        println!(
            "Tomorrow's Forecast Window: start = {:?}, end = {:?}, duration = {} hours",
            day_end, forecast_window_end, tomorrow_duration
        );

        macro_rules! max_in_today_and_tomorrow {
            ($get_value:expr) => {{
                let get_time = |item: &HourlyForecast| item.time.with_timezone(&Local);
                let max_today = find_max_item_between_dates(
                    hourly_forecast_data,
                    &forecast_window_start,
                    &day_end,
                    $get_value,
                    get_time,
                );
                let max_tomorrow = find_max_item_between_dates(
                    hourly_forecast_data,
                    &day_end,
                    &forecast_window_end,
                    $get_value,
                    get_time,
                );
                (max_today, max_tomorrow)
            }};
        }

        let (max_wind_today, max_wind_tomorrow) =
            max_in_today_and_tomorrow!(|item| item.wind.get_speed());

        if max_wind_today > max_wind_tomorrow {
            self.context.max_gust_speed = max_wind_today.to_string();
        } else {
            self.context.max_gust_speed = max_wind_tomorrow.to_string();
            self.context.max_gust_speed_font_style = FontStyle::Italic.to_string();
        }

        let (max_uv_index_today, max_uv_index_tomorrow) =
            max_in_today_and_tomorrow!(|item| item.uv);

        if max_uv_index_today > max_uv_index_tomorrow {
            self.context.max_uv_index = max_uv_index_today.0.to_string();
        } else {
            self.context.max_uv_index = max_uv_index_tomorrow.0.to_string();
            self.context.max_uv_index_font_style = FontStyle::Italic.to_string();
        }

        let (max_relative_humidity_today, max_relative_humidity_tomorrow) =
            max_in_today_and_tomorrow!(|item| item.relative_humidity);

        if max_relative_humidity_today > max_relative_humidity_tomorrow {
            self.context.max_relative_humidity = max_relative_humidity_today.0.to_string();
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

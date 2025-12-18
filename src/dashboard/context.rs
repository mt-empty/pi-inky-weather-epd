use crate::{
    clock::Clock,
    constants::NOT_AVAILABLE_ICON_PATH,
    dashboard::chart::{GraphDataPath, HourlyForecastGraph},
    domain::models::{DailyForecast, HourlyForecast},
    errors::{DashboardError, Description},
    logger,
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
    pub current_wind_speed_unit: String,
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
    pub diagnostic_message: String,
    pub diagnostic_visibility: String,
    // cascading diagnostic icons (SVG fragments for multiple stacked icons)
    pub diagnostic_icons_svg: String,
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
            current_wind_speed_unit: render_options.wind_speed_unit.to_string(),
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
            diagnostic_message: na,
            diagnostic_visibility: ElementVisibility::Hidden.to_string(),
            diagnostic_icons_svg: String::new(),
        }
    }
}

pub struct ContextBuilder {
    pub context: Context,
    diagnostics: Vec<DashboardError>,
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: Context::default(),
            diagnostics: Vec::new(),
        }
    }

    /// Updates the warning display fields based on the highest priority diagnostic.
    /// Called internally after adding diagnostics.
    fn update_warning_display(&mut self) {
        if let Some(highest_priority_error) = self.diagnostics.iter().max_by_key(|e| e.priority()) {
            // Show message for highest priority error only
            self.context.diagnostic_message =
                highest_priority_error.short_description().to_string();
            self.context.diagnostic_visibility = ElementVisibility::Visible.to_string();

            // Generate cascading icons SVG for all diagnostics (sorted by priority)
            self.context.diagnostic_icons_svg = self.generate_cascading_icons_svg();
        } else {
            // No diagnostics - hide warning
            self.context.diagnostic_visibility = ElementVisibility::Hidden.to_string();
            self.context.diagnostic_icons_svg = String::new();
        }
    }

    /// Generates SVG fragments for cascading diagnostic icons.
    /// Icons are stacked diagonally with offset, sorted by priority (high to low).
    /// Highest priority appears at front (lowest x, lowest y), lowest priority at back.
    fn generate_cascading_icons_svg(&self) -> String {
        let mut sorted_diagnostics = self.diagnostics.clone();
        sorted_diagnostics.sort_by_key(|e| std::cmp::Reverse(e.priority())); // High to low

        let icon_size = 74;
        let x_start = 63; // Starting X position for highest priority
        let y_start = -10; // Starting Y position for highest priority
        let x_offset = -5; // Move each subsequent icon left (creates depth)
        let y_offset = -3; // Move each subsequent icon up (creates depth)

        // Reverse order so lowest priority renders first (appears in back)
        sorted_diagnostics
            .iter()
            .enumerate()
            .rev()
            .map(|(index, error)| {
                let x_pos = x_start + (index as i32 * x_offset);
                let y_pos = y_start + (index as i32 * y_offset);
                format!(
                    r#"<image x="{x_pos}" y="{y_pos}" width="{icon_size}" height="{icon_size}" href="{}"/>"#,
                    error.get_icon_path()
                )
            })
            .collect::<Vec<String>>()
            .join("\n        ")
    }

    pub fn with_daily_forecast_data(
        &mut self,
        daily_forecast_data: Vec<DailyForecast>,
        clock: &dyn Clock,
    ) -> &mut Self {
        // Get today's local date for comparison
        let today_local_date = clock.now_local().date_naive();

        logger::detail(format!(
            "Processing daily forecast starting from: {today_local_date}"
        ));

        // Pre-populate day names from local calendar (tomorrow through +6 days)
        self.initialize_day_names(clock.now_local());

        // day_index tracks which forecast day we're filling (0-6 for 7 days)
        // day_index 0 = today (sunrise/sunset only), indices 1-6 = tomorrow through +5 days (day2-day7)
        let mut day_index: i32 = 0;

        for day in daily_forecast_data {
            if let Some(forecast_local_date) = day.date {
                // Skip any dates before today (past dates)
                if forecast_local_date < today_local_date {
                    continue;
                }

                // Once we have 7 days, stop processing
                if day_index >= 7 {
                    break;
                }

                // Warn if date is beyond our 7-day window
                if forecast_local_date >= today_local_date + chrono::Days::new(7) {
                    logger::warning(format!(
                        "Reached day beyond 7-day forecast window: {forecast_local_date}"
                    ));
                }
            } else {
                logger::warning("Daily forecast entry missing date, skipping");
                continue;
            }

            let min_temp_value = day
                .temp_min
                .map_or("NA".to_string(), |temp| temp.to_string());
            let max_temp_value = day
                .temp_max
                .map_or("NA".to_string(), |temp| temp.to_string());
            let icon_value = day.get_icon_path();

            let day_name = match day_index {
                0 => "Today",
                1 => &self.context.day2_name,
                2 => &self.context.day3_name,
                3 => &self.context.day4_name,
                4 => &self.context.day5_name,
                5 => &self.context.day6_name,
                6 => &self.context.day7_name,
                _ => "Unknown",
            };

            logger::detail(format!(
                "{day_name} ({}) - Max {max_temp_value}°, Min {min_temp_value}°",
                day.date.unwrap()
            ));

            match day_index {
                0 => {
                    // Day 0 (today) - show sunrise/sunset times
                    if let Some(ref astro) = day.astronomical {
                        self.context.sunrise_time = astro
                            .sunrise_time
                            .unwrap_or_default()
                            .with_timezone(&Local)
                            .format("%H:%M")
                            .to_string();
                        self.context.sunset_time = astro
                            .sunset_time
                            .unwrap_or_default()
                            .with_timezone(&Local)
                            .format("%H:%M")
                            .to_string();
                    }
                }
                1 => {
                    self.context.day2_mintemp = min_temp_value;
                    self.context.day2_maxtemp = max_temp_value;
                    self.context.day2_icon = icon_value;
                }
                2 => {
                    self.context.day3_mintemp = min_temp_value;
                    self.context.day3_maxtemp = max_temp_value;
                    self.context.day3_icon = icon_value;
                }
                3 => {
                    self.context.day4_mintemp = min_temp_value;
                    self.context.day4_maxtemp = max_temp_value;
                    self.context.day4_icon = icon_value;
                }
                4 => {
                    self.context.day5_mintemp = min_temp_value;
                    self.context.day5_maxtemp = max_temp_value;
                    self.context.day5_icon = icon_value;
                }
                5 => {
                    self.context.day6_mintemp = min_temp_value;
                    self.context.day6_maxtemp = max_temp_value;
                    self.context.day6_icon = icon_value;
                }
                6 => {
                    self.context.day7_mintemp = min_temp_value;
                    self.context.day7_maxtemp = max_temp_value;
                    self.context.day7_icon = icon_value;
                }
                _ => {}
            }

            day_index += 1;
        }

        // Verify we got all 7 days (indices 0-6 means day_index reaches 7)
        if day_index < 7 {
            let details = "Warning: Less than 7 days of daily forecast data, Using Incomplete data"
                .to_string();
            self.with_validation_error(DashboardError::IncompleteData { details })
        } else {
            self
        }
    }

    fn initialize_day_names(&mut self, local_midnight_time: DateTime<Local>) {
        // Pre-fill day names based on local calendar (independent of forecast data)
        self.context.day2_name = (local_midnight_time + chrono::Duration::days(1))
            .format("%a")
            .to_string();
        self.context.day3_name = (local_midnight_time + chrono::Duration::days(2))
            .format("%a")
            .to_string();
        self.context.day4_name = (local_midnight_time + chrono::Duration::days(3))
            .format("%a")
            .to_string();
        self.context.day5_name = (local_midnight_time + chrono::Duration::days(4))
            .format("%a")
            .to_string();
        self.context.day6_name = (local_midnight_time + chrono::Duration::days(5))
            .format("%a")
            .to_string();
        self.context.day7_name = (local_midnight_time + chrono::Duration::days(6))
            .format("%a")
            .to_string();
    }

    // Extrusion Pattern: force everything through one function until it resembles spaghetti
    pub fn with_hourly_forecast_data(
        &mut self,
        hourly_forecast_data: Vec<HourlyForecast>,
        clock: &dyn Clock,
    ) -> &mut Self {
        let (utc_forecast_window_start, utc_forecast_window_end) = match Self::find_forecast_window(
            &hourly_forecast_data,
            clock,
        ) {
            Some((start, end)) => (start, end),
            None => {
                return self.with_validation_error(DashboardError::IncompleteData {
                        details: "No hourly forecast data available, Could Not find a date later than the current date".to_string(),
                    });
            }
        };

        logger::detail(format!(
            "24h UTC forecast window: {} to {}",
            utc_forecast_window_start.format("%Y-%m-%d %H:%M"),
            utc_forecast_window_end.format("%Y-%m-%d %H:%M")
        ));

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

        logger::detail(format!(
            "Local forecast window: {} to {}",
            local_forecast_window_start.format("%Y-%m-%d %H:%M %Z"),
            local_forecast_window_end.format("%Y-%m-%d %H:%M %Z")
        ));

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
            clock,
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
            graph.create_axis_with_labels(local_forecast_window_start.hour() as f32, clock);

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
            |item: &HourlyForecast| item.precipitation.calculate_median(),
            |item| item.time.with_timezone(&Local),
        ))
        .to_string();

        self
    }

    fn find_forecast_window(
        hourly_forecast_data: &[HourlyForecast],
        clock: &dyn Clock,
    ) -> Option<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)> {
        let current_date = clock
            .now_utc()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        logger::detail(format!(
            "Current hour (UTC): {}",
            current_date.format("%Y-%m-%d %H:%M")
        ));

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
        clock: &dyn Clock,
    ) {
        let mut x = 0;
        hourly_forecast_data
            .iter()
            .filter(|forecast| {
                forecast.time >= forecast_window_start && forecast.time < forecast_window_end
            })
            .for_each(|forecast| {
                if x == 0 {
                    self.with_current_hour_data(forecast, clock);
                    self.set_now_values_for_table(forecast)
                } else if x >= 24 {
                    logger::warning(
                        "More than 24 hours of hourly forecast data, this should not happen",
                    );
                    return;
                }
                // we won't push the actual hour right now
                // we can calculate it later
                // we push this index to make scaling graph easier
                for curve_type in &mut graph.curves.iter_mut() {
                    match curve_type {
                        CurveType::ActualTemp(curve) => {
                            curve.add_point(x as f32, *forecast.temperature)
                        }
                        CurveType::TempFeelLike(curve) => {
                            curve.add_point(x as f32, *forecast.apparent_temperature)
                        }
                        CurveType::RainChance(curve) => curve
                            .add_point(x as f32, forecast.precipitation.chance.unwrap_or(0) as f32),
                    }
                }
                graph.uv_data[x] = forecast.uv_index;
                x += 1;
            });
    }

    fn with_current_hour_data(
        &mut self,
        current_hour: &HourlyForecast,
        clock: &dyn Clock,
    ) -> &mut Self {
        self.context.current_hour_actual_temp = current_hour.temperature.to_string();
        self.context.current_hour_weather_icon = current_hour.get_icon_path();
        self.context.current_hour_feels_like = current_hour.apparent_temperature.to_string();
        self.context.current_day_date = clock
            .now_local()
            .format(&CONFIG.render_options.date_format)
            .to_string();
        self.context.current_hour_rain_amount =
            current_hour.precipitation.calculate_median().to_string();
        self.context.current_hour_rain_measure_icon = current_hour.precipitation.get_icon_path();

        self
    }

    fn set_now_values_for_table(&mut self, current_hour: &HourlyForecast) {
        self.context.current_hour_wind_speed = current_hour
            .wind
            .get_speed_in_unit(
                CONFIG.render_options.use_gust_instead_of_wind,
                CONFIG.render_options.wind_speed_unit,
            )
            .to_string();
        self.context.current_hour_wind_icon = current_hour.wind.get_icon_path();
        self.context.current_hour_uv_index = current_hour.uv_index.to_string();
        self.context.current_hour_uv_index_icon =
            crate::domain::icons::UVIndex(current_hour.uv_index).get_icon_path();
        self.context.current_hour_relative_humidity = current_hour.relative_humidity.to_string();
        self.context.current_hour_relative_humidity_icon =
            crate::domain::icons::RelativeHumidity(current_hour.relative_humidity).get_icon_path();
    }

    fn set_max_values_for_table(
        &mut self,
        hourly_forecast_data: &[HourlyForecast],
        forecast_window_start: chrono::DateTime<Local>,
        day_end: chrono::DateTime<Local>,
        forecast_window_end: chrono::DateTime<Local>,
    ) {
        logger::detail("Calculating Max24h values for table");
        let today_duration = day_end
            .signed_duration_since(forecast_window_start)
            .num_hours();
        logger::detail(format!(
            "Today's graph slice: {} to {} ({} hours)",
            forecast_window_start.format("%H:%M"),
            day_end.format("%H:%M"),
            today_duration
        ));

        let tomorrow_duration = forecast_window_end
            .signed_duration_since(day_end)
            .num_hours();
        logger::detail(format!(
            "Tomorrow's graph slice: {} to {} ({} hours)",
            day_end.format("%H:%M"),
            forecast_window_end.format("%H:%M"),
            tomorrow_duration
        ));

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

        let (max_wind_today, max_wind_tomorrow) = max_in_today_and_tomorrow!(|item| item
            .wind
            .get_speed(CONFIG.render_options.use_gust_instead_of_wind));

        // Convert wind speed to configured unit
        let max_wind_today_converted = crate::domain::models::Wind::convert_speed(
            max_wind_today,
            CONFIG.render_options.wind_speed_unit,
        );
        let max_wind_tomorrow_converted = crate::domain::models::Wind::convert_speed(
            max_wind_tomorrow,
            CONFIG.render_options.wind_speed_unit,
        );

        if max_wind_today > max_wind_tomorrow {
            self.context.max_gust_speed = max_wind_today_converted.to_string();
        } else {
            self.context.max_gust_speed = max_wind_tomorrow_converted.to_string();
            self.context.max_gust_speed_font_style = FontStyle::Italic.to_string();
        }

        let (max_uv_index_today, max_uv_index_tomorrow) =
            max_in_today_and_tomorrow!(|item| item.uv_index);

        if max_uv_index_today > max_uv_index_tomorrow {
            self.context.max_uv_index = max_uv_index_today.to_string();
        } else {
            self.context.max_uv_index = max_uv_index_tomorrow.to_string();
            self.context.max_uv_index_font_style = FontStyle::Italic.to_string();
        }

        let (max_relative_humidity_today, max_relative_humidity_tomorrow) =
            max_in_today_and_tomorrow!(|item| item.relative_humidity);

        if max_relative_humidity_today > max_relative_humidity_tomorrow {
            self.context.max_relative_humidity = max_relative_humidity_today.to_string();
        } else {
            self.context.max_relative_humidity = max_relative_humidity_tomorrow.to_string();
            self.context.max_relative_humidity_font_style = FontStyle::Italic.to_string();
        }
    }

    /// Sets a validation error detected internally during context building.
    ///
    /// This method is used when data validation fails (e.g., incomplete forecast data).
    /// It logs the error to stderr, adds it to the diagnostics collection, and updates
    /// the warning display to show the highest priority error.
    ///
    /// Use this for internal validation errors. For external API warnings, use `with_warning`.
    pub fn with_validation_error(&mut self, error: DashboardError) -> &mut Self {
        logger::error(error.long_description());
        self.diagnostics.push(error);
        self.update_warning_display();
        self
    }

    /// Sets a warning message propagated from external sources (e.g., API issues).
    ///
    /// This method is used when external dependencies have issues but fallback data is available
    /// (e.g., using stale cached data because API is unreachable).
    ///
    /// Unlike `with_validation_error`, this does NOT log to stderr because the caller
    /// is expected to have already logged the warning.
    ///
    /// Adds the warning to the diagnostics collection and updates the display to show
    /// the highest priority diagnostic.
    pub fn with_warning(&mut self, warning: DashboardError) -> &mut Self {
        self.diagnostics.push(warning);
        self.update_warning_display();
        self
    }
}

use crate::{
    clock::Clock,
    configs::settings::DashboardSettings,
    constants::{not_available_icon_path, NOT_AVAILABLE},
    dashboard::chart::{GraphDataPath, HourlyForecastGraph},
    domain::models::{DailyForecast, HourlyForecast},
    errors::{DashboardError, Description},
    logger,
    utils::{find_max_item_between_dates, total_between_dates},
    weather::icons::{HumidityIconName, Icon, IconContext, SunPositionIconName, UVIndexIcon},
};
use chrono::{DateTime, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::chart::{generate_unified_precipitation_svg, CurveType, ElementVisibility, FontStyle};

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
    pub snow_colour: String,
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
    // Debug information (displayed when debugging enabled)
    pub debug_info_visibility: String,
    pub debug_version: String,
    pub debug_provider: String,
    pub debug_location: String,
    pub debug_timezone: String,
}

impl Context {
    fn new(settings: &DashboardSettings, today: NaiveDate) -> Self {
        let icon_ctx = IconContext::from_settings(settings, today);
        let not_available_icon_path = not_available_icon_path(settings)
            .to_string_lossy()
            .to_string();
        let colours = settings.colours.clone();
        let render_options = settings.render_options.clone();
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
            snow_colour: colours.snow_colour.to_string(),
            max_uv_index: NOT_AVAILABLE.to_string(),
            max_uv_index_font_style: FontStyle::Normal.to_string(),
            max_gust_speed: NOT_AVAILABLE.to_string(),
            max_gust_speed_font_style: FontStyle::Normal.to_string(),
            max_relative_humidity: NOT_AVAILABLE.to_string(),
            max_relative_humidity_font_style: FontStyle::Normal.to_string(),
            total_rain_today: NOT_AVAILABLE.to_string(),
            temp_unit: render_options.temp_unit.to_string(),
            current_wind_speed_unit: render_options.wind_speed_unit.to_string(),
            current_hour_actual_temp: NOT_AVAILABLE.to_string(),
            current_hour_weather_icon: not_available_icon_path.clone(),
            current_hour_feels_like: NOT_AVAILABLE.to_string(),
            current_hour_wind_speed: NOT_AVAILABLE.to_string(),
            current_hour_wind_icon: not_available_icon_path.clone(),
            current_hour_uv_index: NOT_AVAILABLE.to_string(),
            current_hour_uv_index_icon: not_available_icon_path.clone(),
            current_hour_relative_humidity: NOT_AVAILABLE.to_string(),
            current_hour_relative_humidity_icon: not_available_icon_path.clone(),
            current_day_date: NOT_AVAILABLE.to_string(),
            current_hour_rain_amount: NOT_AVAILABLE.to_string(),
            sunrise_time: NOT_AVAILABLE.to_string(),
            sunset_time: NOT_AVAILABLE.to_string(),
            sunset_icon: SunPositionIconName::Sunset.icon_path(&icon_ctx),
            sunrise_icon: SunPositionIconName::Sunrise.icon_path(&icon_ctx),
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
            day2_mintemp: NOT_AVAILABLE.to_string(),
            day2_maxtemp: NOT_AVAILABLE.to_string(),
            day2_icon: not_available_icon_path.clone(),
            day2_name: NOT_AVAILABLE.to_string(),
            day3_mintemp: NOT_AVAILABLE.to_string(),
            day3_maxtemp: NOT_AVAILABLE.to_string(),
            day3_icon: not_available_icon_path.clone(),
            day3_name: NOT_AVAILABLE.to_string(),
            day4_mintemp: NOT_AVAILABLE.to_string(),
            day4_maxtemp: NOT_AVAILABLE.to_string(),
            day4_icon: not_available_icon_path.clone(),
            day4_name: NOT_AVAILABLE.to_string(),
            day5_mintemp: NOT_AVAILABLE.to_string(),
            day5_maxtemp: NOT_AVAILABLE.to_string(),
            day5_icon: not_available_icon_path.clone(),
            day5_name: NOT_AVAILABLE.to_string(),
            day6_mintemp: NOT_AVAILABLE.to_string(),
            day6_maxtemp: NOT_AVAILABLE.to_string(),
            day6_icon: not_available_icon_path.clone(),
            day6_name: NOT_AVAILABLE.to_string(),
            day7_mintemp: NOT_AVAILABLE.to_string(),
            day7_maxtemp: NOT_AVAILABLE.to_string(),
            day7_icon: not_available_icon_path.clone(),
            day7_name: NOT_AVAILABLE.to_string(),
            diagnostic_message: NOT_AVAILABLE.to_string(),
            diagnostic_visibility: ElementVisibility::Hidden.to_string(),
            diagnostic_icons_svg: String::new(),
            debug_version: String::new(),
            debug_info_visibility: ElementVisibility::Hidden.to_string(),
            debug_provider: String::new(),
            debug_location: String::new(),
            debug_timezone: String::new(),
        }
    }
}

/// Picks which of "today"'s and "tomorrow"'s max value governs the table
/// display, treating "no data in that window" as absent rather than a
/// fallback zero. Mirrors the pre-existing tie-breaking rule (ties, and
/// today having no data, both favor tomorrow — shown in italics).
///
/// Returns `(value, is_from_tomorrow)`, or `None` if neither window has data.
fn pick_today_or_tomorrow_max<V: PartialOrd>(
    today: Option<V>,
    tomorrow: Option<V>,
) -> Option<(V, bool)> {
    match (today, tomorrow) {
        (Some(t), Some(tm)) if t > tm => Some((t, false)),
        (Some(_), Some(tm)) => Some((tm, true)),
        (Some(t), None) => Some((t, false)),
        (None, Some(tm)) => Some((tm, true)),
        (None, None) => None,
    }
}

pub struct ContextBuilder<'a> {
    settings: &'a DashboardSettings,
    icon_ctx: IconContext<'a>,
    pub context: Context,
    diagnostics: Vec<DashboardError>,
}

impl<'a> ContextBuilder<'a> {
    pub fn new(settings: &'a DashboardSettings, clock: &dyn Clock) -> Self {
        let today = clock.now_local(settings.misc.timezone).date_naive();
        let icon_ctx = IconContext::from_settings(settings, today);
        let mut context = Context::new(settings, today);

        if settings.dev.enable_debug_logs {
            context.debug_info_visibility = ElementVisibility::Visible.to_string();
            context.debug_version = format!("v{}", env!("CARGO_PKG_VERSION"));

            context.debug_provider = settings.api.provider.to_string();

            // Location with reduced precision for privacy (1 decimal place ≈ 11km accuracy)
            let lat = settings.api.latitude.into_inner();
            let lon = settings.api.longitude.into_inner();
            context.debug_location = format!("{:.1}, {:.1}", lat, lon);

            // Timezone offset (e.g., "+11:00" or "-05:00")
            let now = clock.now_local(settings.misc.timezone);
            context.debug_timezone = now.format("%z").to_string();
        }

        Self {
            settings,
            icon_ctx,
            context,
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
                    error.icon_path(&self.icon_ctx)
                )
            })
            .collect::<Vec<String>>()
            .join("\n        ")
    }

    /// Defines the 7-day forecast window starting from today.
    /// Returns a vector of NaiveDate representing [today, today+1, ..., today+6]
    fn define_daily_forecast_window(today: NaiveDate) -> Vec<NaiveDate> {
        (0..7)
            .map(|offset| today + chrono::Days::new(offset))
            .collect()
    }

    /// Builds a HashMap mapping NaiveDate to DailyForecast references.
    /// Skips forecasts with None dates.
    fn build_date_to_forecast_map(
        daily_forecast_data: &[DailyForecast],
    ) -> HashMap<NaiveDate, &DailyForecast> {
        daily_forecast_data
            .iter()
            .filter_map(|forecast| {
                // Date is already NaiveDate - no conversion needed
                forecast.date.map(|date| (date, forecast))
            })
            .collect()
    }

    /// Assigns daily forecast data to the appropriate context fields.
    /// Handles missing data by setting "N/A" defaults.
    fn assign_day_data(&mut self, day_index: i32, forecast: Option<&DailyForecast>) {
        let min_temp_value = forecast
            .and_then(|f| f.temp_min)
            .map_or(NOT_AVAILABLE.to_string(), |temp| temp.to_string());
        let max_temp_value = forecast
            .and_then(|f| f.temp_max)
            .map_or(NOT_AVAILABLE.to_string(), |temp| temp.to_string());
        let icon_value = forecast.map_or_else(
            || {
                not_available_icon_path(self.settings)
                    .to_string_lossy()
                    .to_string()
            },
            |f| f.icon_path(&self.icon_ctx),
        );

        match day_index {
            0 => {
                // Day 0 (today) - show sunrise/sunset times
                if let Some(forecast) = forecast {
                    if let Some(ref astro) = forecast.astronomical {
                        // Sunrise/sunset are NaiveDateTime (already in local time)
                        // Format directly without timezone conversion
                        self.context.sunrise_time = astro
                            .sunrise_time
                            .map(|dt| dt.format("%H:%M").to_string())
                            .unwrap_or_else(|| NOT_AVAILABLE.to_string());
                        self.context.sunset_time = astro
                            .sunset_time
                            .map(|dt| dt.format("%H:%M").to_string())
                            .unwrap_or_else(|| NOT_AVAILABLE.to_string());
                    }
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
    }

    pub fn with_daily_forecast_data(
        &mut self,
        daily_forecast_data: Vec<DailyForecast>,
        clock: &dyn Clock,
    ) -> &mut Self {
        // Get today's local date for comparison
        let today_local_date = clock.now_local(self.settings.misc.timezone).date_naive();

        logger::detail(format!(
            "Processing daily forecast starting from: {today_local_date}"
        ));

        // Pre-populate day names from local calendar (tomorrow through +6 days)
        self.initialize_day_names(clock.now_local(self.settings.misc.timezone));

        // Define the 7-day forecast window (today through +6 days)
        let forecast_window = Self::define_daily_forecast_window(today_local_date);

        let forecast_map = Self::build_date_to_forecast_map(&daily_forecast_data);

        // Track how many days are missing
        let mut missing_days_count = 0;

        // Iterate over expected window dates and map to forecasts
        for (day_index, expected_date) in forecast_window.iter().enumerate() {
            let forecast = forecast_map.get(expected_date);

            if forecast.is_none() {
                missing_days_count += 1;
                logger::warning(format!(
                    "Missing daily forecast for date: {} (day_index: {})",
                    expected_date, day_index
                ));
            }

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

            if let Some(day) = forecast {
                let min_temp = day
                    .temp_min
                    .map_or(NOT_AVAILABLE.to_string(), |t| t.to_string());
                let max_temp = day
                    .temp_max
                    .map_or(NOT_AVAILABLE.to_string(), |t| t.to_string());
                logger::detail(format!(
                    "{day_name} ({expected_date}) - Max {max_temp}°, Min {min_temp}°"
                ));
            } else {
                logger::detail(format!("{day_name} ({expected_date}) - No data available"));
            }

            // Assign data (handles missing data with "N/A" defaults)
            self.assign_day_data(day_index as i32, forecast.copied());
        }

        // Raise single IncompleteData error if any days are missing
        if missing_days_count > 0 {
            let details = format!(
                "Missing {} day(s) of daily forecast data, using incomplete data",
                missing_days_count
            );
            self.with_validation_error(DashboardError::IncompleteData { details })
        } else {
            self
        }
    }

    fn initialize_day_names(&mut self, local_midnight_time: DateTime<Tz>) {
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

        let local_forecast_window_start: DateTime<Tz> =
            utc_forecast_window_start.with_timezone(&self.settings.misc.timezone);
        let local_forecast_window_end: DateTime<Tz> =
            utc_forecast_window_end.with_timezone(&self.settings.misc.timezone);
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
            x_axis_always_at_min: self.settings.render_options.x_axis_always_at_min,
            text_colour: self.settings.colours.text_colour.to_string(),
            background_colour: self.settings.colours.background_colour.to_string(),
            tz: self.settings.misc.timezone,
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
        let (temp_curve_data, feel_like_curve_data, rain_curve_data) = Self::extract_curve_data(
            &svg_result,
            &self.context.rain_colour,
            &self.context.snow_colour,
            graph.height,
            self.settings
                .render_options
                .precipitation_opacity_min
                .into_inner(),
            self.settings
                .render_options
                .precipitation_opacity_max
                .into_inner(),
        );
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

        self.context.total_rain_today = (total_between_dates(
            &hourly_forecast_data,
            &local_forecast_window_start,
            &local_forecast_window_end,
            |item: &HourlyForecast| item.precipitation.amount(),
            |item| item.time.with_timezone(&self.settings.misc.timezone),
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
        let today_utc_date = current_date.date_naive();

        logger::detail(format!(
            "Current hour (UTC): {} (date: {})",
            current_date.format("%Y-%m-%d %H:%M"),
            today_utc_date
        ));

        let first_date = hourly_forecast_data.iter().find_map(|forecast| {
            if forecast.time >= current_date {
                Some(forecast.time)
            } else {
                None
            }
        });

        if let Some(forecast_window_start) = first_date {
            // Validate that the first forecast is actually from today (not tomorrow)
            let forecast_date = forecast_window_start.date_naive();
            if forecast_date != today_utc_date {
                logger::warning(format!(
                    "First available forecast is from {} but expected {}",
                    forecast_date, today_utc_date
                ));
                return None;
            }

            let forecast_window_end = forecast_window_start + chrono::Duration::hours(24);
            Some((forecast_window_start, forecast_window_end))
        } else {
            None
        }
    }

    fn extract_curve_data(
        svg_result: &[GraphDataPath],
        rain_colour: &str,
        snow_colour: &str,
        graph_height: f32,
        opacity_min: f32,
        opacity_max: f32,
    ) -> (String, String, String) {
        svg_result.iter().fold(
            (String::new(), String::new(), String::new()),
            |(mut temp_acc, mut feel_like_acc, mut rain_acc), path| {
                match path {
                    GraphDataPath::Temp(data) => temp_acc.push_str(data),
                    GraphDataPath::TempFeelLike(data) => feel_like_acc.push_str(data),
                    GraphDataPath::Precipitation(blocks) => {
                        rain_acc.push_str(&generate_unified_precipitation_svg(
                            blocks,
                            rain_colour,
                            snow_colour,
                            graph_height,
                            opacity_min,
                            opacity_max,
                        ));
                    }
                }
                (temp_acc, feel_like_acc, rain_acc)
            },
        )
    }

    fn populate_graph_data(
        &mut self,
        hourly_forecast_data: &[HourlyForecast],
        forecast_window_start: chrono::DateTime<Tz>,
        forecast_window_end: chrono::DateTime<Tz>,
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
                    self.populate_current_hour_table(forecast)
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
                        CurveType::PrecipitationChance(curve) => curve.add_point(
                            x as f32,
                            forecast.precipitation.chance.unwrap_or(0) as f32,
                            forecast.precipitation.is_primarily_snow(),
                        ),
                    }
                }
                graph.uv_data[x] = forecast.uv_index;

                let chance = forecast.precipitation.chance.unwrap_or(0);
                let precip_mm = forecast.precipitation.amount();
                let is_snow = forecast.precipitation.is_primarily_snow();
                let pattern = HourlyForecastGraph::select_precipitation_pattern(is_snow);
                logger::debug(format!(
                    "h{:02}: temp={:>5.1}° feels={:>5.1}° precip={:>3}% {:>5.2}mm  uv={:>2}  snow={:<5}  → {}",
                    x,
                    *forecast.temperature,
                    *forecast.apparent_temperature,
                    chance,
                    precip_mm,
                    forecast.uv_index,
                    is_snow,
                    pattern,
                ));

                x += 1;
            });
    }

    fn with_current_hour_data(
        &mut self,
        current_hour: &HourlyForecast,
        clock: &dyn Clock,
    ) -> &mut Self {
        self.context.current_hour_actual_temp = current_hour.temperature.to_string();
        self.context.current_hour_weather_icon = current_hour.icon_path(&self.icon_ctx);
        self.context.current_hour_feels_like = current_hour.apparent_temperature.to_string();
        self.context.current_day_date = clock
            .now_local(self.settings.misc.timezone)
            .format(self.settings.render_options.date_format.as_ref())
            .to_string();
        self.context.current_hour_rain_amount = current_hour.precipitation.amount().to_string();

        self
    }

    fn populate_current_hour_table(&mut self, current_hour: &HourlyForecast) {
        self.context.current_hour_wind_speed = current_hour
            .wind
            .speed_in_unit(
                self.settings.render_options.use_gust_instead_of_wind,
                self.settings.render_options.wind_speed_unit,
            )
            .to_string();
        self.context.current_hour_wind_icon = current_hour.wind.icon_path(&self.icon_ctx);
        self.context.current_hour_uv_index = current_hour.uv_index.to_string();
        self.context.current_hour_uv_index_icon =
            UVIndexIcon::from(current_hour.uv_index).icon_path(&self.icon_ctx);
        self.context.current_hour_relative_humidity = current_hour.relative_humidity.to_string();
        self.context.current_hour_relative_humidity_icon =
            HumidityIconName::from(current_hour.relative_humidity).icon_path(&self.icon_ctx);
    }

    fn set_max_values_for_table(
        &mut self,
        hourly_forecast_data: &[HourlyForecast],
        forecast_window_start: chrono::DateTime<Tz>,
        day_end: chrono::DateTime<Tz>,
        forecast_window_end: chrono::DateTime<Tz>,
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
                let get_time =
                    |item: &HourlyForecast| item.time.with_timezone(&self.settings.misc.timezone);
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
            .speed(self.settings.render_options.use_gust_instead_of_wind));

        match pick_today_or_tomorrow_max(max_wind_today, max_wind_tomorrow) {
            Some((value, is_tomorrow)) => {
                let converted = crate::domain::models::Wind::convert_speed(
                    value,
                    self.settings.render_options.wind_speed_unit,
                );
                self.context.max_gust_speed = converted.to_string();
                if is_tomorrow {
                    self.context.max_gust_speed_font_style = FontStyle::Italic.to_string();
                }
            }
            None => self.context.max_gust_speed = NOT_AVAILABLE.to_string(),
        }

        let (max_uv_index_today, max_uv_index_tomorrow) =
            max_in_today_and_tomorrow!(|item| item.uv_index);

        match pick_today_or_tomorrow_max(max_uv_index_today, max_uv_index_tomorrow) {
            Some((value, is_tomorrow)) => {
                self.context.max_uv_index = value.to_string();
                if is_tomorrow {
                    self.context.max_uv_index_font_style = FontStyle::Italic.to_string();
                }
            }
            None => self.context.max_uv_index = NOT_AVAILABLE.to_string(),
        }

        let (max_relative_humidity_today, max_relative_humidity_tomorrow) =
            max_in_today_and_tomorrow!(|item| item.relative_humidity);

        match pick_today_or_tomorrow_max(
            max_relative_humidity_today,
            max_relative_humidity_tomorrow,
        ) {
            Some((value, is_tomorrow)) => {
                self.context.max_relative_humidity = value.to_string();
                if is_tomorrow {
                    self.context.max_relative_humidity_font_style = FontStyle::Italic.to_string();
                }
            }
            None => self.context.max_relative_humidity = NOT_AVAILABLE.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::FixedClock;
    use crate::domain::models::{Astronomical, Temperature};

    mod pick_today_or_tomorrow_max_tests {
        use super::*;

        #[test]
        fn today_greater_than_tomorrow_picks_today() {
            assert_eq!(
                pick_today_or_tomorrow_max(Some(10), Some(5)),
                Some((10, false))
            );
        }

        #[test]
        fn tomorrow_greater_than_today_picks_tomorrow() {
            assert_eq!(
                pick_today_or_tomorrow_max(Some(5), Some(10)),
                Some((10, true))
            );
        }

        #[test]
        fn tie_favors_tomorrow() {
            assert_eq!(
                pick_today_or_tomorrow_max(Some(7), Some(7)),
                Some((7, true))
            );
        }

        #[test]
        fn only_today_has_data() {
            assert_eq!(
                pick_today_or_tomorrow_max(Some(3), None::<u16>),
                Some((3, false))
            );
        }

        #[test]
        fn only_tomorrow_has_data() {
            assert_eq!(
                pick_today_or_tomorrow_max(None::<u16>, Some(3)),
                Some((3, true))
            );
        }

        #[test]
        fn neither_has_data() {
            assert_eq!(pick_today_or_tomorrow_max(None::<u16>, None::<u16>), None);
        }
    }

    /// Regression test for Issue #16: forecast date names were computed from
    /// UTC timestamps, so converting to local time could shift the date to
    /// the previous/next day and leave day 7 unpopulated ("NA").
    mod issue_16_seventh_day_regression {
        use super::*;

        /// 7 days of mock forecast data starting from `start_date` (local calendar days).
        fn mock_daily_forecast(start_date: NaiveDate, num_days: usize) -> Vec<DailyForecast> {
            (0..num_days)
                .map(|i| {
                    let date = start_date + chrono::Days::new(i as u64);
                    let naive_datetime = date.and_hms_opt(6, 30, 0).unwrap();
                    DailyForecast {
                        date: Some(date),
                        temp_max: Some(Temperature::celsius(20.0 + i as f32)),
                        temp_min: Some(Temperature::celsius(10.0 + i as f32)),
                        precipitation: None,
                        astronomical: Some(Astronomical {
                            sunrise_time: Some(naive_datetime),
                            sunset_time: Some(naive_datetime),
                        }),
                        cloud_cover: None,
                        weather_code: None,
                    }
                })
                .collect()
        }

        /// Clock: Oct 26, 2025, 9:00 AM Melbourne (UTC+11) = Oct 25, 2025, 22:00 UTC.
        /// Forecast data: 7 days starting from Oct 26 (today) as NaiveDate values.
        /// day_index 0 = today (Oct 26, sunrise/sunset only); day_index 1-6 fill
        /// day2-day7 with temp/icon data from Oct 27-Nov 1.
        #[test]
        fn all_seven_days_are_populated() {
            let clock = FixedClock::from_rfc3339("2025-10-25T22:00:00Z")
                .expect("failed to create fixed clock");
            let start_date = NaiveDate::from_ymd_opt(2025, 10, 26).unwrap();
            let daily_forecast_data = mock_daily_forecast(start_date, 7);

            let settings = DashboardSettings::load_test_config().unwrap();
            let mut builder = ContextBuilder::new(&settings, &clock);
            builder.with_daily_forecast_data(daily_forecast_data, &clock);
            let context = &builder.context;

            assert_eq!(context.day2_name, "Mon", "day 2 should be Monday (Oct 27)");
            assert_eq!(context.day2_mintemp, "11");
            assert_eq!(context.day2_maxtemp, "21");

            assert_eq!(context.day3_name, "Tue", "day 3 should be Tuesday (Oct 28)");
            assert_eq!(context.day3_mintemp, "12");
            assert_eq!(context.day3_maxtemp, "22");

            assert_eq!(
                context.day4_name, "Wed",
                "day 4 should be Wednesday (Oct 29)"
            );
            assert_eq!(context.day4_mintemp, "13");
            assert_eq!(context.day4_maxtemp, "23");

            assert_eq!(
                context.day5_name, "Thu",
                "day 5 should be Thursday (Oct 30)"
            );
            assert_eq!(context.day5_mintemp, "14");
            assert_eq!(context.day5_maxtemp, "24");

            assert_eq!(context.day6_name, "Fri", "day 6 should be Friday (Oct 31)");
            assert_eq!(context.day6_mintemp, "15");
            assert_eq!(context.day6_maxtemp, "25");

            assert_eq!(context.day7_name, "Sat", "day 7 should be Saturday (Nov 1)");
            assert_eq!(context.day7_mintemp, "16");
            assert_eq!(context.day7_maxtemp, "26");

            assert_ne!(
                context.day7_name, "NA",
                "day 7 name is 'NA' - timezone bug is present"
            );
        }
    }

    /// Tests for the multi-error priority system: when multiple errors/warnings
    /// occur, the highest priority error is displayed
    /// (ApiError > NetworkError > IncompleteData, per `DashboardError::priority`).
    mod error_priority_display {
        use super::*;
        use crate::dashboard::chart::ElementVisibility;
        use chrono::{TimeZone, Utc};

        /// `ContextBuilder::new` requires a clock, but only reads it when
        /// `dev.enable_debug_logs` is true (off by default in test config) —
        /// tests below that don't otherwise need a clock use this placeholder.
        fn placeholder_clock() -> FixedClock {
            FixedClock::new(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap())
        }

        #[test]
        fn single_validation_error_displays() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let clock = placeholder_clock();
            let mut builder = ContextBuilder::new(&settings, &clock);

            builder.with_validation_error(DashboardError::IncompleteData {
                details: "Only 5 days available".to_string(),
            });

            let context = builder.context;
            assert_eq!(context.diagnostic_message, "Incomplete Data");
            assert_eq!(
                context.diagnostic_visibility,
                ElementVisibility::Visible.to_string()
            );
            assert!(context.diagnostic_icons_svg.contains("code-yellow.svg"));
        }

        #[test]
        fn single_api_warning_displays() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let clock = placeholder_clock();
            let mut builder = ContextBuilder::new(&settings, &clock);

            builder.with_warning(DashboardError::NetworkError {
                details: "Using cached data".to_string(),
            });

            let context = builder.context;
            assert_eq!(context.diagnostic_message, "API unreachable -> Stale Data");
            assert_eq!(
                context.diagnostic_visibility,
                ElementVisibility::Visible.to_string()
            );
            assert!(context.diagnostic_icons_svg.contains("code-orange.svg"));
        }

        #[test]
        fn high_priority_api_error_overrides_low_priority_incomplete_data() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let clock = placeholder_clock();
            let mut builder = ContextBuilder::new(&settings, &clock);

            builder.with_validation_error(DashboardError::IncompleteData {
                details: "Only 5 days available".to_string(),
            });
            builder.with_warning(DashboardError::ApiError {
                details: "Server returned error 500".to_string(),
            });

            let context = builder.context;

            assert_eq!(context.diagnostic_message, "API error -> Stale Data");
            assert!(
                context.diagnostic_icons_svg.contains("code-red.svg"),
                "expected red icon for ApiError in cascading SVG"
            );
            assert_eq!(
                context.diagnostic_visibility,
                ElementVisibility::Visible.to_string()
            );
        }

        #[test]
        fn medium_priority_overrides_low_priority() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let clock = placeholder_clock();
            let mut builder = ContextBuilder::new(&settings, &clock);

            builder.with_validation_error(DashboardError::IncompleteData {
                details: "Only 5 days available".to_string(),
            });
            builder.with_warning(DashboardError::NetworkError {
                details: "Using cached data".to_string(),
            });

            let context = builder.context;

            assert_eq!(context.diagnostic_message, "API unreachable -> Stale Data");
            assert!(
                context.diagnostic_icons_svg.contains("code-orange.svg"),
                "expected orange icon for NetworkError in cascading SVG"
            );
        }

        #[test]
        fn order_doesnt_matter_highest_priority_wins() {
            let clock = placeholder_clock();
            let settings1 = DashboardSettings::load_test_config().unwrap();
            let mut builder1 = ContextBuilder::new(&settings1, &clock);
            let settings2 = DashboardSettings::load_test_config().unwrap();
            let mut builder2 = ContextBuilder::new(&settings2, &clock);

            builder1.with_validation_error(DashboardError::IncompleteData {
                details: "Issue 1".to_string(),
            });
            builder1.with_warning(DashboardError::ApiError {
                details: "Issue 2".to_string(),
            });

            builder2.with_warning(DashboardError::ApiError {
                details: "Issue 2".to_string(),
            });
            builder2.with_validation_error(DashboardError::IncompleteData {
                details: "Issue 1".to_string(),
            });

            assert_eq!(
                builder1.context.diagnostic_message,
                builder2.context.diagnostic_message
            );
            assert_eq!(
                builder1.context.diagnostic_icons_svg,
                builder2.context.diagnostic_icons_svg
            );
            assert!(builder1
                .context
                .diagnostic_icons_svg
                .contains("code-red.svg"));
        }

        #[test]
        fn multiple_errors_same_priority_shows_first() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let clock = placeholder_clock();
            let mut builder = ContextBuilder::new(&settings, &clock);

            builder.with_validation_error(DashboardError::IncompleteData {
                details: "Issue 1".to_string(),
            });
            builder.with_validation_error(DashboardError::IncompleteData {
                details: "Issue 2".to_string(),
            });

            let context = builder.context;

            assert_eq!(context.diagnostic_message, "Incomplete Data");
            assert!(context.diagnostic_icons_svg.contains("code-yellow.svg"));
        }

        /// Real-world scenario: API is unreachable (medium priority), so cached
        /// data is used, and that cached data happens to be incomplete (low
        /// priority) — the higher-priority API warning should win.
        #[test]
        fn realistic_scenario_api_stale_and_incomplete_data() {
            let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 10, 15, 10, 0, 0).unwrap());
            let settings = DashboardSettings::load_test_config().unwrap();
            let mut builder = ContextBuilder::new(&settings, &clock);

            builder.with_warning(DashboardError::NetworkError {
                details: "Could not reach API server".to_string(),
            });

            let incomplete_daily_data: Vec<DailyForecast> = vec![]; // only 3 days instead of 7
            builder.with_daily_forecast_data(incomplete_daily_data, &clock);

            let context = builder.context;

            assert_eq!(context.diagnostic_message, "API unreachable -> Stale Data");
            assert!(context.diagnostic_icons_svg.contains("code-orange.svg"));
        }

        #[test]
        fn no_errors_hides_warning_display() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let clock = placeholder_clock();
            let builder = ContextBuilder::new(&settings, &clock);
            let context = builder.context;

            assert_eq!(
                context.diagnostic_visibility,
                ElementVisibility::Hidden.to_string()
            );
        }

        #[test]
        fn cascading_icons_svg_generated_for_multiple_errors() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let clock = placeholder_clock();
            let mut builder = ContextBuilder::new(&settings, &clock);

            builder.with_warning(DashboardError::NetworkError {
                details: "Network issue".to_string(),
            });
            builder.with_validation_error(DashboardError::IncompleteData {
                details: "Missing days".to_string(),
            });
            builder.with_warning(DashboardError::ApiError {
                details: "Server error".to_string(),
            });

            let context = builder.context;

            assert_eq!(context.diagnostic_message, "API error -> Stale Data");
            assert!(context.diagnostic_icons_svg.contains("code-red.svg"));
            assert!(context.diagnostic_icons_svg.contains("code-orange.svg"));
            assert!(context.diagnostic_icons_svg.contains("code-yellow.svg"));

            let image_count = context.diagnostic_icons_svg.matches("<image").count();
            assert_eq!(image_count, 3, "should have 3 image tags for 3 diagnostics");
        }

        /// Icons should appear in reverse-priority order in the SVG markup
        /// (lowest priority first) so the lowest priority renders in back and
        /// the highest priority renders last, appearing in front.
        #[test]
        fn cascading_icons_are_sorted_by_priority() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let clock = placeholder_clock();
            let mut builder = ContextBuilder::new(&settings, &clock);

            builder.with_validation_error(DashboardError::IncompleteData {
                details: "Issue 1".to_string(),
            });
            builder.with_warning(DashboardError::NetworkError {
                details: "Issue 2".to_string(),
            });
            builder.with_warning(DashboardError::ApiError {
                details: "Issue 3".to_string(),
            });

            let context = builder.context;
            let svg = &context.diagnostic_icons_svg;
            let red_pos = svg.find("code-red.svg").unwrap();
            let orange_pos = svg.find("code-orange.svg").unwrap();
            let yellow_pos = svg.find("code-yellow.svg").unwrap();

            assert!(
                yellow_pos < orange_pos,
                "yellow should render before orange"
            );
            assert!(orange_pos < red_pos, "orange should render before red");
        }

        #[test]
        fn single_error_shows_one_icon() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let clock = placeholder_clock();
            let mut builder = ContextBuilder::new(&settings, &clock);

            builder.with_validation_error(DashboardError::IncompleteData {
                details: "Only issue".to_string(),
            });

            let context = builder.context;
            let image_count = context.diagnostic_icons_svg.matches("<image").count();
            assert_eq!(image_count, 1, "should have 1 image tag for 1 diagnostic");
            assert!(context.diagnostic_icons_svg.contains("code-yellow.svg"));
        }
    }

    /// `with_daily_forecast_data` in a negative-UTC-offset timezone (New York,
    /// EST/UTC-5): verifies noon-UTC forecast timestamps map to the correct
    /// local date without shifting to the previous day.
    mod new_york_timezone_daily_forecast {
        use super::*;
        use crate::configs::settings::TemperatureUnit;
        use crate::domain::models::{Astronomical, Precipitation};
        use chrono::{NaiveDateTime, TimeZone};

        fn temp_c(value: f32) -> Temperature {
            Temperature::new(value, TemperatureUnit::C)
        }

        fn ny_test_settings() -> DashboardSettings {
            let mut settings = DashboardSettings::load_test_config().unwrap();
            settings.misc.timezone = chrono_tz::America::New_York;
            settings
        }

        /// Current time: Dec 17, 2025 at 10:00 AM EST.
        #[test]
        fn with_daily_forecast_data_new_york_est() {
            let settings = ny_test_settings();
            let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 12, 17, 15, 0, 0).unwrap());

            // Dec 17-23, 2025: Wed, Thu, Fri, Sat, Sun, Mon, Tue
            let daily_forecasts = vec![
                DailyForecast {
                    date: Some(NaiveDate::from_ymd_opt(2025, 12, 17).unwrap()),
                    temp_max: Some(temp_c(9.9)),
                    temp_min: Some(temp_c(-2.8)),
                    precipitation: Some(Precipitation::new(Some(10), None, Some(0))),
                    astronomical: Some(Astronomical {
                        // 12:19 UTC = 7:19 AM EST, 21:33 UTC = 4:33 PM EST
                        sunrise_time: Some(
                            NaiveDateTime::parse_from_str(
                                "2025-12-17 07:19:00",
                                "%Y-%m-%d %H:%M:%S",
                            )
                            .unwrap(),
                        ),
                        sunset_time: Some(
                            NaiveDateTime::parse_from_str(
                                "2025-12-17 16:33:00",
                                "%Y-%m-%d %H:%M:%S",
                            )
                            .unwrap(),
                        ),
                    }),
                    cloud_cover: None,
                    weather_code: None,
                },
                DailyForecast {
                    date: Some(NaiveDate::from_ymd_opt(2025, 12, 18).unwrap()),
                    temp_max: Some(temp_c(10.3)),
                    temp_min: Some(temp_c(-1.2)),
                    precipitation: Some(Precipitation::new(Some(30), None, Some(1))),
                    astronomical: None,
                    cloud_cover: None,
                    weather_code: None,
                },
                DailyForecast {
                    date: Some(NaiveDate::from_ymd_opt(2025, 12, 19).unwrap()),
                    temp_max: Some(temp_c(11.5)),
                    temp_min: Some(temp_c(1.9)),
                    precipitation: Some(Precipitation::new(Some(50), None, Some(2))),
                    astronomical: None,
                    cloud_cover: None,
                    weather_code: None,
                },
                DailyForecast {
                    date: Some(NaiveDate::from_ymd_opt(2025, 12, 20).unwrap()),
                    temp_max: Some(temp_c(2.2)),
                    temp_min: Some(temp_c(-1.1)),
                    precipitation: None,
                    astronomical: None,
                    cloud_cover: None,
                    weather_code: None,
                },
                DailyForecast {
                    date: Some(NaiveDate::from_ymd_opt(2025, 12, 21).unwrap()),
                    temp_max: Some(temp_c(7.2)),
                    temp_min: Some(temp_c(1.7)),
                    precipitation: None,
                    astronomical: None,
                    cloud_cover: None,
                    weather_code: None,
                },
                DailyForecast {
                    date: Some(NaiveDate::from_ymd_opt(2025, 12, 22).unwrap()),
                    temp_max: Some(temp_c(5.0)),
                    temp_min: Some(temp_c(-1.5)),
                    precipitation: None,
                    astronomical: None,
                    cloud_cover: None,
                    weather_code: None,
                },
                DailyForecast {
                    date: Some(NaiveDate::from_ymd_opt(2025, 12, 23).unwrap()),
                    temp_max: Some(temp_c(1.3)),
                    temp_min: Some(temp_c(-3.0)),
                    precipitation: None,
                    astronomical: None,
                    cloud_cover: None,
                    weather_code: None,
                },
            ];

            let mut builder = ContextBuilder::new(&settings, &clock);
            builder.with_daily_forecast_data(daily_forecasts, &clock);
            let context = builder.context;

            // Dec 18-23: Thu, Fri, Sat, Sun, Mon, Tue
            assert_eq!(context.day2_name, "Thu");
            assert_eq!(context.day3_name, "Fri");
            assert_eq!(context.day4_name, "Sat");
            assert_eq!(context.day5_name, "Sun");
            assert_eq!(context.day6_name, "Mon");
            assert_eq!(context.day7_name, "Tue");

            // Day 0 (today, Dec 17) - only sunrise/sunset used
            assert_eq!(context.sunrise_time, "07:19");
            assert_eq!(context.sunset_time, "16:33");

            assert_eq!(context.day2_maxtemp, "10"); // 10.3°C → 10
            assert_eq!(context.day2_mintemp, "-1"); // -1.2°C → -1
            assert_eq!(context.day3_maxtemp, "12"); // 11.5°C → 12
            assert_eq!(context.day3_mintemp, "2"); // 1.9°C → 2
            assert_eq!(context.day4_maxtemp, "2"); // 2.2°C → 2
            assert_eq!(context.day4_mintemp, "-1"); // -1.1°C → -1
            assert_eq!(context.day5_maxtemp, "7"); // 7.2°C → 7
            assert_eq!(context.day5_mintemp, "2"); // 1.7°C → 2
            assert_eq!(context.day6_maxtemp, "5"); // 5.0°C → 5
            assert_eq!(context.day6_mintemp, "-2"); // -1.5°C → -2
            assert_eq!(context.day7_maxtemp, "1"); // 1.3°C → 1
            assert_eq!(context.day7_mintemp, "-3"); // -3.0°C → -3
        }

        /// Critical case: 2025-12-17T12:00:00Z → 2025-12-17T07:00:00-05:00
        /// (same day) — noon UTC timestamps must not shift the date in EST.
        #[test]
        fn noon_utc_prevents_date_shift_in_est() {
            let settings = ny_test_settings();
            // Fixed clock: Dec 17, 2025 at 2:00 AM EST (early morning)
            let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 12, 17, 7, 0, 0).unwrap());

            let daily_forecasts = vec![
                (17, 10.0, -3.0),
                (18, 11.0, -2.0),
                (19, 12.0, -1.0),
                (20, 13.0, 0.0),
                (21, 14.0, 1.0),
                (22, 15.0, 2.0),
                (23, 16.0, 3.0),
            ]
            .into_iter()
            .map(|(day, temp_max, temp_min)| DailyForecast {
                date: Some(NaiveDate::from_ymd_opt(2025, 12, day).unwrap()),
                temp_max: Some(temp_c(temp_max)),
                temp_min: Some(temp_c(temp_min)),
                precipitation: None,
                astronomical: None,
                cloud_cover: None,
                weather_code: None,
            })
            .collect();

            let mut builder = ContextBuilder::new(&settings, &clock);
            builder.with_daily_forecast_data(daily_forecasts, &clock);
            let context = builder.context;

            // All days should be correctly assigned despite early morning test time
            assert_eq!(context.day2_name, "Thu");
            assert_eq!(context.day2_maxtemp, "11"); // Dec 18 data goes to day2
            assert_eq!(context.day3_maxtemp, "12"); // Dec 19 data goes to day3
            assert_eq!(context.day4_maxtemp, "13"); // Dec 20 data goes to day4
            assert_eq!(context.day5_maxtemp, "14"); // Dec 21 data goes to day5
            assert_eq!(context.day6_maxtemp, "15"); // Dec 22 data goes to day6
            assert_eq!(context.day7_maxtemp, "16"); // Dec 23 data goes to day7
        }

        #[test]
        fn skips_past_dates() {
            let settings = ny_test_settings();
            // Fixed clock: Dec 19, 2025 at 10:00 AM EST
            let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 12, 19, 15, 0, 0).unwrap());

            // Includes past dates (Dec 17, 18) which should be skipped
            let daily_forecasts = vec![
                (17, 10.0, -3.0),
                (18, 11.0, -2.0),
                (19, 12.0, -1.0),
                (20, 13.0, 0.0),
                (21, 14.0, 1.0),
                (22, 15.0, 2.0),
                (23, 16.0, 3.0),
                (24, 17.0, 4.0),
                (25, 18.0, 5.0),
            ]
            .into_iter()
            .map(|(day, temp_max, temp_min)| DailyForecast {
                date: Some(NaiveDate::from_ymd_opt(2025, 12, day).unwrap()),
                temp_max: Some(temp_c(temp_max)),
                temp_min: Some(temp_c(temp_min)),
                precipitation: None,
                astronomical: None,
                cloud_cover: None,
                weather_code: None,
            })
            .collect();

            let mut builder = ContextBuilder::new(&settings, &clock);
            builder.with_daily_forecast_data(daily_forecasts, &clock);
            let context = builder.context;

            // Dec 19 is today (day 0), so day2 should be Dec 20 (Sat)
            assert_eq!(context.day2_name, "Sat");
            assert_eq!(context.day2_maxtemp, "13"); // Dec 20 → day2
            assert_eq!(context.day3_maxtemp, "14"); // Dec 21 → day3
            assert_eq!(context.day4_maxtemp, "15"); // Dec 22 → day4
            assert_eq!(context.day5_maxtemp, "16"); // Dec 23 → day5
            assert_eq!(context.day6_maxtemp, "17"); // Dec 24 → day6
            assert_eq!(context.day7_maxtemp, "18"); // Dec 25 → day7
        }
    }
}

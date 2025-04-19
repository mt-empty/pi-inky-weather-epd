use crate::apis::bom::models::*;
use crate::dashboard::context::Context;
use crate::{constants::*, dashboard, errors, utils, weather, CONFIG};
use anyhow::Error;
use chrono::{DateTime, Local, Timelike, Utc};
use dashboard::chart::{DailyForecastGraph, DataType, GraphData, GraphDataPath};
use errors::*;
use serde::Deserialize;
use std::io::Write;
use std::{fs, path::PathBuf};
use tinytemplate::{format_unescaped, TinyTemplate};
pub use utils::*;
use weather::icons::*;

enum FetchOutcome<T> {
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
    // TODO: delegate the Config::debugging.disable_network_requests to a different function
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
        std::path::Path::new(&CONFIG.misc.weather_data_cache_path).join("daily_forecast.json");
    fetch_data(&DAILY_FORECAST_ENDPOINT, &file_path)
}

fn fetch_hourly_forecast_data() -> Result<FetchOutcome<HourlyForecastResponse>, Error> {
    let file_path =
        std::path::Path::new(&CONFIG.misc.weather_data_cache_path).join("hourly_forecast.json");
    fetch_data(&HOURLY_FORECAST_ENDPOINT, &file_path)
}

fn update_current_hour_data(current_hour: &HourlyForecast, context: &mut Context) {
    context.current_hour_temp = current_hour.temp.to_string();
    context.current_hour_weather_icon = current_hour.get_icon_path();
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
            if naive_date < local_date_truncated {
                // If the date is in the past, skip it
                continue;
            } else if naive_date > local_date_truncated + chrono::Duration::days(7) {
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
        match day_index {
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

        day_index += 1;
    }

    if day_index < 8 {
        let details =
            "Warning: Less than 7 days of daily forecast data, Using Incomplete data".to_string();
        handle_errors(context, DashboardError::IncompleteData { details });
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
    let forecast_window_start = match first_date {
        Some(date) => date,
        None => {
            handle_errors(
                context,
                DashboardError::IncompleteData {
                    details: "No hourly forecast data available, Could Not find a date later than the current date".to_string(),
                },
            );
            return Ok(());
        }
    };

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
                update_current_hour_data(forecast, context);
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

    let forecast_window_start_local = forecast_window_start.with_timezone(&chrono::Local);
    let axis_data_path = graph.create_axis_with_labels(forecast_window_start_local.hour() as f64);

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

fn update_forecast_context(context: &mut Context) -> Result<(), Error> {
    println!("## Daily forecast ...");
    match update_daily_forecast_data(context) {
        Ok(context) => context,
        Err(e) => {
            eprintln!("Failed to update daily forecast");
            return Err(e);
        }
    };
    println!("## Hourly forecast ...");
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
            let mut output = fs::File::create(&CONFIG.misc.generated_svg_name)?;
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

    println!("## Rendering dashboard to SVG ...");
    render_dashboard_template(&mut context, template_svg)?;
    println!(
        "SVG has been modified and saved successfully at {}",
        current_dir.join(&CONFIG.misc.generated_svg_name).display()
    );

    if !CONFIG.debugging.disable_png_output {
        println!("## Converting SVG to PNG ...");
        convert_svg_to_png(
            &CONFIG.misc.generated_svg_name,
            &CONFIG.misc.generated_png_name,
            2.0,
        )?;

        println!(
            "PNG has been generated successfully at {}",
            current_dir.join(&CONFIG.misc.generated_png_name).display()
        );
    }
    Ok(())
}

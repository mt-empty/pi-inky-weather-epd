use crate::apis::bom::models::*;
use crate::dashboard::context::{Context, ContextBuilder};
use crate::{constants::*, errors, utils, CONFIG};
use anyhow::Error;
use errors::*;
use serde::Deserialize;
use std::io::Write;
use std::{fs, path::PathBuf};
use tinytemplate::{format_unescaped, TinyTemplate};
use url::Url;
pub use utils::*;

enum FetchOutcome<T> {
    Fresh(T),
    Stale { data: T, error: DashboardError },
}

fn load_cached<T: for<'de> Deserialize<'de>>(file_path: &PathBuf) -> Result<T, Error> {
    let cached = fs::read_to_string(file_path).map_err(|_| {
        Error::msg(
            "Weather data JSON file not found. If this is your first time running the application. Please ensure 'disable_weather_api_requests' is set to false in the configuration so data can be cached.",
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
    endpoint: Url,
    file_path: &PathBuf,
) -> Result<FetchOutcome<T>, Error> {
    if !file_path.exists() {
        fs::create_dir_all(file_path.parent().unwrap())?;
    }
    // TODO: delegate the Config::debugging.disable_weather_api_requests to a different function
    if !CONFIG.debugging.disable_weather_api_requests {
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
    let file_path = CONFIG
        .misc
        .weather_data_cache_path
        .join("daily_forecast.json");
    fetch_data(DAILY_FORECAST_ENDPOINT.clone(), &file_path)
}

fn fetch_hourly_forecast_data() -> Result<FetchOutcome<HourlyForecastResponse>, Error> {
    let file_path = CONFIG
        .misc
        .weather_data_cache_path
        .join("hourly_forecast.json");
    fetch_data(HOURLY_FORECAST_ENDPOINT.clone(), &file_path)
}

fn update_daily_forecast_data(context_builder: &mut ContextBuilder) -> Result<(), Error> {
    match fetch_daily_forecast_data() {
        Ok(FetchOutcome::Fresh(data)) => {
            context_builder.with_daily_forecast_data(data.data);
        }
        Ok(FetchOutcome::Stale { data, error }) => {
            eprintln!("Warning: Using stale data");
            context_builder
                .set_errors(error)
                .with_daily_forecast_data(data.data);
        }
        Err(e) => return Err(e),
    };
    Ok(())
}

fn update_hourly_forecast_data(context_builder: &mut ContextBuilder) -> Result<(), Error> {
    match fetch_hourly_forecast_data() {
        Ok(FetchOutcome::Fresh(data)) => {
            context_builder.with_hourly_forecast_data(data.data);
        }
        Ok(FetchOutcome::Stale { data, error }) => {
            eprintln!("Warning: Using stale data");
            context_builder
                .set_errors(error)
                .with_hourly_forecast_data(data.data);
        }
        Err(e) => return Err(e),
    };

    Ok(())
}

fn update_forecast_context(context_builder: &mut ContextBuilder) -> Result<(), Error> {
    println!("## Daily forecast ...");
    match update_daily_forecast_data(context_builder) {
        Ok(context) => context,
        Err(e) => {
            eprintln!("Failed to update daily forecast");
            return Err(e);
        }
    };
    println!("## Hourly forecast ...");
    match update_hourly_forecast_data(context_builder) {
        Ok(context) => context,
        Err(e) => {
            eprintln!("Failed to update hourly forecast");
            return Err(e);
        }
    };
    Ok(())
}

fn render_dashboard_template(context: &Context, dashboard_svg: String) -> Result<(), Error> {
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
    let mut context_builder = ContextBuilder::new();

    let template_svg = match fs::read_to_string(CONFIG.misc.template_path.clone()) {
        Ok(svg) => svg,
        Err(e) => {
            println!("Current directory: {}", current_dir.display());
            println!("Template path: {}", &CONFIG.misc.template_path.display());
            println!("Failed to read template file: {}", e);
            return Err(e.into());
        }
    };
    update_forecast_context(&mut context_builder)?;

    println!("## Rendering dashboard to SVG ...");
    render_dashboard_template(&context_builder.context, template_svg)?;
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

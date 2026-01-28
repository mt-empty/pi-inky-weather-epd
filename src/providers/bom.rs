use anyhow::Error;
use std::path::PathBuf;

use crate::{
    apis::bom::models::{BomError, DailyForecastResponse, HourlyForecastResponse},
    constants::{
        DAILY_CACHE_SUFFIX, DAILY_FORECAST_ENDPOINT, HOURLY_CACHE_SUFFIX, HOURLY_FORECAST_ENDPOINT,
    },
    domain::models::{DailyForecast, HourlyForecast},
    errors::DashboardError,
    providers::{
        fetcher::{FetchOutcome, Fetcher},
        FetchResult, WeatherProvider,
    },
};

fn check_bom_error(body: &str) -> Result<(), DashboardError> {
    use crate::logger;
    logger::debug("Checking for API errors");
    // Try to parse as error response; if it's not an error format, that's fine (return Ok)
    let api_error = match serde_json::from_str::<BomError>(body) {
        Ok(err) => err,
        Err(_) => return Ok(()), // Not an error response format, continue processing
    };

    // If we have errors, format all of them into a single message
    if !api_error.errors.is_empty() {
        let error_details: Vec<String> = api_error
            .errors
            .iter()
            .enumerate()
            .map(|(i, err)| format!("  {}. {}", i + 1, err.detail))
            .collect();

        let combined_details = if api_error.errors.len() == 1 {
            api_error.errors[0].detail.clone()
        } else {
            format!(
                "BOM API returned {} errors:\n{}",
                api_error.errors.len(),
                error_details.join("\n")
            )
        };

        return Err(DashboardError::ApiError {
            details: combined_details,
        });
    }

    Ok(())
}

pub struct BomProvider {
    fetcher: Fetcher,
}

impl BomProvider {
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            fetcher: Fetcher::new(cache_path),
        }
    }
}

impl WeatherProvider for BomProvider {
    fn fetch_hourly_forecast(&self) -> Result<FetchResult<Vec<HourlyForecast>>, Error> {
        match self.fetcher.fetch_data::<HourlyForecastResponse>(
            HOURLY_FORECAST_ENDPOINT.clone(),
            &self.generate_cache_filename(HOURLY_CACHE_SUFFIX),
            Some(check_bom_error),
        )? {
            FetchOutcome::Fresh(data) => {
                // Convert BOM models to domain models
                let domain_data: Vec<HourlyForecast> =
                    data.data.into_iter().map(|h| h.into()).collect();
                crate::logger::debug(format!(
                    "Converted {} BOM hourly entries to domain model",
                    domain_data.len()
                ));
                Ok(FetchResult::fresh(domain_data))
            }
            FetchOutcome::Stale { data, error } => {
                let domain_data: Vec<HourlyForecast> =
                    data.data.into_iter().map(|h| h.into()).collect();
                Ok(FetchResult::stale(domain_data, error))
            }
        }
    }

    fn fetch_daily_forecast(&self) -> Result<FetchResult<Vec<DailyForecast>>, Error> {
        match self.fetcher.fetch_data::<DailyForecastResponse>(
            DAILY_FORECAST_ENDPOINT.clone(),
            &self.generate_cache_filename(DAILY_CACHE_SUFFIX),
            Some(check_bom_error),
        )? {
            FetchOutcome::Fresh(data) => {
                // Convert BOM models to domain models
                let domain_data: Vec<DailyForecast> =
                    data.data.into_iter().map(|d| d.into()).collect();
                crate::logger::debug(format!(
                    "Converted {} BOM daily entries to domain model",
                    domain_data.len()
                ));
                Ok(FetchResult::fresh(domain_data))
            }
            FetchOutcome::Stale { data, error } => {
                let domain_data: Vec<DailyForecast> =
                    data.data.into_iter().map(|d| d.into()).collect();
                Ok(FetchResult::stale(domain_data, error))
            }
        }
    }

    fn provider_name(&self) -> &str {
        "Bureau of Meteorology (BOM)"
    }

    fn provider_filename_prefix(&self) -> &str {
        "bom_"
    }
}

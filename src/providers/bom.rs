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

/// BOM-specific error checker
fn check_bom_error(body: &str) -> Result<(), DashboardError> {
    // Try to parse as error response; if it's not an error format, that's fine (return Ok)
    let api_error = match serde_json::from_str::<BomError>(body) {
        Ok(err) => err,
        Err(_) => return Ok(()), // Not an error response format, continue processing
    };
    
    // If we have errors, report them and return the first one
    if let Some(first_error) = api_error.errors.first() {
        eprintln!("Warning: BOM API request failed, trying to load cached data");
        for (i, error) in api_error.errors.iter().enumerate() {
            eprintln!("BOM API Error {}: {}", i + 1, error.detail);
        }
        return Err(DashboardError::ApiError(first_error.detail.clone()));
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
        match self
            .fetcher
            .fetch_data::<HourlyForecastResponse>(
                HOURLY_FORECAST_ENDPOINT.clone(),
                &self.generate_cache_filename(HOURLY_CACHE_SUFFIX),
                Some(check_bom_error),
            )? {
            FetchOutcome::Fresh(data) => {
                // Convert BOM models to domain models
                let domain_data: Vec<HourlyForecast> =
                    data.data.into_iter().map(|h| h.into()).collect();
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

use anyhow::Error;
use std::path::PathBuf;

use crate::{
    apis::bom::models::{BomError, DailyForecastResponse, HourlyForecastResponse},
    constants::{DAILY_FORECAST_ENDPOINT, HOURLY_FORECAST_ENDPOINT},
    domain::models::{DailyForecast, HourlyForecast},
    errors::DashboardError,
    providers::{
        fetcher::{FetchOutcome, Fetcher},
        FetchResult, WeatherProvider,
    },
};

/// BOM-specific error checker
fn check_bom_error(body: &str) -> Result<(), DashboardError> {
    if let Ok(api_error) = serde_json::from_str::<BomError>(body) {
        if let Some(first_error) = api_error.errors.first() {
            eprintln!("Warning: BOM API request failed, trying to load cached data");
            for (i, error) in api_error.errors.iter().enumerate() {
                eprintln!("BOM API Error {}: {}", i + 1, error.detail);
            }
            return Err(DashboardError::ApiError(first_error.detail.clone()));
        }
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
            .fetch_data::<HourlyForecastResponse, BomError>(
                HOURLY_FORECAST_ENDPOINT.clone(),
                "hourly_forecast.json",
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
        match self.fetcher.fetch_data::<DailyForecastResponse, BomError>(
            DAILY_FORECAST_ENDPOINT.clone(),
            "daily_forecast.json",
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
}

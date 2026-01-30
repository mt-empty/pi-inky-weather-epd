use anyhow::Error;
use std::path::PathBuf;

use crate::{
    apis::open_meteo::models::{OpenMeteoDailyResponse, OpenMeteoError, OpenMeteoHourlyResponse},
    constants::{
        DAILY_CACHE_SUFFIX, HOURLY_CACHE_SUFFIX, OPEN_METEO_DAILY_ENDPOINT,
        OPEN_METEO_HOURLY_ENDPOINT,
    },
    domain::models::{DailyForecast, HourlyForecast},
    errors::DashboardError,
    providers::{
        fetcher::{FetchOutcome, Fetcher},
        FetchResult, WeatherProvider,
    },
};

fn check_open_meteo_error(body: &str) -> Result<(), DashboardError> {
    use crate::logger;
    logger::debug("Checking for API errors");
    // Try to parse as error response; if it's not an error format, that's fine (return Ok)
    let api_error = match serde_json::from_str::<OpenMeteoError>(body) {
        Ok(err) => err,
        Err(_) => return Ok(()),
    };

    // OpenMeteoError.error field indicates if this is actually an error
    if api_error.error {
        return Err(DashboardError::ApiError {
            details: api_error.reason,
        });
    }

    Ok(())
}

pub struct OpenMeteoProvider {
    fetcher: Fetcher,
}

impl OpenMeteoProvider {
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            fetcher: Fetcher::new(cache_path),
        }
    }
}

impl WeatherProvider for OpenMeteoProvider {
    fn fetch_hourly_forecast(&self) -> Result<FetchResult<Vec<HourlyForecast>>, Error> {
        let result = match self.fetcher.fetch_data::<OpenMeteoHourlyResponse>(
            OPEN_METEO_HOURLY_ENDPOINT.clone(),
            &self.generate_cache_filename(HOURLY_CACHE_SUFFIX),
            Some(check_open_meteo_error),
        )? {
            FetchOutcome::Fresh(data) => FetchResult::fresh(data.into()),
            FetchOutcome::Stale { data, error } => FetchResult::stale(data.into(), error),
        };

        Ok(result)
    }

    fn fetch_daily_forecast(&self) -> Result<FetchResult<Vec<DailyForecast>>, Error> {
        let result = match self.fetcher.fetch_data::<OpenMeteoDailyResponse>(
            OPEN_METEO_DAILY_ENDPOINT.clone(),
            &self.generate_cache_filename(DAILY_CACHE_SUFFIX),
            Some(check_open_meteo_error),
        )? {
            FetchOutcome::Fresh(data) => FetchResult::fresh(data.into()),
            FetchOutcome::Stale { data, error } => FetchResult::stale(data.into(), error),
        };

        Ok(result)
    }

    fn provider_name(&self) -> &str {
        "Open-Meteo"
    }
    fn provider_filename_prefix(&self) -> &str {
        "open_meteo_"
    }
}

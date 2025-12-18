use anyhow::Error;
use std::cell::RefCell;
use std::path::PathBuf;

use crate::{
    apis::open_meteo::models::{OpenMeteoError, OpenMeteoHourlyResponse},
    constants::{CACHE_SUFFIX, OPEN_METEO_ENDPOINT},
    domain::models::{DailyForecast, HourlyForecast},
    errors::DashboardError,
    providers::{
        fetcher::{FetchOutcome, Fetcher},
        FetchResult, WeatherProvider,
    },
};

/// Open-Meteo-specific error checker
fn check_open_meteo_error(body: &str) -> Result<(), DashboardError> {
    use crate::logger;
    logger::debug("Checking for API errors");
    // Try to parse as error response; if it's not an error format, that's fine (return Ok)
    let api_error = match serde_json::from_str::<OpenMeteoError>(body) {
        Ok(err) => err,
        Err(_) => return Ok(()), // Not an error response format, continue processing
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
    cached_response: RefCell<Option<OpenMeteoHourlyResponse>>,
}

impl OpenMeteoProvider {
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            fetcher: Fetcher::new(cache_path),
            cached_response: RefCell::new(None),
        }
    }
    pub fn fetch_response(&self) -> Result<FetchResult<OpenMeteoHourlyResponse>, Error> {
        if let Some(cached) = self.cached_response.borrow().as_ref() {
            return Ok(FetchResult::fresh(cached.clone()));
        }

        let result = match self.fetcher.fetch_data::<OpenMeteoHourlyResponse>(
            OPEN_METEO_ENDPOINT.clone(),
            &self.generate_cache_filename(CACHE_SUFFIX),
            Some(check_open_meteo_error),
        )? {
            FetchOutcome::Fresh(data) => {
                self.cached_response.borrow_mut().replace(data.clone());
                FetchResult::fresh(data)
            }
            FetchOutcome::Stale { data, error } => {
                self.cached_response.borrow_mut().replace(data.clone());
                FetchResult::stale(data, error)
            }
        };

        Ok(result)
    }
}

impl WeatherProvider for OpenMeteoProvider {
    fn fetch_hourly_forecast(&self) -> Result<FetchResult<Vec<HourlyForecast>>, Error> {
        Ok(self.fetch_response()?.map(|response| response.into()))
    }

    fn fetch_daily_forecast(&self) -> Result<FetchResult<Vec<DailyForecast>>, Error> {
        Ok(self.fetch_response()?.map(|response| response.into()))
    }

    fn provider_name(&self) -> &str {
        "Open-Meteo"
    }
    fn provider_filename_prefix(&self) -> &str {
        "open_meteo_"
    }
}

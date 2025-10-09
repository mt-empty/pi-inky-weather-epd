use anyhow::Error;
use std::path::PathBuf;

use crate::{
    apis::open_metro::models::OpenMeteoHourlyResponse,
    constants::{DAILY_CACHE_SUFFIX, HOURLY_CACHE_SUFFIX, OPEN_METEO_ENDPOINT},
    domain::models::{DailyForecast, HourlyForecast},
    providers::{
        fetcher::{FetchOutcome, Fetcher},
        FetchResult, WeatherProvider,
    },
};

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
        // OpenMeteo doesn't have custom error format, use None for error_checker
        match self.fetcher.fetch_data::<OpenMeteoHourlyResponse, ()>(
            OPEN_METEO_ENDPOINT.clone(),
            &self.get_cache_filename(HOURLY_CACHE_SUFFIX),
            None,
        )? {
            FetchOutcome::Fresh(data) => {
                let domain_data: Vec<HourlyForecast> = data.into();
                Ok(FetchResult::fresh(domain_data))
            }
            FetchOutcome::Stale { data, error } => {
                let domain_data: Vec<HourlyForecast> = data.into();
                Ok(FetchResult::stale(domain_data, error))
            }
        }
    }

    fn fetch_daily_forecast(&self) -> Result<FetchResult<Vec<DailyForecast>>, Error> {
        // OpenMeteo doesn't have custom error format, use None for error_checker
        match self.fetcher.fetch_data::<OpenMeteoHourlyResponse, ()>(
            OPEN_METEO_ENDPOINT.clone(),
            &self.get_cache_filename(DAILY_CACHE_SUFFIX),
            None,
        )? {
            FetchOutcome::Fresh(data) => {
                let domain_data: Vec<DailyForecast> = data.into();
                Ok(FetchResult::fresh(domain_data))
            }
            FetchOutcome::Stale { data, error } => {
                let domain_data: Vec<DailyForecast> = data.into();
                Ok(FetchResult::stale(domain_data, error))
            }
        }
    }

    fn provider_name(&self) -> &str {
        "Open-Meteo"
    }
    fn provider_filename_prefix(&self) -> &str {
        "open_meteo_"
    }
}

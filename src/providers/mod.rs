use anyhow::Error;

pub mod bom;
pub mod factory;
pub mod fetcher;
pub mod open_meteo;

use crate::domain::models::{DailyForecast, HourlyForecast};
use crate::errors::DashboardError;

/// Result of a weather data fetch operation
pub struct FetchResult<T> {
    pub data: T,
    pub warning: Option<DashboardError>,
}

impl<T> FetchResult<T> {
    pub fn fresh(data: T) -> Self {
        Self {
            data,
            warning: None,
        }
    }

    pub fn stale(data: T, error: DashboardError) -> Self {
        Self {
            data,
            warning: Some(error),
        }
    }
}

pub trait WeatherProvider {
    fn fetch_hourly_forecast(&self) -> Result<FetchResult<Vec<HourlyForecast>>, Error>;
    fn fetch_daily_forecast(&self) -> Result<FetchResult<Vec<DailyForecast>>, Error>;
    fn provider_name(&self) -> &str;
}

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
    fn provider_filename_prefix(&self) -> &str;

    /// Helper method to generate cache filename from provider prefix and suffix
    ///
    /// # Arguments
    /// * `suffix` - The cache file suffix (e.g., "hourly_forecast.json")
    ///
    /// # Returns
    /// * Full cache filename (e.g., "bom_hourly_forecast.json")
    fn get_cache_filename(&self, suffix: &str) -> String {
        format!("{}{}", self.provider_filename_prefix(), suffix)
    }
}

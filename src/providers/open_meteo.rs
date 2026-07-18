// According to this issue https://github.com/open-meteo/open-meteo/issues/228
// the WMO code represents the worst case, so you might get Rime Fog in middle of summer, even most of the day is sunny

use anyhow::Error;
use std::path::PathBuf;

use crate::{
    apis::open_meteo::models::{OpenMeteoDailyResponse, OpenMeteoError, OpenMeteoHourlyResponse},
    configs::settings::DashboardSettings,
    constants::{
        open_meteo_daily_endpoint, open_meteo_hourly_endpoint, DAILY_CACHE_SUFFIX,
        HOURLY_CACHE_SUFFIX,
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
    fn fetch_hourly_forecast(
        &self,
        settings: &DashboardSettings,
    ) -> Result<FetchResult<Vec<HourlyForecast>>, Error> {
        let result = match self.fetcher.fetch_data::<OpenMeteoHourlyResponse>(
            settings,
            open_meteo_hourly_endpoint(settings),
            &self.generate_cache_filename(HOURLY_CACHE_SUFFIX),
            Some(check_open_meteo_error),
        )? {
            FetchOutcome::Fresh(data) => FetchResult::fresh(data.into_domain(settings)),
            FetchOutcome::Stale { data, error } => {
                FetchResult::stale(data.into_domain(settings), error)
            }
        };

        Ok(result)
    }

    fn fetch_daily_forecast(
        &self,
        settings: &DashboardSettings,
    ) -> Result<FetchResult<Vec<DailyForecast>>, Error> {
        let result = match self.fetcher.fetch_data::<OpenMeteoDailyResponse>(
            settings,
            open_meteo_daily_endpoint(settings),
            &self.generate_cache_filename(DAILY_CACHE_SUFFIX),
            Some(check_open_meteo_error),
        )? {
            FetchOutcome::Fresh(data) => FetchResult::fresh(data.into_domain(settings)),
            FetchOutcome::Stale { data, error } => {
                FetchResult::stale(data.into_domain(settings), error)
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_body_is_ok() {
        assert!(check_open_meteo_error("not json at all").is_ok());
    }

    #[test]
    fn error_false_is_ok() {
        let body = r#"{"error": false, "reason": ""}"#;
        assert!(check_open_meteo_error(body).is_ok());
    }

    #[test]
    fn error_true_becomes_api_error_with_reason() {
        let body = r#"{"error": true, "reason": "Latitude must be in range of -90 to 90"}"#;
        let err = check_open_meteo_error(body).unwrap_err();
        match err {
            DashboardError::ApiError { details } => {
                assert_eq!(details, "Latitude must be in range of -90 to 90")
            }
            other => panic!("expected ApiError, got {other:?}"),
        }
    }
}

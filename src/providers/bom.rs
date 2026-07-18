use anyhow::Error;
use std::path::PathBuf;

use crate::{
    apis::bom::models::{BomError, DailyForecastResponse, HourlyForecastResponse},
    configs::settings::DashboardSettings,
    constants::{
        daily_forecast_endpoint, hourly_forecast_endpoint, DAILY_CACHE_SUFFIX, HOURLY_CACHE_SUFFIX,
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
    fn fetch_hourly_forecast(
        &self,
        settings: &DashboardSettings,
    ) -> Result<FetchResult<Vec<HourlyForecast>>, Error> {
        match self.fetcher.fetch_data::<HourlyForecastResponse>(
            settings,
            hourly_forecast_endpoint(settings),
            &self.generate_cache_filename(HOURLY_CACHE_SUFFIX),
            Some(check_bom_error),
        )? {
            FetchOutcome::Fresh(data) => {
                // Convert BOM models to domain models
                let domain_data: Vec<HourlyForecast> = data
                    .data
                    .into_iter()
                    .map(|h| HourlyForecast::from_bom(h, settings))
                    .collect();
                crate::logger::debug(format!(
                    "Converted {} BOM hourly entries to domain model",
                    domain_data.len()
                ));
                Ok(FetchResult::fresh(domain_data))
            }
            FetchOutcome::Stale { data, error } => {
                let domain_data: Vec<HourlyForecast> = data
                    .data
                    .into_iter()
                    .map(|h| HourlyForecast::from_bom(h, settings))
                    .collect();
                Ok(FetchResult::stale(domain_data, error))
            }
        }
    }

    fn fetch_daily_forecast(
        &self,
        settings: &DashboardSettings,
    ) -> Result<FetchResult<Vec<DailyForecast>>, Error> {
        match self.fetcher.fetch_data::<DailyForecastResponse>(
            settings,
            daily_forecast_endpoint(settings),
            &self.generate_cache_filename(DAILY_CACHE_SUFFIX),
            Some(check_bom_error),
        )? {
            FetchOutcome::Fresh(data) => {
                // Convert BOM models to domain models
                let domain_data: Vec<DailyForecast> = data
                    .data
                    .into_iter()
                    .map(|d| DailyForecast::from_bom(d, settings))
                    .collect();
                crate::logger::debug(format!(
                    "Converted {} BOM daily entries to domain model",
                    domain_data.len()
                ));
                Ok(FetchResult::fresh(domain_data))
            }
            FetchOutcome::Stale { data, error } => {
                let domain_data: Vec<DailyForecast> = data
                    .data
                    .into_iter()
                    .map(|d| DailyForecast::from_bom(d, settings))
                    .collect();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_body_is_ok() {
        assert!(check_bom_error("not json at all").is_ok());
    }

    #[test]
    fn non_error_shaped_json_is_ok() {
        assert!(check_bom_error(r#"{"data": []}"#).is_ok());
    }

    #[test]
    fn empty_errors_array_is_ok() {
        assert!(check_bom_error(r#"{"errors": []}"#).is_ok());
    }

    #[test]
    fn single_error_becomes_its_own_detail_message() {
        let body = r#"{"errors": [{"detail": "Invalid geohash"}]}"#;
        let err = check_bom_error(body).unwrap_err();
        match err {
            DashboardError::ApiError { details } => assert_eq!(details, "Invalid geohash"),
            other => panic!("expected ApiError, got {other:?}"),
        }
    }

    #[test]
    fn multiple_errors_are_combined_and_numbered() {
        let body = r#"{"errors": [{"detail": "Invalid geohash"}, {"detail": "Rate limited"}]}"#;
        let err = check_bom_error(body).unwrap_err();
        match err {
            DashboardError::ApiError { details } => {
                assert!(details.contains("BOM API returned 2 errors"));
                assert!(details.contains("1. Invalid geohash"));
                assert!(details.contains("2. Rate limited"));
            }
            other => panic!("expected ApiError, got {other:?}"),
        }
    }
}

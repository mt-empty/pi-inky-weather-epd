use crate::{configs::settings::TemperatureUnit, CONFIG};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use url::Url;

pub const BOM_API_TEMP_UNIT: TemperatureUnit = TemperatureUnit::C;
pub const DEFAULT_AXIS_LABEL_FONT_SIZE: u16 = 19;

const BASE_WEATHER_URL: &str = "https://api.weather.bom.gov.au/v1/locations";
const NOT_AVAILABLE_ICON_NAME: &str = "not-available.svg";

fn build_forecast_url(frequency: &str) -> Url {
    {
        let mut u =
            Url::parse(BASE_WEATHER_URL).expect("Failed to construct forecast endpoint URL");

        u.path_segments_mut()
            .unwrap()
            .push(CONFIG.api.location.as_ref())
            .push("forecasts")
            .push(frequency);
        u
    }
}

pub static DAILY_FORECAST_ENDPOINT: Lazy<Url> = Lazy::new(|| build_forecast_url("daily"));
pub static HOURLY_FORECAST_ENDPOINT: Lazy<Url> = Lazy::new(|| build_forecast_url("hourly"));

pub static NOT_AVAILABLE_ICON_PATH: Lazy<PathBuf> = Lazy::new(|| {
    CONFIG
        .misc
        .weather_data_cache_path
        .join(NOT_AVAILABLE_ICON_NAME)
});

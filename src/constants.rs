use crate::CONFIG;
use lazy_static::lazy_static;

pub const NOT_AVAILABLE_ICON: &str = "not-available.svg";
pub const DEFAULT_AXIS_LABEL_FONT_SIZE: usize = 19;

const WEATHER_PROVIDER: &str = "https://api.weather.bom.gov.au/v1/locations";
lazy_static! {
    pub static ref DAILY_FORECAST_ENDPOINT: String = format!(
        "{}/{}/forecasts/daily",
        WEATHER_PROVIDER, CONFIG.api.location
    );
    pub static ref HOURLY_FORECAST_ENDPOINT: String = format!(
        "{}/{}/forecasts/hourly",
        WEATHER_PROVIDER, CONFIG.api.location
    );
    pub static ref NOT_AVAILABLE_ICON_PATH: String = format!(
        "{}{}",
        CONFIG.misc.weather_data_cache_path, NOT_AVAILABLE_ICON
    );
}

//! Integration tests for the cache "grace period": how many days of
//! consecutive fetch failures the app tolerates — falling back to an aging
//! cache — before the display starts showing N/A for days/hours the cache
//! no longer covers.
//!
//! The grace period is driven entirely by `forecast_days` (and, for daily,
//! `past_days`) in `src/constants.rs` vs. what the dashboard actually needs
//! to display (7 days daily, 24h hourly) — there is no cache-age/staleness
//! check of its own; `Fetcher` uses whatever is cached, however old.
//!
//! Each test seeds a cache file dated from a fixed `FETCH_DATE`, points a
//! mock server at an always-400 (non-retryable) response so `Fetcher` falls
//! back to that cache on the first attempt, and feeds the result into
//! `ContextBuilder` exactly as `generate_weather_dashboard_injection` does —
//! exercising the real fetch/fallback/parse/context pipeline end to end.

mod helpers;

use chrono::{Datelike, Days, NaiveDate, TimeZone, Utc};
use helpers::test_utils::open_meteo_settings;
use pi_inky_weather_epd::clock::FixedClock;
use pi_inky_weather_epd::constants::NOT_AVAILABLE;
use pi_inky_weather_epd::dashboard::context::ContextBuilder;
use pi_inky_weather_epd::providers::factory::create_provider;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Fixed reference date standing in for "the last time a fetch succeeded".
fn fetch_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2025, 1, 10).unwrap()
}

/// Minimal Daily response JSON covering `num_days` consecutive dates from
/// `start_date`, matching `open_meteo_daily_endpoint`'s current
/// `forecast_days=14, past_days=1` (15 dates total).
fn build_daily_json(start_date: NaiveDate, num_days: u64) -> String {
    let dates: Vec<String> = (0..num_days)
        .map(|i| (start_date + Days::new(i)).format("%Y-%m-%d").to_string())
        .collect();
    let sunrise: Vec<String> = dates.iter().map(|d| format!("{d}T07:00")).collect();
    let sunset: Vec<String> = dates.iter().map(|d| format!("{d}T18:00")).collect();
    let n = dates.len();

    serde_json::json!({
        "latitude": 51.5,
        "longitude": -0.1,
        "timezone": "UTC",
        "daily_units": {
            "temperature_2m_max": "°C",
            "temperature_2m_min": "°C",
            "precipitation_sum": "mm",
            "precipitation_probability_max": "%",
            "snowfall_sum": "cm"
        },
        "daily": {
            "time": dates,
            "sunrise": sunrise,
            "sunset": sunset,
            "temperature_2m_max": vec![20.0; n],
            "temperature_2m_min": vec![10.0; n],
            "precipitation_sum": vec![0.0; n],
            "precipitation_probability_max": vec![10; n],
            "snowfall_sum": vec![0.0; n],
            "cloud_cover_mean": vec![20; n]
        }
    })
    .to_string()
}

/// Minimal Hourly response JSON covering `num_hours` consecutive UTC hours
/// from `start`, matching `open_meteo_hourly_endpoint`'s current
/// `forecast_days=14` (14 * 24 = 336 hours).
fn build_hourly_json(start: chrono::DateTime<Utc>, num_hours: i64) -> String {
    let times: Vec<String> = (0..num_hours)
        .map(|i| {
            (start + chrono::Duration::hours(i))
                .format("%Y-%m-%dT%H:%M")
                .to_string()
        })
        .collect();
    let n = times.len();

    serde_json::json!({
        "latitude": 51.5,
        "longitude": -0.1,
        "timezone": "UTC",
        "current_units": {"interval": "seconds", "is_day": ""},
        "current": {"time": times[0], "is_day": 1},
        "hourly_units": {
            "temperature_2m": "°C",
            "apparent_temperature": "°C",
            "precipitation_probability": "%",
            "precipitation": "mm",
            "snowfall": "cm",
            "uv_index": "",
            "wind_speed_10m": "km/h",
            "wind_gusts_10m": "km/h",
            "relative_humidity_2m": "%"
        },
        "hourly": {
            "time": times,
            "temperature_2m": vec![20.0; n],
            "apparent_temperature": vec![18.0; n],
            "precipitation_probability": vec![10; n],
            "precipitation": vec![0.0; n],
            "snowfall": vec![0.0; n],
            "uv_index": vec![5.0; n],
            "wind_speed_10m": vec![15.0; n],
            "wind_gusts_10m": vec![25.0; n],
            "relative_humidity_2m": vec![50; n],
            "cloud_cover": vec![30; n]
        }
    })
    .to_string()
}

/// Result fields relevant to the grace-period boundary, pulled out of the
/// `ContextBuilder` before it (and the settings/temp cache dir it borrows)
/// drop at the end of the blocking task.
struct Probe {
    day7_name: String,
    day7_maxtemp: String,
    current_hour_actual_temp: String,
}

/// Seeds a cache dated from `fetch_date()`, points a mock server that always
/// responds 400 (so `Fetcher` falls back to that cache on the first, non-retried
/// attempt), runs the real provider fetch + `ContextBuilder` pipeline with the
/// clock at `now`, and returns the fields the grace-period tests assert on.
async fn probe_at(now: chrono::DateTime<Utc>) -> Probe {
    let mock_server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/v1/forecast"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": true,
            "reason": "simulated outage"
        })))
        .mount(&mock_server)
        .await;

    let mock_uri = mock_server.uri();

    tokio::task::spawn_blocking(move || {
        let mut settings = open_meteo_settings(&mock_uri);
        settings.misc.timezone = chrono_tz::UTC;

        let cache_dir = settings.misc.weather_data_cache_path.clone();
        std::fs::write(
            cache_dir.join("open_meteo_daily_forecast.json"),
            build_daily_json(fetch_date() - Days::new(1), 15),
        )
        .expect("failed to seed daily cache");
        std::fs::write(
            cache_dir.join("open_meteo_hourly_forecast.json"),
            build_hourly_json(
                Utc.from_utc_datetime(&fetch_date().and_hms_opt(0, 0, 0).unwrap()),
                14 * 24,
            ),
        )
        .expect("failed to seed hourly cache");

        let provider = create_provider(&settings).expect("failed to create provider");
        let daily = provider
            .fetch_daily_forecast(&settings)
            .expect("fetch_daily_forecast returned Err instead of falling back to cache");
        let hourly = provider
            .fetch_hourly_forecast(&settings)
            .expect("fetch_hourly_forecast returned Err instead of falling back to cache");
        assert!(
            daily.warning.is_some(),
            "expected the mock's 400 response to force a stale-cache fallback"
        );
        assert!(
            hourly.warning.is_some(),
            "expected the mock's 400 response to force a stale-cache fallback"
        );

        let clock = FixedClock::new(now);
        let mut builder = ContextBuilder::new(&settings, &clock);
        builder.with_daily_forecast_data(daily.data, &clock);
        builder.with_hourly_forecast_data(hourly.data, &clock);

        Probe {
            day7_name: builder.context.day7_name.clone(),
            day7_maxtemp: builder.context.day7_maxtemp.clone(),
            current_hour_actual_temp: builder.context.current_hour_actual_temp.clone(),
        }
    })
    .await
    .expect("blocking task panicked")
}

fn at_hour_utc(date: NaiveDate, hour: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), hour, 0, 0)
        .unwrap()
}

fn noon_utc_on(date: NaiveDate) -> chrono::DateTime<Utc> {
    at_hour_utc(date, 12)
}

/// Daily cache covers `fetch_date - 1` through `fetch_date + 13` (15 dates:
/// `past_days=1` + `forecast_days=14`). day7 is `today + 6`, so the last day
/// this cache can still populate is `today == fetch_date + 7`.
#[tokio::test]
async fn day7_still_populated_at_the_edge_of_the_daily_grace_period() {
    let today = fetch_date() + Days::new(7);
    let probe = probe_at(noon_utc_on(today)).await;

    assert_ne!(probe.day7_name, "Unknown");
    assert_eq!(probe.day7_maxtemp, "20");
}

#[tokio::test]
async fn day7_goes_not_available_one_day_past_the_daily_grace_period() {
    let today = fetch_date() + Days::new(8);
    let probe = probe_at(noon_utc_on(today)).await;

    assert_eq!(probe.day7_maxtemp, NOT_AVAILABLE);
}

/// Hourly cache covers 336 consecutive hours from `fetch_date` 00:00 UTC
/// (`forecast_days=14`, no `past_days`), so the last hour it can still
/// resolve a forecast window from is `fetch_date + 13` 23:00 UTC.
#[tokio::test]
async fn current_hour_still_populated_within_the_hourly_grace_period() {
    let now = noon_utc_on(fetch_date() + Days::new(13));
    let probe = probe_at(now).await;

    assert_ne!(probe.current_hour_actual_temp, NOT_AVAILABLE);
}

#[tokio::test]
async fn current_hour_goes_not_available_past_the_hourly_grace_period() {
    // Last cached hour is fetch_date + 13, 23:00 UTC; one hour past that has
    // no entry left for `find_forecast_window` to pick up.
    let now = at_hour_utc(fetch_date() + Days::new(14), 0);
    let probe = probe_at(now).await;

    assert_eq!(probe.current_hour_actual_temp, NOT_AVAILABLE);
}

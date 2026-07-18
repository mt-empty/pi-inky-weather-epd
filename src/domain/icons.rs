use super::models::{DailyForecast, HourlyForecast, Precipitation, Wind};
use crate::logger;
use crate::weather::icons::{
    DayNight, Icon, IconContext, PrecipitationChanceName, PrecipitationKind, WindIconName,
};
use crate::weather::utils::moon_phase_icon_name;

// ============================================================================
// Icon implementations for domain models
// ============================================================================

impl Icon for Wind {
    fn icon_name(&self, ctx: &IconContext) -> String {
        let speed = self.speed(ctx.render_options.use_gust_instead_of_wind);
        match speed {
            0..=20 => WindIconName::Wind,
            21..=40 => WindIconName::UmbrellaWind,
            41.. => WindIconName::UmbrellaWindAlt,
        }
        .to_string()
    }
}

/// If `use_moon_phase_instead_of_clear_night` is enabled and the icon represents
/// a clear night, replaces it with the current moon phase icon.
/// only relevant for hourly forecasts, as daily forecasts always use day icons.
fn apply_moon_phase_override(icon: String, is_night: bool, ctx: &IconContext) -> String {
    if !is_night || !ctx.render_options.use_moon_phase_instead_of_clear_night {
        return icon;
    }

    let clear_night_suffix = format!("{}{}.svg", PrecipitationChanceName::Clear, DayNight::Night);

    if icon.ends_with(&clear_night_suffix) {
        logger::detail("Using moon phase icon instead of clear night");
        moon_phase_icon_name(ctx.today).to_string()
    } else {
        icon
    }
}

/// Converts a percentage (0–100) to a `PrecipitationChanceName`.
/// Used for both cloud cover data and precipitation chance fallback.
#[must_use]
fn percentage_to_cloud_name(pct: u16) -> PrecipitationChanceName {
    match pct {
        0..=25 => PrecipitationChanceName::Clear,
        26..=50 => PrecipitationChanceName::PartlyCloudy,
        51..=75 => PrecipitationChanceName::Overcast,
        76.. => PrecipitationChanceName::Extreme,
    }
}

/// Converts the precipitation amount to a corresponding `PrecipitationKind`.
/// Returns `None` when there is no meaningful precipitation.
///
/// `is_hourly`: if true, scales the amount to a daily equivalent before classifying.
#[must_use]
fn precipitation_amount_to_name(
    precip: &Precipitation,
    is_hourly: bool,
) -> Option<PrecipitationKind> {
    let mut amount = precip.amount();

    if is_hourly {
        amount *= 24.0;
    }

    // If primarily snow, return snow variant instead of rain
    if precip.is_primarily_snow() {
        return match amount {
            0.0..1.4 => None,
            _ => Some(PrecipitationKind::Snow),
        };
    }

    match amount {
        0.0..3.0 => None,
        3.0..=20.0 => Some(PrecipitationKind::Drizzle),
        _ => Some(PrecipitationKind::Rain),
    }
}

/// Ensures precipitation amount requires minimum cloud coverage.
/// Heavy precipitation cannot occur with completely clear skies.
///
/// # Arguments
///
/// * `cloud_name` - Cloud cover level from cloud data or precipitation chance
/// * `amount_name` - Precipitation amount (None, Drizzle, or Rain)
///
/// # Returns
///
/// * Adjusted cloud level ensuring consistency with precipitation amount
#[must_use]
fn apply_precipitation_override(
    cloud_name: PrecipitationChanceName,
    amount_name: Option<PrecipitationKind>,
) -> PrecipitationChanceName {
    let Some(kind) = amount_name else {
        return cloud_name;
    };
    match kind {
        PrecipitationKind::Drizzle => {
            // Drizzle requires at least partly cloudy
            match cloud_name {
                PrecipitationChanceName::Clear => PrecipitationChanceName::PartlyCloudy,
                _ => cloud_name,
            }
        }
        PrecipitationKind::Rain => {
            // Heavy rain requires at least overcast
            match cloud_name {
                PrecipitationChanceName::Clear | PrecipitationChanceName::PartlyCloudy => {
                    PrecipitationChanceName::Overcast
                }
                _ => cloud_name,
            }
        }
        PrecipitationKind::Snow => {
            // Snow requires at least partly cloudy
            match cloud_name {
                PrecipitationChanceName::Clear => PrecipitationChanceName::PartlyCloudy,
                _ => cloud_name,
            }
        }
        PrecipitationKind::Sleet => {
            // Sleet (freezing rain/drizzle) requires at least partly cloudy
            match cloud_name {
                PrecipitationChanceName::Clear => PrecipitationChanceName::PartlyCloudy,
                _ => cloud_name,
            }
        }
        PrecipitationKind::Hail => {
            // Hail requires at least overcast (severe weather)
            match cloud_name {
                PrecipitationChanceName::Clear | PrecipitationChanceName::PartlyCloudy => {
                    PrecipitationChanceName::Overcast
                }
                _ => cloud_name,
            }
        }
    }
}

impl Icon for DailyForecast {
    fn icon_name(&self, ctx: &IconContext) -> String {
        // Priority 1: Use WMO weather code if available (most accurate)
        let fallback_reason = if ctx.render_options.prefer_weather_codes {
            match self.weather_code {
                Some(Ok(wmo_code)) => {
                    logger::debug("DailyForecast: Using WMO weather code for icon selection");
                    return wmo_code.icon_name(false);
                }
                Some(Err(code)) => format!("unknown WMO code ({code})"),
                None => "no WMO code available".to_string(),
            }
        } else {
            "prefer_weather_codes disabled".to_string()
        };
        logger::debug(format!(
            "DailyForecast: Falling back to precipitation-based icon logic ({fallback_reason})"
        ));

        // Priority 2: Fall back to precipitation-based logic (BOM provider, missing codes)
        if let Some(ref precip) = self.precipitation {
            // Determine cloud coverage from cloud_cover data if available, otherwise fall back to precipitation chance
            let raw_cloud_name = if let Some(cloud_cover) = self.cloud_cover {
                percentage_to_cloud_name(cloud_cover)
            } else {
                percentage_to_cloud_name(precip.chance.unwrap_or(0))
            };

            let amount_name = precipitation_amount_to_name(precip, false);

            // Apply precipitation override: ensure heavy rain requires adequate cloud cover
            // Note: After override, Clear can only occur with amount_name = None
            let cloud_name = apply_precipitation_override(raw_cloud_name, amount_name);
            let suffix = amount_name.map(|k| k.to_string()).unwrap_or_default();

            format!("{cloud_name}{}{suffix}.svg", DayNight::Day)
        } else {
            // Default to clear day if no precipitation data
            format!("{}{}.svg", PrecipitationChanceName::Clear, DayNight::Day)
        }
    }
}

impl Icon for HourlyForecast {
    fn icon_name(&self, ctx: &IconContext) -> String {
        // Priority 1: Use WMO weather code if available (most accurate)
        let fallback_reason = if ctx.render_options.prefer_weather_codes {
            match self.weather_code {
                Some(Ok(wmo_code)) => {
                    logger::debug("HourlyForecast: Using WMO weather code for icon selection");
                    let icon = wmo_code.icon_name(self.is_night);
                    return apply_moon_phase_override(icon, self.is_night, ctx);
                }
                Some(Err(code)) => format!("unknown WMO code ({code})"),
                None => "no WMO code available".to_string(),
            }
        } else {
            "prefer_weather_codes disabled".to_string()
        };
        logger::debug(format!(
            "HourlyForecast: Falling back to precipitation-based icon logic ({fallback_reason})"
        ));

        // Priority 2: Fall back to cloud_cover + precipitation logic (BOM provider, missing codes)
        // Determine cloud coverage from cloud_cover data if available, otherwise fall back to precipitation chance
        let raw_cloud_name = if let Some(cloud_cover) = self.cloud_cover {
            percentage_to_cloud_name(cloud_cover)
        } else {
            percentage_to_cloud_name(self.precipitation.chance.unwrap_or(0))
        };

        let amount_name = precipitation_amount_to_name(&self.precipitation, true);
        let day_night = if self.is_night {
            DayNight::Night
        } else {
            DayNight::Day
        };

        // Apply precipitation override: ensure heavy rain requires adequate cloud cover
        // Note: After override, Clear can only occur with amount_name = None
        let cloud_name = apply_precipitation_override(raw_cloud_name, amount_name);
        let suffix = amount_name.map(|k| k.to_string()).unwrap_or_default();

        let icon = format!("{cloud_name}{day_night}{suffix}.svg");
        apply_moon_phase_override(icon, self.is_night, ctx)
    }
}

/// Icon name generation with cloud cover data and precipitation override logic.
///
/// The icon selection system prioritizes cloud_cover data when available, with
/// fallback to precipitation-based estimation. The precipitation override
/// ensures realistic weather combinations (e.g., no heavy rain with clear skies).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::settings::DashboardSettings;
    use crate::domain::models::Temperature;
    use crate::weather::icons::placeholder_today;
    use chrono::{NaiveDate, Utc};

    mod cloud_cover {
        use super::*;

        #[test]
        fn overrides_low_precipitation_chance() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // High cloud cover (80%) should produce overcast icon even with low precip chance (10%)
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(18.0),
                apparent_temperature: Temperature::celsius(16.0),
                wind: Wind::new(10, 15),
                precipitation: Precipitation::new(Some(10), Some(0), Some(0)),
                uv_index: 3,
                relative_humidity: 70,
                is_night: false,
                cloud_cover: Some(80),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "extreme-day.svg");
        }

        #[test]
        fn boundary_25_percent() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // 25% cloud cover is upper limit of Clear range
            let forecast_25 = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(20.0),
                apparent_temperature: Temperature::celsius(19.0),
                wind: Wind::new(8, 12),
                precipitation: Precipitation::new(Some(50), Some(0), Some(0)),
                uv_index: 5,
                relative_humidity: 50,
                is_night: false,
                cloud_cover: Some(25),
                weather_code: None,
            };
            let forecast_26 = HourlyForecast {
                cloud_cover: Some(26),
                ..forecast_25.clone()
            };

            assert_eq!(forecast_25.icon_name(&ctx), "clear-day.svg");
            assert_eq!(forecast_26.icon_name(&ctx), "partly-cloudy-day.svg");
        }

        #[test]
        fn boundary_50_and_51_percent() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            let forecast_50 = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(18.0),
                apparent_temperature: Temperature::celsius(17.0),
                wind: Wind::new(10, 15),
                precipitation: Precipitation::new(Some(30), Some(0), Some(0)),
                uv_index: 3,
                relative_humidity: 65,
                is_night: false,
                cloud_cover: Some(50),
                weather_code: None,
            };
            let forecast_51 = HourlyForecast {
                cloud_cover: Some(51),
                ..forecast_50.clone()
            };

            assert_eq!(forecast_50.icon_name(&ctx), "partly-cloudy-day.svg");
            assert_eq!(forecast_51.icon_name(&ctx), "overcast-day.svg");
        }

        #[test]
        fn boundary_75_and_76_percent() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            let forecast_75 = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(16.0),
                apparent_temperature: Temperature::celsius(15.0),
                wind: Wind::new(12, 20),
                precipitation: Precipitation::new(Some(60), Some(0), Some(0)),
                uv_index: 2,
                relative_humidity: 80,
                is_night: false,
                cloud_cover: Some(75),
                weather_code: None,
            };
            let forecast_76 = HourlyForecast {
                cloud_cover: Some(76),
                ..forecast_75.clone()
            };

            assert_eq!(forecast_75.icon_name(&ctx), "overcast-day.svg");
            assert_eq!(forecast_76.icon_name(&ctx), "extreme-day.svg");
        }

        #[test]
        fn null_cloud_cover_falls_back_to_precipitation() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(20.0),
                apparent_temperature: Temperature::celsius(19.0),
                wind: Wind::new(10, 15),
                precipitation: Precipitation::new(Some(40), Some(0), Some(0)),
                uv_index: 4,
                relative_humidity: 60,
                is_night: false,
                cloud_cover: None,
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "partly-cloudy-day.svg");
        }
    }

    mod precipitation_override {
        use super::*;

        #[test]
        fn drizzle_requires_partly_cloudy() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // Clear skies (20% cloud) + drizzle should be bumped to PartlyCloudy
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(18.0),
                apparent_temperature: Temperature::celsius(17.0),
                wind: Wind::new(8, 12),
                precipitation: Precipitation::new(Some(20), Some(0), Some(1)), // 24mm/day = Drizzle
                uv_index: 4,
                relative_humidity: 65,
                is_night: false,
                cloud_cover: Some(15),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "partly-cloudy-day-drizzle.svg");
        }

        #[test]
        fn rain_requires_overcast() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // Clear skies (20% cloud) + heavy rain should be bumped to Overcast
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(16.0),
                apparent_temperature: Temperature::celsius(15.0),
                wind: Wind::new(15, 25),
                precipitation: Precipitation::new(Some(25), Some(10), Some(20)), // 360mm/day = Rain
                uv_index: 2,
                relative_humidity: 85,
                is_night: false,
                cloud_cover: Some(20),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "overcast-day-rain.svg");
        }

        #[test]
        fn partly_cloudy_rain_becomes_overcast() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // PartlyCloudy (40% cloud) + heavy rain should be bumped to Overcast
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(16.0),
                apparent_temperature: Temperature::celsius(14.0),
                wind: Wind::new(18, 30),
                precipitation: Precipitation::new(Some(50), Some(8), Some(15)), // 276mm/day = Rain
                uv_index: 1,
                relative_humidity: 90,
                is_night: false,
                cloud_cover: Some(40),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "overcast-day-rain.svg");
        }

        #[test]
        fn fallback_with_precipitation_override() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // Low precip chance + drizzle amount; median of 0-5mm = 2.5mm which
            // is in the None range (0-2.0), so no override is needed.
            let forecast = DailyForecast {
                date: Some(chrono::Local::now().date_naive()),
                temp_max: Some(Temperature::celsius(25.0)),
                temp_min: Some(Temperature::celsius(15.0)),
                precipitation: Some(Precipitation::new(Some(15), Some(0), Some(5))),
                astronomical: None,
                cloud_cover: None,
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "clear-day.svg");
        }
    }

    mod valid_combinations {
        use super::*;

        #[test]
        fn partly_cloudy_with_drizzle_is_valid() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(18.0),
                apparent_temperature: Temperature::celsius(16.0),
                wind: Wind::new(12, 20),
                precipitation: Precipitation::new(Some(40), Some(0), Some(1)), // 24mm/day -> Drizzle
                uv_index: 3,
                relative_humidity: 70,
                is_night: false,
                cloud_cover: None,
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "partly-cloudy-day-drizzle.svg");
        }

        #[test]
        fn overcast_with_rain_is_valid() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(16.0),
                apparent_temperature: Temperature::celsius(14.0),
                wind: Wind::new(15, 25),
                precipitation: Precipitation::new(Some(60), Some(5), Some(15)), // 240mm/day -> Rain
                uv_index: 2,
                relative_humidity: 85,
                is_night: false,
                cloud_cover: None,
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "overcast-day-rain.svg");
        }

        #[test]
        fn extreme_with_drizzle_is_valid() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(12.0),
                apparent_temperature: Temperature::celsius(10.0),
                wind: Wind::new(20, 35),
                precipitation: Precipitation::new(Some(85), Some(0), Some(1)), // 12mm/day -> Drizzle
                uv_index: 1,
                relative_humidity: 90,
                is_night: true,
                cloud_cover: None,
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "extreme-night-drizzle.svg");
        }

        #[test]
        fn zero_chance_zero_amount_produces_clear() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            let forecast = DailyForecast {
                date: Some(chrono::Local::now().date_naive()),
                temp_max: Some(Temperature::celsius(28.0)),
                temp_min: Some(Temperature::celsius(18.0)),
                precipitation: Some(Precipitation::new(Some(0), Some(0), Some(0))),
                astronomical: None,
                cloud_cover: None,
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "clear-day.svg");
        }

        #[test]
        fn boundary_25_percent_is_still_clear() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(20.0),
                apparent_temperature: Temperature::celsius(19.0),
                wind: Wind::new(8, 12),
                precipitation: Precipitation::new(Some(25), Some(0), Some(0)),
                uv_index: 4,
                relative_humidity: 55,
                is_night: false,
                cloud_cover: Some(22),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "clear-day.svg");
        }

        #[test]
        fn boundary_26_percent_allows_precipitation_suffix() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(20.0),
                apparent_temperature: Temperature::celsius(19.0),
                wind: Wind::new(8, 12),
                precipitation: Precipitation::new(Some(26), Some(0), Some(1)), // Drizzle (24mm/day)
                uv_index: 4,
                relative_humidity: 55,
                is_night: false,
                cloud_cover: None,
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "partly-cloudy-day-drizzle.svg");
        }
    }

    mod snow_icon_selection {
        use super::*;

        #[test]
        fn high_snowfall_selects_snow_icon() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // 14.3cm snow × 1.43 ≈ 20.4mm water, total ~10mm = 204% (well above threshold)
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(-2.0),
                apparent_temperature: Temperature::celsius(-5.0),
                wind: Wind::new(10, 15),
                precipitation: Precipitation::new_with_snowfall(
                    Some(80),
                    Some(8),
                    Some(12),
                    Some(143),
                ),
                uv_index: 1,
                relative_humidity: 85,
                is_night: false,
                cloud_cover: Some(80),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "extreme-day-snow.svg");
        }

        #[test]
        fn at_60_percent_threshold_is_snow() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // 4.3cm -> 6.14mm water > 6.0mm (60% of 10mm median) -> 61.4% snow
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(0.0),
                apparent_temperature: Temperature::celsius(-2.0),
                wind: Wind::new(8, 12),
                precipitation: Precipitation::new_with_snowfall(
                    Some(65),
                    Some(9),
                    Some(11),
                    Some(43),
                ),
                uv_index: 2,
                relative_humidity: 80,
                is_night: true,
                cloud_cover: Some(70),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "overcast-night-snow.svg");
        }

        #[test]
        fn below_snow_threshold_is_rain() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // 3.0cm × 10/7 ≈ 4.29mm water / 10mm total = 42.9% (below 60%)
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(2.0),
                apparent_temperature: Temperature::celsius(0.0),
                wind: Wind::new(12, 18),
                precipitation: Precipitation::new_with_snowfall(
                    Some(70),
                    Some(9),
                    Some(11),
                    Some(30),
                ),
                uv_index: 1,
                relative_humidity: 85,
                is_night: false,
                cloud_cover: Some(75),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "overcast-day-rain.svg");
        }

        #[test]
        fn snow_override_requires_partly_cloudy() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // Snow with clear skies should bump to partly-cloudy
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(-3.0),
                apparent_temperature: Temperature::celsius(-6.0),
                wind: Wind::new(5, 8),
                precipitation: Precipitation::new_with_snowfall(
                    Some(30),
                    Some(0),
                    Some(2),
                    Some(15),
                ),
                uv_index: 2,
                relative_humidity: 70,
                is_night: false,
                cloud_cover: Some(20),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "partly-cloudy-day-snow.svg");
        }

        #[test]
        fn low_snowfall_shows_clear_not_snow() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // 0.2cm × 10/7 ≈ 0.29mm water — below the 1.4mm threshold
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(-1.0),
                apparent_temperature: Temperature::celsius(-3.0),
                wind: Wind::new(3, 5),
                precipitation: Precipitation::new_with_snowfall(
                    Some(15),
                    Some(0),
                    Some(0),
                    Some(2),
                ),
                uv_index: 3,
                relative_humidity: 60,
                is_night: false,
                cloud_cover: Some(10),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "clear-day.svg");
        }

        #[test]
        fn mixed_precipitation_favors_rain() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // 2.0cm × 10/7 ≈ 2.86mm water / 10mm total = 28.6% (below 60%)
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(3.0),
                apparent_temperature: Temperature::celsius(1.0),
                wind: Wind::new(10, 15),
                precipitation: Precipitation::new_with_snowfall(
                    Some(60),
                    Some(8),
                    Some(12),
                    Some(20),
                ),
                uv_index: 1,
                relative_humidity: 90,
                is_night: true,
                cloud_cover: Some(65),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "overcast-night-rain.svg");
        }

        #[test]
        fn partly_cloudy_snow_at_night() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // 10cm × 1.43 ≈ 14.3mm water, total ~8mm = 179% (well above threshold)
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(-5.0),
                apparent_temperature: Temperature::celsius(-8.0),
                wind: Wind::new(6, 10),
                precipitation: Precipitation::new_with_snowfall(
                    Some(45),
                    Some(7),
                    Some(9),
                    Some(100),
                ),
                uv_index: 0,
                relative_humidity: 80,
                is_night: true,
                cloud_cover: Some(40),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "partly-cloudy-night-snow.svg");
        }

        #[test]
        fn overcast_day_snow() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // 11.5cm × 1.43 ≈ 16.4mm water, total ~9mm = 183% (well above threshold)
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(-4.0),
                apparent_temperature: Temperature::celsius(-7.0),
                wind: Wind::new(12, 18),
                precipitation: Precipitation::new_with_snowfall(
                    Some(60),
                    Some(8),
                    Some(10),
                    Some(115),
                ),
                uv_index: 1,
                relative_humidity: 85,
                is_night: false,
                cloud_cover: Some(60),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "overcast-day-snow.svg");
        }

        #[test]
        fn extreme_night_snow() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());
            // 20cm × 1.43 ≈ 28.6mm water, total ~15mm = 190% (well above threshold)
            let forecast = HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(-10.0),
                apparent_temperature: Temperature::celsius(-15.0),
                wind: Wind::new(20, 35),
                precipitation: Precipitation::new_with_snowfall(
                    Some(90),
                    Some(14),
                    Some(16),
                    Some(200),
                ),
                uv_index: 0,
                relative_humidity: 90,
                is_night: true,
                cloud_cover: Some(85),
                weather_code: None,
            };

            assert_eq!(forecast.icon_name(&ctx), "extreme-night-snow.svg");
        }
    }

    mod moon_phase_override {
        use super::*;

        fn clear_night_forecast() -> HourlyForecast {
            HourlyForecast {
                time: Utc::now(),
                temperature: Temperature::celsius(10.0),
                apparent_temperature: Temperature::celsius(9.0),
                wind: Wind::new(5, 8),
                precipitation: Precipitation::new(Some(0), Some(0), Some(0)),
                uv_index: 0,
                relative_humidity: 50,
                is_night: true,
                cloud_cover: Some(10),
                weather_code: None,
            }
        }

        #[test]
        fn disabled_shows_clear_night() {
            let settings = DashboardSettings::load_test_config().unwrap();
            let ctx = IconContext::from_settings(&settings, placeholder_today());

            assert_eq!(clear_night_forecast().icon_name(&ctx), "clear-night.svg");
        }

        #[test]
        fn is_deterministic_for_a_fixed_date() {
            let mut settings = DashboardSettings::load_test_config().unwrap();
            settings
                .render_options
                .use_moon_phase_instead_of_clear_night = true;
            let today = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
            let ctx = IconContext::from_settings(&settings, today);

            let icon = clear_night_forecast().icon_name(&ctx);

            assert!(
                icon.starts_with("moon-"),
                "expected a moon-phase icon, got {icon}"
            );
            assert_eq!(icon, clear_night_forecast().icon_name(&ctx));
        }
    }
}

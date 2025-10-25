use serde::{Deserialize, Deserializer};

use crate::{configs::settings::TemperatureUnit, constants::BOM_API_TEMP_UNIT, CONFIG};

use super::models::Temperature;

pub fn de_temp_celsius<'de, D>(deserializer: D) -> Result<Temperature, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i16::deserialize(deserializer)?;
    let temp = Temperature {
        value: value as f32,
        unit: BOM_API_TEMP_UNIT,
    };

    Ok(match CONFIG.render_options.temp_unit {
        TemperatureUnit::C => temp,
        TemperatureUnit::F => temp.to_fahrenheit(),
    })
}

pub fn de_temp_celsius_opt<'de, D>(deserializer: D) -> Result<Option<Temperature>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i16::deserialize(deserializer);
    if let Ok(value) = value {
        let temp = Temperature {
            value: value as f32,
            unit: BOM_API_TEMP_UNIT,
        };
        Ok(Some(match CONFIG.render_options.temp_unit {
            TemperatureUnit::C => temp,
            TemperatureUnit::F => temp.to_fahrenheit(),
        }))
    } else {
        Ok(None)
    }
}

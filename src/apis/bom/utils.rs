use serde::{Deserialize, Deserializer};

use crate::constants::BOM_API_TEMP_UNIT;

use super::models::Temperature;

// Deserializers stay config-free: BOM temperatures are parsed as raw Celsius
// and converted to the configured unit in the API->domain mapping layer.
pub fn de_temp_celsius<'de, D>(deserializer: D) -> Result<Temperature, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i16::deserialize(deserializer)?;
    Ok(Temperature {
        value: value as f32,
        unit: BOM_API_TEMP_UNIT,
    })
}

pub fn de_temp_celsius_opt<'de, D>(deserializer: D) -> Result<Option<Temperature>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i16::deserialize(deserializer);
    if let Ok(value) = value {
        Ok(Some(Temperature {
            value: value as f32,
            unit: BOM_API_TEMP_UNIT,
        }))
    } else {
        Ok(None)
    }
}

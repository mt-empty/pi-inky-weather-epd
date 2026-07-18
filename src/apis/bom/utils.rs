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
    // `Option<i16>::deserialize` treats a JSON `null` as `None` and otherwise
    // delegates to `i16::deserialize` — so a genuinely absent/null value
    // becomes `None`, but a present-and-malformed value (wrong JSON type)
    // propagates as an error instead of silently disappearing.
    let value = Option::<i16>::deserialize(deserializer)?;
    Ok(value.map(|value| Temperature {
        value: value as f32,
        unit: BOM_API_TEMP_UNIT,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn de_opt(json: &str) -> Option<Temperature> {
        let mut de = serde_json::Deserializer::from_str(json);
        de_temp_celsius_opt(&mut de).unwrap()
    }

    fn de_opt_result(json: &str) -> Result<Option<Temperature>, serde_json::Error> {
        let mut de = serde_json::Deserializer::from_str(json);
        de_temp_celsius_opt(&mut de)
    }

    #[test]
    fn valid_integer_becomes_some_temperature() {
        let temp = de_opt("20").unwrap();
        assert_eq!(temp.value, 20.0);
        assert_eq!(temp.unit, BOM_API_TEMP_UNIT);
    }

    #[test]
    fn negative_integer_becomes_some_temperature() {
        let temp = de_opt("-5").unwrap();
        assert_eq!(temp.value, -5.0);
    }

    #[test]
    fn null_becomes_none() {
        assert!(de_opt("null").is_none());
    }

    #[test]
    fn malformed_value_is_a_deserialize_error() {
        // A present-but-wrong-type value (e.g. BOM sending a string instead
        // of a number, due to an API bug or schema drift) must surface as a
        // parse error rather than silently becoming "temperature absent" —
        // masking a real upstream problem as ordinary missing data.
        assert!(de_opt_result(r#""not a number""#).is_err());
    }
}

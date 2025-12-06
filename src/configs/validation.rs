use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: Cow<'static, str>,
}

impl ValidationError {
    pub fn new(message: &'static str) -> ValidationError {
        ValidationError {
            message: Cow::Borrowed(message),
        }
    }
}

impl std::error::Error for ValidationError {
    fn description(&self) -> &str {
        &self.message
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// See this https://www.w3.org/TR/SVG11/types.html#ColorKeywords
const NAMED_COLOURS: [&str; 147] = [
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgreen",
    "darkgrey",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "grey",
    "green",
    "greenyellow",
    "honeydew",
    "hotpink",
    "indianred",
    "indigo",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgreen",
    "lightgrey",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "turquoise",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
];

const SPECIAL_COLOURS: [&str; 4] = ["currentColor", "inherit", "transparent", "initial"];

fn is_named_colour(colour: &str) -> bool {
    NAMED_COLOURS.contains(&colour)
}
fn is_hex_colour(colour: &str) -> bool {
    // This regex matches hex colours in the format "#FFF" or "#FFFFFF"
    let hex_colour_re = Regex::new(r"^#(?:[0-9a-fA-F]{3}){1,2}$").unwrap();
    hex_colour_re.is_match(colour)
}
fn is_rgb_colour(colour: &str) -> bool {
    let rgb_values: Vec<&str> = colour[4..colour.len() - 1].split(',').collect();
    if rgb_values.len() == 3 {
        for value in rgb_values {
            if let Ok(num) = value.trim().parse::<i32>() {
                if !(0..=255).contains(&num) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    } else {
        false
    }
}
fn is_rgba_colour(colour: &str) -> bool {
    // Check if the colour is in rgba format
    let rgba_values: Vec<&str> = colour[5..colour.len() - 1].split(',').collect();
    if rgba_values.len() == 4 {
        for value in &rgba_values[..3] {
            if let Ok(num) = value.trim().parse::<i32>() {
                if !(0..=255).contains(&num) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Ok(alpha) = rgba_values[3].trim().parse::<f32>() {
            if !(0.0..=1.0).contains(&alpha) {
                return false;
            }
        } else {
            return false;
        }
        true
    } else {
        false
    }
}

fn is_hsl_colour(colour: &str) -> bool {
    let hsl_values: Vec<&str> = colour[4..colour.len() - 1].split(',').collect();
    if hsl_values.len() == 3 {
        for value in &hsl_values[..2] {
            if let Ok(num) = value.trim().parse::<f32>() {
                if !(0.0..=360.0).contains(&num) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Ok(lightness) = hsl_values[2].trim().parse::<f32>() {
            if !(0.0..=1.0).contains(&lightness) {
                return false;
            }
        } else {
            return false;
        }
        true
    } else {
        false
    }
}
fn is_hsla_colour(colour: &str) -> bool {
    let hsla_values: Vec<&str> = colour[5..colour.len() - 1].split(',').collect();
    if hsla_values.len() == 4 {
        for value in &hsla_values[..2] {
            if let Ok(num) = value.trim().parse::<f32>() {
                if !(0.0..=360.0).contains(&num) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Ok(alpha) = hsla_values[3].trim().parse::<f32>() {
            if !(0.0..=1.0).contains(&alpha) {
                return false;
            }
        } else {
            return false;
        }
        true
    } else {
        false
    }
}

fn is_special_colour(colour: &str) -> bool {
    SPECIAL_COLOURS.contains(&colour)
}
pub fn is_valid_colour(colour: &str) -> Result<(), ValidationError> {
    let clean_colour = colour.trim().to_ascii_lowercase();

    if is_special_colour(&clean_colour)
        || is_named_colour(&clean_colour)
        || is_hex_colour(&clean_colour)
        || is_rgb_colour(&clean_colour)
        || is_rgba_colour(&clean_colour)
        || is_hsl_colour(&clean_colour)
        || is_hsla_colour(&clean_colour)
    {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid colour format"))
    }
}

pub fn is_valid_longitude(longitude: &f64) -> Result<(), ValidationError> {
    if (-180.0..=180.0).contains(longitude) {
        Ok(())
    } else {
        Err(ValidationError::new(
            "Longitude must be between -180.0 and 180.0",
        ))
    }
}

pub fn is_valid_latitude(latitude: &f64) -> Result<(), ValidationError> {
    if (-90.0..=90.0).contains(latitude) {
        Ok(())
    } else {
        Err(ValidationError::new(
            "Latitude must be between -90.0 and 90.0",
        ))
    }
}

/// Maximum allowed length for formatted date output.
/// This prevents overly long strings that won't fit on the e-paper display.
/// Based on longest reasonable format: "Wednesday, 28 September 2025" = 28 chars
/// We allow some extra room for custom text.
const MAX_DATE_FORMAT_OUTPUT_LENGTH: usize = 30;

/// Validates a chrono strftime date format string.
///
/// # Validation Rules
/// 1. Format string must not be empty or whitespace-only
/// 2. Formatted output (using longest possible date) must not exceed MAX_DATE_FORMAT_OUTPUT_LENGTH
///
/// Note: Invalid specifiers like `%Q` will be output literally by chrono's format().
/// This is acceptable - users will see the issue immediately on their display.
///
/// # Arguments
/// * `format` - A strftime format string (e.g., "%A, %d %B" or "%m/%d/%Y")
///
/// # Returns
/// * `Ok(())` if the format is valid
/// * `Err(ValidationError)` if validation fails
///
/// # Examples
/// ```
/// use pi_inky_weather_epd::configs::validation::is_valid_date_format;
///
/// assert!(is_valid_date_format("%A, %d %B").is_ok());      // "Saturday, 06 December"
/// assert!(is_valid_date_format("%m/%d/%Y").is_ok());       // "12/06/2025"
/// assert!(is_valid_date_format("%-d %b %Y").is_ok());      // "6 Dec 2025"
/// assert!(is_valid_date_format("").is_err());              // Empty string
/// ```
pub fn is_valid_date_format(format: &str) -> Result<(), ValidationError> {
    // Check for empty or whitespace-only format
    let trimmed = format.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::new(
            "Date format cannot be empty or whitespace-only",
        ));
    }

    // Test the format by formatting the longest possible date
    // Wednesday (9 chars) + September (9 chars) = longest day + month combination
    use chrono::{TimeZone, Utc};
    let longest_date = Utc.with_ymd_and_hms(2025, 9, 17, 12, 0, 0).unwrap(); // Wednesday, 17 September 2025
    let formatted = longest_date.format(trimmed).to_string();

    // Check output length
    if formatted.len() > MAX_DATE_FORMAT_OUTPUT_LENGTH {
        let message = format!(
            "Date format produces output that is too long for display, it must be {MAX_DATE_FORMAT_OUTPUT_LENGTH} characters or fewer"
        );
        return Err(ValidationError {
            message: Cow::Owned(message),
        });
    }

    Ok(())
}

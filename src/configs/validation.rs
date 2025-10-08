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
    NAMED_COLOURS.iter().any(|&c| c == colour)
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
    SPECIAL_COLOURS.iter().any(|&c| c == colour)
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

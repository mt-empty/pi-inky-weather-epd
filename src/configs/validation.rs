use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use serde::{Deserialize, Serialize};

use crate::i18n::{format_localized_date, Language};
use crate::utils::measure_text_ink_width;

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

const SPECIAL_COLOURS: [&str; 4] = ["currentcolor", "inherit", "transparent", "initial"];

fn is_named_colour(colour: &str) -> bool {
    NAMED_COLOURS.contains(&colour)
}
fn is_hex_colour(colour: &str) -> bool {
    // Matches hex colours in the format "#FFF" or "#FFFFFF"
    if !colour.starts_with('#') || (colour.len() != 4 && colour.len() != 7) {
        return false;
    } else {
        for c in colour.chars().skip(1) {
            if !c.is_ascii_hexdigit() {
                return false;
            }
        }
    }
    true
}
fn is_rgb_colour(colour: &str) -> bool {
    let Some(inner) = colour
        .strip_prefix("rgb(")
        .and_then(|s| s.strip_suffix(")"))
    else {
        return false;
    };
    let rgb_values: Vec<&str> = inner.split(',').collect();
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
    let Some(inner) = colour
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(")"))
    else {
        return false;
    };
    let rgba_values: Vec<&str> = inner.split(',').collect();
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
    let Some(inner) = colour
        .strip_prefix("hsl(")
        .and_then(|s| s.strip_suffix(")"))
    else {
        return false;
    };
    let hsl_values: Vec<&str> = inner.split(',').collect();
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
    let Some(inner) = colour
        .strip_prefix("hsla(")
        .and_then(|s| s.strip_suffix(")"))
    else {
        return false;
    };
    let hsla_values: Vec<&str> = inner.split(',').collect();
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

/// Widest the header date may render, in pixels (centred at x=400 on the
/// 800px canvas). Sized to keep every documented English format, the widest
/// being "Wednesday, September 27, 2023" at 503px; near this width the text
/// edges approach the current-weather icon and the "Incomplete Data" badge.
pub const HEADER_DATE_MAX_WIDTH_PX: u32 = 510;

/// Font of the header date — must match `current_day_date` in
/// `dashboard-template-min.svg`.
const HEADER_DATE_FONT_FAMILY: &str = "Roboto, sans-serif";
const HEADER_DATE_FONT_SIZE: f32 = 35.0;

/// Validates the syntax of a chrono strftime date format string. Whether it
/// fits the header depends on `render_options.language`, which this
/// field-level validator can't see; see [`widest_header_date_width`].
///
/// # Validation Rules
/// 1. Format string must not be empty or whitespace-only
/// 2. chrono must be able to render it
///
/// Note: Invalid specifiers like `%Q` will be output literally by chrono's format().
/// This is acceptable - users will see the issue immediately on their display.
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
    let trimmed = format.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::new(
            "Date format cannot be empty or whitespace-only",
        ));
    }

    use chrono::{TimeZone, Utc};
    use std::fmt::Write;
    let date = Utc.with_ymd_and_hms(2025, 9, 17, 12, 0, 0).unwrap();

    // Write into a buffer instead of calling `.to_string()`: chrono's
    // `DelayedFormat::fmt` returns `Err` for some malformed specifiers (e.g.
    // an unterminated "%{"), and `ToString::to_string()` panics on that
    // instead of propagating it.
    let mut formatted = String::new();
    if write!(formatted, "{}", date.format(trimmed)).is_err() {
        return Err(ValidationError::new(
            "Date format contains an invalid or unsupported specifier",
        ));
    }

    Ok(())
}

/// Rendered pixel width of the widest header `format` can produce in
/// `language`, measured through the dashboard's own font database. `None`
/// if nothing could be measured (e.g. fonts failed to load), in which case
/// callers should accept the format rather than reject it blind.
///
/// `format` must already have passed [`is_valid_date_format`]. Weekday and
/// month names are probed independently (widest weekday first, then every
/// month on that weekday) with two-digit days, which finds the widest
/// combination in 19 renders instead of all 84.
pub fn widest_header_date_width(format: &str, language: Language) -> Option<u32> {
    use chrono::{Datelike, NaiveDate, TimeZone, Utc};
    use std::collections::HashMap;

    let mut cache: HashMap<String, Option<u32>> = HashMap::new();
    let mut width_on = |date: NaiveDate| -> Option<u32> {
        let datetime = Utc.from_utc_datetime(&date.and_hms_opt(12, 0, 0)?);
        let rendered = format_localized_date(datetime, format.trim(), language);
        *cache.entry(rendered).or_insert_with_key(|text| {
            measure_text_ink_width(text, HEADER_DATE_FONT_FAMILY, HEADER_DATE_FONT_SIZE)
        })
    };

    // 22..=28 holds every weekday exactly once, all with two-digit days.
    let days =
        |month: u32| (22..=28).filter_map(move |day| NaiveDate::from_ymd_opt(2025, month, day));

    let mut widest: Option<u32> = None;
    let mut widest_weekday = None;
    for date in days(9) {
        if let Some(width) = width_on(date) {
            if widest.is_none_or(|w| width > w) {
                widest = Some(width);
                widest_weekday = Some(date.weekday());
            }
        }
    }
    let widest_weekday = widest_weekday?;
    for month in 1..=12 {
        if let Some(date) = days(month).find(|d| d.weekday() == widest_weekday) {
            if let Some(width) = width_on(date) {
                widest = Some(widest.map_or(width, |w| w.max(width)));
            }
        }
    }
    widest
}

/// Tests for the configurable date format feature: verifies that users can
/// configure the date display format using strftime format strings.
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use strum::IntoEnumIterator;

    /// Formats a fixed date (Saturday, December 6, 2025) with the given format string.
    fn format_test_date(format: &str) -> String {
        let test_date = Utc.with_ymd_and_hms(2025, 12, 6, 10, 30, 0).unwrap();
        test_date.format(format).to_string()
    }

    /// Formats the longest possible date (Sunday, 28 September 2025 —
    /// longest day + month name combination).
    fn format_longest_date(format: &str) -> String {
        let test_date = Utc.with_ymd_and_hms(2025, 9, 28, 10, 30, 0).unwrap();
        test_date.format(format).to_string()
    }

    mod regional_formats {
        use super::*;

        #[test]
        fn australian_day_month_year() {
            let format = "%d/%m/%Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "06/12/2025");
        }

        #[test]
        fn american_month_day_year() {
            let format = "%m/%d/%Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "12/06/2025");
        }

        #[test]
        fn japanese_year_month_day() {
            let format = "%Y/%m/%d";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "2025/12/06");
        }

        #[test]
        fn iso8601() {
            let format = "%Y-%m-%d";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "2025-12-06");
        }
    }

    mod separator_styles {
        use super::*;

        #[test]
        fn slash_separator() {
            let format = "%d/%m/%Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "06/12/2025");
        }

        #[test]
        fn dot_separator() {
            // German style: DD.MM.YYYY
            let format = "%d.%m.%Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "06.12.2025");
        }

        #[test]
        fn hyphen_separator() {
            let format = "%d-%m-%Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "06-12-2025");
        }

        #[test]
        fn space_separator() {
            let format = "%d %m %Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "06 12 2025");
        }
    }

    mod written_language_styles {
        use super::*;

        #[test]
        fn long_full_weekday_day_full_month() {
            let format = "%A, %-d %B %Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "Saturday, 6 December 2025");
        }

        #[test]
        fn long_full_month_day_year() {
            // US written style: December 6, 2025
            let format = "%B %-d, %Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "December 6, 2025");
        }

        #[test]
        fn short_abbreviated_weekday_day_month() {
            let format = "%a, %-d %b %Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "Sat, 6 Dec 2025");
        }

        #[test]
        fn short_day_abbreviated_month() {
            let format = "%-d %b";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "6 Dec");
        }

        #[test]
        fn short_abbreviated_month_day() {
            let format = "%b %-d";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "Dec 6");
        }

        #[test]
        fn current_default_format() {
            // Saturday, 06 December (no year)
            let format = "%A, %d %B";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "Saturday, 06 December");
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn short_year() {
            let format = "%d/%m/%y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "06/12/25");
        }

        #[test]
        fn weekday_only() {
            let format = "%A";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "Saturday");
        }

        #[test]
        fn custom_text_mixed_with_date() {
            let format = "Today is %A";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_test_date(format), "Today is Saturday");
        }
    }

    mod validation_errors {
        use super::*;

        #[test]
        fn empty_string_is_invalid() {
            assert!(is_valid_date_format("").is_err());
        }

        #[test]
        fn whitespace_only_is_invalid() {
            assert!(is_valid_date_format("   ").is_err());
        }

        #[test]
        fn longest_reasonable_date_is_valid() {
            // Sunday, 28 September 2025 — longest day + month name combination,
            // a reasonable max length.
            let format = "%A, %-d %B %Y";
            assert!(is_valid_date_format(format).is_ok());
            assert_eq!(format_longest_date(format), "Sunday, 28 September 2025");
        }
    }

    mod header_width {
        use super::*;
        use chrono::NaiveDate;

        fn fits(format: &str, language: Language) -> bool {
            let width = widest_header_date_width(format, language)
                .unwrap_or_else(|| panic!("could not measure {format:?} for {language:?}"));
            width <= HEADER_DATE_MAX_WIDTH_PX
        }

        /// Documented English formats must never fall back to the default.
        #[test]
        fn formats_accepted_before_still_fit_for_english() {
            for format in [
                "%A, %d %B",
                "%B %-d, %Y",
                "%d/%m/%Y",
                "%m/%d/%Y",
                "%Y-%m-%d",
                "%a, %-d %b",
                "%A, %-d %B %Y",
                "%A, %d %B %Y",
                "%A, %B %-d, %Y",
            ] {
                assert!(fits(format, Language::En), "{format:?} no longer fits");
            }
        }

        #[test]
        fn every_language_default_fits() {
            for language in Language::iter() {
                assert!(
                    fits(language.default_date_format(), language),
                    "{language:?} default {:?} is too wide",
                    language.default_date_format()
                );
            }
        }

        #[test]
        fn far_too_long_format_does_not_fit() {
            let long_format = "%A, %B %-d, %Y - %A, %B %-d, %Y - %A, %B %-d, %Y";
            assert!(!fits(long_format, Language::En));
        }

        /// The fast probe (widest weekday, then every month on it) agrees
        /// with measuring all 84 weekday × month combinations.
        #[test]
        fn widest_probe_matches_exhaustive_search() {
            for (format, language) in [
                ("%A, %-d %B %Y", Language::Pl),
                ("%a %d %b", Language::Ru),
                ("%A, %B %-d, %Y", Language::En),
            ] {
                let exhaustive = (1..=12)
                    .flat_map(|month| {
                        (22..=28).filter_map(move |day| NaiveDate::from_ymd_opt(2025, month, day))
                    })
                    .map(|date| {
                        let datetime = Utc.from_utc_datetime(&date.and_hms_opt(12, 0, 0).unwrap());
                        let text = format_localized_date(datetime, format, language);
                        measure_text_ink_width(
                            &text,
                            HEADER_DATE_FONT_FAMILY,
                            HEADER_DATE_FONT_SIZE,
                        )
                        .unwrap()
                    })
                    .max();
                assert_eq!(
                    widest_header_date_width(format, language),
                    exhaustive,
                    "{format:?} {language:?}"
                );
            }
        }
    }

    mod fuzzing {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// `format` is user-controlled config, not API data, but it still
            /// reaches chrono's `strftime`-style formatter with no prior
            /// sanitization beyond the empty/whitespace check — arbitrary
            /// strings (unicode, unmatched `%` sequences, huge inputs) should
            /// never panic, only return `Ok`/`Err`.
            #[test]
            fn never_panics_on_arbitrary_input(format in ".*") {
                let _ = is_valid_date_format(&format);
            }

            /// `format_localized_date` swaps `%A`/`%a`/`%B`/`%b` for translated
            /// names after chrono runs; for any syntactically valid format
            /// that substitution must not make rendering panic in any language.
            #[test]
            fn valid_formats_render_in_every_language(
                format in date_format_strategy()
            ) {
                if is_valid_date_format(&format).is_ok() {
                    let date = Utc.with_ymd_and_hms(2023, 9, 27, 10, 30, 0).unwrap();
                    for language in Language::iter() {
                        let _ = format_localized_date(date, format.trim(), language);
                    }
                }
            }
        }

        proptest! {
            /// `colour` is user-controlled config, hand-checked byte-by-byte
            /// (no regex) since the regex crate was dropped — arbitrary
            /// strings (empty, too short, unicode) must never panic, only
            /// return `Ok`/`Err`.
            #[test]
            fn colour_never_panics_on_arbitrary_input(colour in ".*") {
                let _ = is_valid_colour(&colour);
            }
        }

        /// Realistic date-format tokens (not fully arbitrary strings — see
        /// `never_panics_on_arbitrary_input` above for that), biased toward
        /// combinations that actually exercise %A/%B and %%-escaping.
        fn date_format_strategy() -> impl Strategy<Value = String> {
            let token = prop_oneof![
                Just("%A".to_string()),
                Just("%a".to_string()),
                Just("%B".to_string()),
                Just("%b".to_string()),
                Just("%%A".to_string()),
                Just("%%B".to_string()),
                Just("%d".to_string()),
                Just("%-d".to_string()),
                Just("%m".to_string()),
                Just("%Y".to_string()),
                Just(", ".to_string()),
                Just(" ".to_string()),
                Just("-".to_string()),
            ];
            proptest::collection::vec(token, 1..8).prop_map(|tokens| tokens.concat())
        }
    }
}

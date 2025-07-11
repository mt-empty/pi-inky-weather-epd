use anyhow::Error;
use anyhow::Result;
use chrono::Local;
use chrono::NaiveDate;
use chrono::TimeZone;
use chrono::{DateTime, NaiveDateTime, Utc};
use resvg::tiny_skia;
use resvg::usvg;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use usvg::fontdb;

/// Converts an SVG file to a PNG file.
///
/// # Arguments
///
/// * `input_path` - Path to the input SVG file.
/// * `output_path` - Path to save the output PNG file.
/// * `scale_factor` - The scale factor to apply to the SVG.
///
/// # Returns
///
/// * `Result<(), Error>` - Ok(()) if successful, or an error message.
pub fn convert_svg_to_png(
    input_path: &PathBuf,
    output_path: &PathBuf,
    scale_factor: f32,
) -> Result<(), Error> {
    // Read the SVG file
    let svg_data = fs::read_to_string(input_path)
        .map_err(|e| Error::msg(format!("Failed to read SVG file: {}", e)))?;

    let mut font_db = fontdb::Database::new();
    load_fonts(&mut font_db);

    // Parse the SVG
    let opts = usvg::Options {
        fontdb: font_db.into(),
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(&svg_data, &opts)
        .map_err(|e| Error::msg(format!("Failed to parse SVG: {}", e)))?;

    // Create a higher resolution canvas
    let pixmap_size = tree.size().to_int_size();
    let width = (pixmap_size.width() as f32 * scale_factor) as u32;
    let height = (pixmap_size.height() as f32 * scale_factor) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| Error::msg("Failed to create pixmap"))?;

    // Create a transform that scales the SVG
    let transform = tiny_skia::Transform::from_scale(scale_factor, scale_factor);

    // Render SVG onto the canvas with scaling
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Save the PNG file
    pixmap
        .save_png(output_path)
        .map_err(|e| Error::msg(format!("Failed to save PNG: {}", e)))?;

    Ok(())
}

/// Loads fonts into the provided font database.
///
/// # Arguments
///
/// * `font_db` - A mutable reference to a `fontdb::Database` to load fonts into.
fn load_fonts(font_db: &mut fontdb::Database) {
    font_db.load_system_fonts();

    // print current path
    let current_path = std::env::current_dir().unwrap();

    let font_files = [
        "static/fonts/Roboto-VariableFont_wdth,wght.ttf",
        "static/fonts/Roboto-Italic-VariableFont_wdth,wght.ttf",
        "static/fonts/Roboto-Regular-Dashed.ttf",
    ];

    for file in &font_files {
        match font_db.load_font_file(current_path.join(file)) {
            Ok(_) => println!("Loaded font file: {}", file),
            Err(e) => eprintln!("Failed to load font file: {}", e),
        }
    }
}

/// Converts a UTC timestamp string to the local date.
///
/// # Arguments
///
/// * `utc_time` - A string slice representing the UTC time in ISO 8601 format (e.g., "2024-12-26T12:00:00Z").
///
/// # Returns
///
/// * `Ok(NaiveDate)` - The corresponding local date as `NaiveDate` if the conversion succeeds.
/// * `Err(chrono::ParseError)` - If the input string cannot be parsed into a valid `NaiveDateTime`.
pub fn convert_utc_to_local_date(utc_time: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDateTime::parse_from_str(utc_time, "%Y-%m-%dT%H:%M:%SZ").map(|datetime| {
        // Convert UTC NaiveDateTime to Local timezone DateTime
        Local.from_utc_datetime(&datetime).date_naive()
    })
}

/// Converts a UTC timestamp string to the local date and time.
///
/// # Arguments
///
/// * `utc_time` - A string slice representing the UTC time in ISO 8601 format (e.g., "2024-12-26T12:00:00Z").
///
/// # Returns
///
/// * `Ok(NaiveDateTime)` - The corresponding local date and time as `NaiveDateTime` if the conversion succeeds.
/// * `Err(chrono::ParseError)` - If the input string cannot be parsed into a valid `NaiveDateTime`.
pub fn convert_utc_to_local_datetime(utc_time: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(utc_time, "%Y-%m-%dT%H:%M:%SZ").map(|datetime| {
        // Convert UTC NaiveDateTime to Local timezone DateTime
        Local.from_utc_datetime(&datetime).naive_local()
    })
}

/// Calculates the total value between two dates from a dataset.
///
/// # Arguments
///
/// * `data` - A slice of data items.
/// * `start_date` - The start date as `DateTime<TZ>`.
/// * `end_date` - The end date as `DateTime<TZ>`.
/// * `get_value` - A function to extract the value from a data item.
/// * `get_time` - A function to extract the time from a data item.
///
/// # Returns
///
/// * `V` - The total value between the specified dates.
pub fn get_total_between_dates<T, V, TZ: TimeZone>(
    data: &[T],
    start_date: &DateTime<TZ>,
    end_date: &DateTime<TZ>,
    get_value: impl Fn(&T) -> V,
    get_time: impl Fn(&T) -> DateTime<TZ>,
) -> V
where
    V: std::iter::Sum + Default,
{
    data.iter()
        .filter_map(|item| {
            let item_date = &get_time(item);
            if item_date >= start_date && item_date < end_date {
                Some(get_value(item))
            } else {
                None
            }
        })
        .sum()
}

/// Finds the maximum value between two dates from a dataset.
///
/// # Arguments
///
/// * `data` - A slice of data items.
/// * `start_date` - The start date as `DateTime<TZ>`.
/// * `end_date` - The end date as `DateTime<TZ>`, not inclusive.
/// * `get_value` - A function to extract the value from a data item.
/// * `get_time` - A function to extract the time from a data item.
///
/// # Returns
///
/// * `V` - The maximum value between the specified dates.
pub fn find_max_item_between_dates<T, V, TZ: TimeZone>(
    data: &[T],
    start_date: &DateTime<TZ>,
    end_date: &DateTime<TZ>,
    get_value: impl Fn(&T) -> V,
    get_time: impl Fn(&T) -> DateTime<TZ>,
) -> V
where
    V: PartialOrd + Copy + Default,
{
    // Use V::default() as the initial value for finding the maximum, it should be fine for numeric types here since they are all positive
    data.iter()
        .filter_map(|item| {
            let date = &get_time(item);
            if date >= start_date && date < end_date {
                Some(get_value(item))
            } else {
                None
            }
        })
        .fold(V::default(), |acc, x| if x > acc { x } else { acc })
}

/// Deserializes an optional NaiveDateTime from a string.
///
/// # Arguments
///
/// * `deserializer` - The deserializer to use.
///
/// # Returns
///
/// * `Result<Option<NaiveDateTime>, D::Error>` - The deserialized `NaiveDateTime` or an error.
pub fn deserialize_optional_naive_date<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    if let Some(date_str) = opt {
        NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%SZ")
            .map(|dt| Some(Local.from_utc_datetime(&dt).naive_local()))
            .map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}

/// Deserializes a NaiveDateTime from a string.
///
/// # Arguments
///
/// * `s` - The deserializer to use.
///
/// # Returns
///
/// * `Result<NaiveDateTime, S::Error>` - The deserialized `NaiveDateTime` or an error.
pub fn deserialize_naive_date<'de, S>(s: S) -> Result<NaiveDateTime, S::Error>
where
    S: serde::de::Deserializer<'de>,
{
    let date_str: &str = serde::Deserialize::deserialize(s)?;
    NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%SZ")
        .map(|dt| Local.from_utc_datetime(&dt).naive_local())
        .map_err(serde::de::Error::custom)
}

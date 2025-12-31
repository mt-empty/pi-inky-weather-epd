use crate::errors::GeohashError;
use crate::logger;
use anyhow::Error;
use anyhow::Result;
use chrono::Local;
use chrono::TimeZone;
use chrono::{DateTime, NaiveDateTime};
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
        .map_err(|e| Error::msg(format!("Failed to read SVG file: {e}")))?;

    let mut font_db = fontdb::Database::new();
    load_fonts(&mut font_db);

    // Parse the SVG
    let opts = usvg::Options {
        fontdb: font_db.into(),
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(&svg_data, &opts)
        .map_err(|e| Error::msg(format!("Failed to parse SVG: {e}")))?;

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
        .map_err(|e| Error::msg(format!("Failed to save PNG: {e}")))?;

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
            Ok(_) => {}
            Err(e) => logger::warning(format!("Failed to load font file: {e}")),
        }
    }
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

// Below code was adopted from Geohash crate
// https://github.com/georust/geohash/blob/main/src/core.rs

// the alphabet for the base32 encoding used in geohashing
#[rustfmt::skip]
const BASE32_CODES: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

// bit shifting functions used in encoding and decoding

// spread takes a u32 and deposits its bits into the evenbit positions of a u64
#[inline]
fn spread(x: u32) -> u64 {
    let mut new_x = x as u64;
    new_x = (new_x | (new_x << 16)) & 0x0000ffff0000ffff;
    new_x = (new_x | (new_x << 8)) & 0x00ff00ff00ff00ff;
    new_x = (new_x | (new_x << 4)) & 0x0f0f0f0f0f0f0f0f;
    new_x = (new_x | (new_x << 2)) & 0x3333333333333333;
    new_x = (new_x | (new_x << 1)) & 0x5555555555555555;

    new_x
}

// spreads the inputs, then shifts the y input and does a bitwise or to fill the remaining bits in x
#[inline]
fn interleave(x: u32, y: u32) -> u64 {
    spread(x) | (spread(y) << 1)
}

/// Encode a coordinate to a geohash with length `len`.
///
/// # Arguments
///
/// * `lon_x` - The longitude (x coordinate) in degrees, must be in range [-180, 180]
/// * `lat_y` - The latitude (y coordinate) in degrees, must be in range [-90, 90]
/// * `len` - The desired length of the geohash string (1-12)
///
/// # Examples
///
/// Encoding a coordinate to a length five geohash:
///
/// ```ignore
/// let geohash_string = encode(-120.6623, 35.3003, 5).expect("Invalid coordinate");
/// assert_eq!(geohash_string, "9q60y");
/// ```
///
/// Encoding a coordinate to a length ten geohash:
///
/// ```ignore
/// let geohash_string = encode(-120.6623, 35.3003, 10).expect("Invalid coordinate");
/// assert_eq!(geohash_string, "9q60y60rhs");
/// ```
pub fn encode(lon_x: f64, lat_y: f64, len: usize) -> Result<String, GeohashError> {
    let max_lat = 90f64;
    let min_lat = -90f64;
    let max_lon = 180f64;
    let min_lon = -180f64;

    if !(min_lon..=max_lon).contains(&lon_x) || !(min_lat..=max_lat).contains(&lat_y) {
        return Err(GeohashError::InvalidCoordinateRange(lon_x, lat_y));
    }

    if !(1..=12).contains(&len) {
        return Err(GeohashError::InvalidLength(len));
    }

    // divides the latitude by 180, then adds 1.5 to give a value between 1 and 2
    // then we take the first 32 bits of the significand as a u32
    let lat32 = ((lat_y * 0.005555555555555556 + 1.5).to_bits() >> 20) as u32;
    // same as latitude, but a division by 360 instead of 180
    let lon32 = ((lon_x * 0.002777777777777778 + 1.5).to_bits() >> 20) as u32;

    let mut interleaved_int = interleave(lat32, lon32);

    let mut out = String::with_capacity(len);
    // loop through and take the first 5 bits of the interleaved value ech iteration
    for _ in 0..len {
        // shifts so that the high 5 bits are now the low five bits, then masks to get their value
        let code = (interleaved_int >> 59) as usize & (0x1f);
        // uses that value to index into the array of base32 codes
        out.push(BASE32_CODES[code]);
        // shifts the interleaved bits left by 5, so we get the next 5 bits on the next iteration
        interleaved_int <<= 5;
    }
    Ok(out)
}

// Finish Geohash crate code

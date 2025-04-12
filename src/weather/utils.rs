use chrono::Datelike;

use crate::CONFIG;

pub fn get_moon_phase_icon_path() -> String {
    let now = chrono::Local::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();

    // Calculate the approximate age of the moon in days since the last new moon
    let mut moon_age_days = ((year as f64 - 2000.0) * 365.25 + month as f64 * 30.6 + day as f64
        - 2451550.1)
        % 29.530588;
    if moon_age_days < 0.0 {
        moon_age_days += 29.530588; // Ensure positive values
    }

    // Determine the moon phase icon based on the moon age
    let icon_name = match moon_age_days {
        age if age < 1.84566 => "moon-new.svg",
        age if age < 5.53699 => "moon-waxing-crescent.svg",
        age if age < 9.22831 => "moon-first-quarter.svg",
        age if age < 12.91963 => "moon-waxing-gibbous.svg",
        age if age < 16.61096 => "moon-full.svg",
        age if age < 20.30228 => "moon-waning-gibbous.svg",
        age if age < 23.99361 => "moon-last-quarter.svg",
        _ => "moon-waning-crescent.svg",
    };

    format!("{}{}", CONFIG.misc.svg_icons_directory, icon_name)
}

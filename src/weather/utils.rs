use chrono::Datelike;
use strum_macros::Display;

// Determine the moon phase icon based on the moon age
#[derive(Debug, Display)]
pub enum MoonPhaseIconName {
    #[strum(to_string = "moon-new.svg")]
    New,
    #[strum(to_string = "moon-waxing-crescent.svg")]
    WaxingCrescent,
    #[strum(to_string = "moon-first-quarter.svg")]
    FirstQuarter,
    #[strum(to_string = "moon-waxing-gibbous.svg")]
    WaxingGibbous,
    #[strum(to_string = "moon-full.svg")]
    Full,
    #[strum(to_string = "moon-waning-gibbous.svg")]
    WaningGibbous,
    #[strum(to_string = "moon-last-quarter.svg")]
    LastQuarter,
    #[strum(to_string = "moon-waning-crescent.svg")]
    WaningCrescent,
}

pub fn get_moon_phase_icon_name() -> MoonPhaseIconName {
    let now = chrono::Local::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();

    // Calculate the approximate age of the moon in days since the last new moon
    let mut moon_age_days = ((year as f32 - 2000.0) * 365.25 + month as f32 * 30.6 + day as f32
        - 2451550.1)
        % 29.530588;
    if moon_age_days < 0.0 {
        moon_age_days += 29.530588; // Ensure positive values
    }

    // Determine the moon phase icon based on the moon age
    match moon_age_days {
        age if age < 1.84566 => MoonPhaseIconName::New,
        age if age < 5.53699 => MoonPhaseIconName::WaxingCrescent,
        age if age < 9.22831 => MoonPhaseIconName::FirstQuarter,
        age if age < 12.91963 => MoonPhaseIconName::WaxingGibbous,
        age if age < 16.61096 => MoonPhaseIconName::Full,
        age if age < 20.30228 => MoonPhaseIconName::WaningGibbous,
        age if age < 23.99361 => MoonPhaseIconName::LastQuarter,
        _ => MoonPhaseIconName::WaningCrescent,
    }
}

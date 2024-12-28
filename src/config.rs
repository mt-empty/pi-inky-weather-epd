use serde::Deserialize;

#[derive(Debug, Deserialize)]

pub struct DashboardConfig {
    pub release: Release,
    pub api: Api,
    pub colours: Colours,
    pub misc: Misc,
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub download_release_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Api {
    pub update_interval_mins: u32,
    pub location: String,
}

#[derive(Debug, Deserialize)]
pub struct Colours {
    pub background_colour: String,
    pub text_colour: String,
    pub x_axis_colour: String,
    pub y_left_axis_colour: String,
    pub y_right_axis_colour: String,
    pub temp_colour: String,
    pub feels_like_colour: String,
    pub rain_colour: String,
}

#[derive(Debug, Deserialize)]
pub struct Misc {
    pub store_local: bool,
    pub store_local_path: String,
    pub template_path: String,
    pub modified_template_name: String,
    pub icon_path: String,
    pub unit: String,
    pub use_moon_phase_instead_of_clear_night: bool,
    pub use_online_data: bool,
}

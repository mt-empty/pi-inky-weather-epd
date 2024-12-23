use serde::Deserialize;

#[derive(Debug, Deserialize)]

pub struct DashboardConfig {
    pub release: Release,
    pub api: Api,
    pub dashboard: Dashboard,
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
    pub use_online_data: bool,
}

#[derive(Debug, Deserialize)]
pub struct Dashboard {
    pub background_color: String,
    pub text_color: String,
    pub x_axis_color: String,
    pub y_left_axis_color: String,
    pub y_right_axis_color: String,
    pub temp_color: String,
    pub feels_like_color: String,
    pub rain_color: String,
}

#[derive(Debug, Deserialize)]
pub struct Misc {
    pub store_local: bool,
    pub store_local_path: String,
    pub template_path: String,
    pub modified_template_name: String,
    pub modified_template_dir_path: String,
    pub icon_path: String,
    pub unit: String,
    pub use_moon_phase_instead_of_clear_night: bool,
}

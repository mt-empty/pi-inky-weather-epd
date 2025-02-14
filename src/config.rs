use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DashboardConfig {
    pub release: Release,
    pub api: Api,
    pub colours: Colours,
    pub misc: Misc,
    pub render_options: RenderOptions,
    pub debugging: Debugging,
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub release_info_url: String,
    pub download_base_url: String,
    pub auto_update: bool,
    pub update_interval_days: i64,
}

#[derive(Debug, Deserialize)]
pub struct Api {
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
    pub weather_data_store_path: String,
    pub template_path: String,
    pub modified_template_name: String,
    pub generated_png_name: String,
    pub svg_icons_directory: String,
    pub python_script_path: String,
    pub python_path: String,
}

#[derive(Debug, Deserialize)]
pub struct RenderOptions {
    pub saturation: f32,
    pub temp_unit: String,
    pub use_moon_phase_instead_of_clear_night: bool,
    pub x_axis_always_at_min: bool,
}

#[derive(Debug, Deserialize)]
pub struct Debugging {
    pub disable_network_requests: bool,
    pub disable_png_output: bool,
    pub disable_drawing_on_epd: bool,
}

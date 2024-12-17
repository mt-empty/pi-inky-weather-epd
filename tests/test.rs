use pi_inky_weather_epd::{generate_weather_dashboard, MODIFIED_TEMPLATE_PATH};
use std::fs;

#[test]
fn test_weather_dashboard_ok() {
    let result = generate_weather_dashboard();
    assert!(result.is_ok());
}

#[test]
fn produced_svg_ok() {
    let result = generate_weather_dashboard();
    assert!(result.is_ok());

    let svg_content = fs::read_to_string(MODIFIED_TEMPLATE_PATH).expect("Failed to read SVG file");
    let svg_tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default());

    assert!(svg_tree.is_ok(), "The file is not a valid SVG");
}

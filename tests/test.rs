use pi_inky_weather_epd::{generate_weather_dashboard_wrapper, CONFIG};
use std::fs;

#[test]
fn test_weather_dashboard_ok() {
    let result = generate_weather_dashboard_wrapper();
    assert!(result.is_ok());
}

#[test]
fn template_svg_ok() {
    let svg_content =
        fs::read_to_string(CONFIG.misc.template_path.clone()).expect("Failed to read SVG file");
    let svg_tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default());

    assert!(svg_tree.is_ok(), "The file is not a valid SVG");
}

#[test]
fn produced_svg_ok() {
    let result = generate_weather_dashboard_wrapper();
    assert!(result.is_ok());

    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read SVG file");
    let svg_tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default());

    assert!(svg_tree.is_ok(), "The file is not a valid SVG");
}

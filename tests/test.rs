use pi_inky_weather_epd::{generate_weather_dashboard_wrapper, CONFIG};
use std::fs;

#[test]
fn base_template_svg_ok() {
    let svg_content = fs::read_to_string(CONFIG.misc.template_path.clone())
        .expect("Failed to read the  base template SVG file");
    let svg_tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default());

    assert!(
        svg_tree.is_ok(),
        "The base template file is not a valid SVG"
    );
}

#[test]
fn produced_svg_ok() {
    let result = generate_weather_dashboard_wrapper();
    assert!(result.is_ok(), "Failed to generate the SVG");

    let svg_content = fs::read_to_string(CONFIG.misc.generated_svg_name.clone())
        .expect("Failed to read SVG file");
    let svg_tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default());

    assert!(svg_tree.is_ok(), "The generated file is not a valid SVG");

    let png_file = CONFIG.misc.generated_png_name.clone();
    assert!(
        fs::metadata(&png_file).is_ok(),
        "The generated PNG file was not created"
    );
}

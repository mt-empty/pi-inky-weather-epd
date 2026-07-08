mod helpers;

use helpers::test_utils;
use std::fs;

/// Validates that the base SVG template file is valid
///
/// This test ensures the template file can be parsed as valid SVG.
/// It doesn't test rendering - see snapshot_provider_test.rs for E2E tests.
#[test]
fn base_template_svg_ok() {
    let settings = test_utils::test_settings(|_| {});
    let svg_content = fs::read_to_string(&settings.misc.template_path)
        .expect("Failed to read the base template SVG file");
    let svg_tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default());

    assert!(
        svg_tree.is_ok(),
        "The base template file is not a valid SVG"
    );
}

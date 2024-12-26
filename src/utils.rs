use resvg::tiny_skia;
use resvg::usvg;
use std::fs;
use std::path::PathBuf;

pub fn has_write_permission(path: PathBuf) -> Result<bool, std::io::Error> {
    match fs::create_dir_all(path.to_owned()) {
        Ok(()) => {
            let metadata = fs::metadata(path.clone())?;
            Ok(!metadata.permissions().readonly())
        }
        Err(error) => Err(error),
    }
}

/// Converts an SVG file to a PNG file.
///
/// # Arguments
///
/// * `input_path` - Path to the input SVG file.
/// * `output_path` - Path to save the output PNG file.
///
/// # Returns
///
/// * `Result<(), String>` - Ok(()) if successful, or an error message.
pub fn convert_svg_to_png(input_path: &str, output_path: &str) -> Result<(), String> {
    // Read the SVG file
    let svg_data =
        fs::read_to_string(input_path).map_err(|e| format!("Failed to read SVG file: {}", e))?;

    // Parse the SVG
    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg_data, &opts)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    // Create a canvas
    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
        .ok_or_else(|| "Failed to create pixmap".to_string())?;

    // Render SVG onto the canvas
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Save the PNG file
    pixmap
        .save_png(output_path)
        .map_err(|e| format!("Failed to save PNG: {}", e))?;

    Ok(())
}

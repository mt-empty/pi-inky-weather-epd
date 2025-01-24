use std::process::Command;

use pi_inky_weather_epd::CONFIG;

pub fn pimironi_image_py() {
    let output = Command::new(CONFIG.misc.python_path.clone())
        .arg(CONFIG.misc.python_script_path.clone())
        .arg(CONFIG.misc.modified_template_name.clone())
        .output()
        .expect("Failed to execute Pimironi script");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Script output:\n{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Script error:\n{}", stderr);
    }
}

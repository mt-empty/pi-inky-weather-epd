use std::process::Command;

use crate::CONFIG;

/// Invokes the Pimironi image generation script using the Python interpreter specified in the configuration.
///
/// This function constructs a command to run the Python script with the necessary arguments and executes it.
/// It captures the output of the script and prints it to the standard output if the script runs successfully.
/// If the script fails, it prints the error output to the standard error and returns an error.
///
/// # Panics
///
/// Panics if the command to execute the script cannot be spawned.
///
/// # Errors
///
/// This function will return an error if the script execution fails.
///
/// # Returns
///
/// * `Ok(())` if the script executes successfully.
/// * `Err(anyhow::Error)` if the script execution fails.
pub fn invoke_pimironi_image_script() -> Result<(), anyhow::Error> {
    let output = Command::new(CONFIG.misc.python_path.clone())
        .arg(CONFIG.misc.python_script_path.clone())
        .arg(CONFIG.misc.generated_png_name.clone())
        .output()
        .expect("Failed to execute Pimironi script");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Script output:\n{}", stdout);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Script error:\n{}", stderr);
        Err(anyhow::anyhow!("Failed to execute Pimironi script"))
    }
}

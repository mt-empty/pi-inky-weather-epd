use crate::clock::{Clock, SystemClock};
use crate::dashboard::context::{Context, ContextBuilder};
use crate::errors::{DashboardError, Description};
use crate::logger;
use crate::providers::factory::create_provider;
use crate::update::read_last_update_status;
use crate::{utils, CONFIG};
use anyhow::Error;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tinytemplate::{format_unescaped, TinyTemplate};
pub use utils::*;

fn update_forecast_context(
    context_builder: &mut ContextBuilder,
    clock: &dyn Clock,
) -> Result<(), Error> {
    let provider = create_provider()?;
    let mut warnings: Vec<DashboardError> = Vec::new();

    // Check if the last update failed and add warning if so
    if let Some(error_details) = read_last_update_status() {
        warnings.push(DashboardError::UpdateFailed {
            details: error_details,
        });
    }

    logger::subsection(format!("Using provider: {}", provider.provider_name()));

    logger::subsection("Fetching daily forecast");
    let daily_result = provider.fetch_daily_forecast()?;
    if let Some(warning) = daily_result.warning {
        logger::warning(format!(
            "Using cached data due to: {}",
            warning.long_description()
        ));
        warnings.push(warning);
    } else {
        logger::success("Daily forecast retrieved");
    }
    context_builder.with_daily_forecast_data(daily_result.data, clock);

    std::thread::sleep(Duration::from_secs(3));

    logger::subsection("Fetching hourly forecast");
    let hourly_result = provider.fetch_hourly_forecast()?;
    if let Some(warning) = hourly_result.warning {
        logger::warning(format!(
            "Using cached data due to: {}",
            warning.long_description()
        ));
        warnings.push(warning);
    } else {
        logger::success("Hourly forecast retrieved");
    }
    context_builder.with_hourly_forecast_data(hourly_result.data, clock);

    // Add all accumulated warnings to the context
    for warning in warnings {
        context_builder.with_warning(warning);
    }

    Ok(())
}

fn render_dashboard_template(
    context: &Context,
    dashboard_svg: String,
    output_svg_name: &Path,
) -> Result<(), Error> {
    let mut tt = TinyTemplate::new();
    let tt_name = "dashboard";

    if let Err(e) = tt.add_template(tt_name, &dashboard_svg) {
        logger::error(format!("Failed to add template: {e}"));
        return Err(e.into());
    }
    tt.set_default_formatter(&format_unescaped);
    // Attempt to render the template
    match tt.render(tt_name, &context) {
        Ok(rendered) => {
            let mut output = fs::File::create(output_svg_name)?;
            output.write_all(rendered.as_bytes())?;
            Ok(())
        }
        Err(e) => {
            logger::error(format!("Failed to render template: {e}"));
            Err(e.into())
        }
    }
}

/// Generate weather dashboard using the system clock (production)
pub fn generate_weather_dashboard() -> Result<(), Error> {
    let clock = SystemClock;
    let input_template_name = &CONFIG.misc.template_path;
    let output_svg_name = &CONFIG.misc.generated_svg_name;
    generate_weather_dashboard_injection(&clock, input_template_name, output_svg_name)
}

/// Generate weather dashboard with a custom clock and custom paths  (for testing)
///
/// This function allows dependency injection of a Clock implementation and custom paths,
/// enabling deterministic testing with FixedClock.
///
/// # Arguments
///
/// * `clock` - The clock implementation to use for time-dependent operations
/// * `input_template_name` - Path to the input SVG template file
/// * `output_svg_name` - Path to save the generated SVG file
///
/// # Examples
///
/// ```ignore
/// use pi_inky_weather_epd::clock::FixedClock;
///
/// let input_template_name = std::path::Path::new("templates/weather_dashboard.svg");
/// let output_svg_name = std::path::Path::new("output/weather_dashboard.svg");
/// let clock = FixedClock::from_rfc3339("2025-10-09T22:00:00Z").unwrap();
/// generate_weather_dashboard_injection(&clock, input_template_name, output_svg_name)?;
/// ```
pub fn generate_weather_dashboard_injection(
    clock: &dyn Clock,
    input_template_name: &Path,
    output_svg_name: &Path,
) -> Result<(), Error> {
    let current_dir = std::env::current_dir()?;
    let mut context_builder = ContextBuilder::new();

    let template_svg = match fs::read_to_string(input_template_name) {
        Ok(svg) => svg,
        Err(e) => {
            logger::error(format!("Failed to read template file: {e}"));
            logger::detail(format!("Current directory: {}", current_dir.display()));
            logger::detail(format!("Template path: {}", &input_template_name.display()));
            return Err(e.into());
        }
    };

    update_forecast_context(&mut context_builder, clock)?;

    logger::subsection("Rendering dashboard to SVG");
    // Ensure the parent directory for the output SVG exists
    if let Some(parent) = output_svg_name.parent() {
        std::fs::create_dir_all(parent)?;
    }

    render_dashboard_template(&context_builder.context, template_svg, output_svg_name)?;
    logger::success(format!(
        "SVG saved: {}",
        current_dir.join(output_svg_name).display()
    ));

    if !CONFIG.debugging.disable_png_output {
        logger::subsection("Converting SVG to PNG");
        // Ensure the parent directory for the generated PNG exists
        if let Some(png_parent) = CONFIG.misc.generated_png_name.parent() {
            std::fs::create_dir_all(png_parent)?;
        }

        convert_svg_to_png(
            &output_svg_name.to_path_buf(),
            &CONFIG.misc.generated_png_name,
            2.0,
        )?;

        logger::success(format!(
            "PNG saved: {}",
            current_dir.join(&CONFIG.misc.generated_png_name).display()
        ));
    }
    Ok(())
}

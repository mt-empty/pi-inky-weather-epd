use crate::dashboard::context::{Context, ContextBuilder};
use crate::errors::DashboardError;
use crate::providers::factory::create_provider;
use crate::{utils, CONFIG};
use anyhow::Error;
use std::fs;
use std::io::Write;
use tinytemplate::{format_unescaped, TinyTemplate};
pub use utils::*;

fn update_forecast_context(context_builder: &mut ContextBuilder) -> Result<(), Error> {
    let provider = create_provider()?;
    let mut warnings: Vec<DashboardError> = Vec::new();

    println!("## Using provider: {}", provider.provider_name());

    println!("## Fetching daily forecast...");
    let daily_result = provider.fetch_daily_forecast()?;
    if let Some(warning) = daily_result.warning {
        println!("⚠️  Warning: Using stale daily forecast data");
        warnings.push(warning);
    }
    context_builder.with_daily_forecast_data(daily_result.data);

    println!("## Fetching hourly forecast...");
    let hourly_result = provider.fetch_hourly_forecast()?;
    if let Some(warning) = hourly_result.warning {
        println!("⚠️  Warning: Using stale hourly forecast data");
        warnings.push(warning);
    }
    context_builder.with_hourly_forecast_data(hourly_result.data);

    // If any warnings occurred, set the warning message
    if !warnings.is_empty() {
        // Use the first warning for display (could combine multiple in future)
        context_builder.with_warning(warnings[0].clone());
    }

    Ok(())
}

fn render_dashboard_template(context: &Context, dashboard_svg: String) -> Result<(), Error> {
    let mut tt = TinyTemplate::new();
    let tt_name = "dashboard";

    if let Err(e) = tt.add_template(tt_name, &dashboard_svg) {
        println!("Failed to add template: {}", e);
        return Err(e.into());
    }
    tt.set_default_formatter(&format_unescaped);
    // Attempt to render the template
    match tt.render(tt_name, &context) {
        Ok(rendered) => {
            let mut output = fs::File::create(&CONFIG.misc.generated_svg_name)?;
            output.write_all(rendered.as_bytes())?;
            Ok(())
        }
        Err(e) => {
            println!("Failed to render template: {}", e);
            Err(e.into())
        }
    }
}

pub fn generate_weather_dashboard() -> Result<(), Error> {
    let current_dir = std::env::current_dir()?;
    let mut context_builder = ContextBuilder::new();

    let template_svg = match fs::read_to_string(CONFIG.misc.template_path.clone()) {
        Ok(svg) => svg,
        Err(e) => {
            println!("Current directory: {}", current_dir.display());
            println!("Template path: {}", &CONFIG.misc.template_path.display());
            println!("Failed to read template file: {}", e);
            return Err(e.into());
        }
    };
    update_forecast_context(&mut context_builder)?;

    println!("## Rendering dashboard to SVG ...");
    render_dashboard_template(&context_builder.context, template_svg)?;
    println!(
        "SVG has been modified and saved successfully at {}",
        current_dir.join(&CONFIG.misc.generated_svg_name).display()
    );

    if !CONFIG.debugging.disable_png_output {
        println!("## Converting SVG to PNG ...");
        convert_svg_to_png(
            &CONFIG.misc.generated_svg_name,
            &CONFIG.misc.generated_png_name,
            2.0,
        )?;

        println!(
            "PNG has been generated successfully at {}",
            current_dir.join(&CONFIG.misc.generated_png_name).display()
        );
    }
    Ok(())
}

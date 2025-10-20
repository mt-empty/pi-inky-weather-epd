use chrono::{TimeZone, Utc};
/// Tests for the multi-error priority system
///
/// Verifies that when multiple errors/warnings occur, the highest priority
/// error is displayed (ApiError > NoInternet > IncompleteData)
use pi_inky_weather_epd::clock::FixedClock;
use pi_inky_weather_epd::dashboard::chart::ElementVisibility;
use pi_inky_weather_epd::dashboard::context::ContextBuilder;
use pi_inky_weather_epd::domain::models::DailyForecast;
use pi_inky_weather_epd::errors::DashboardError;

#[test]
fn test_single_validation_error_displays() {
    let mut builder = ContextBuilder::new();

    // Add a single validation error
    builder.with_validation_error(DashboardError::IncompleteData {
        details: "Only 5 days available".to_string(),
    });

    let context = builder.context;
    assert_eq!(context.diagnostic_message, "Incomplete Data");
    assert_eq!(
        context.diagnostic_visibility,
        ElementVisibility::Visible.to_string()
    );
    assert!(context.diagnostic_icons_svg.contains("code-yellow.svg"));
}

#[test]
fn test_single_api_warning_displays() {
    let mut builder = ContextBuilder::new();

    // Add API warning
    builder.with_warning(DashboardError::NoInternet {
        details: "Using cached data".to_string(),
    });

    let context = builder.context;
    assert_eq!(context.diagnostic_message, "API unreachable -> Stale Data");
    assert_eq!(
        context.diagnostic_visibility,
        ElementVisibility::Visible.to_string()
    );
    assert!(context.diagnostic_icons_svg.contains("code-orange.svg"));
}

#[test]
fn test_high_priority_api_error_overrides_low_priority_incomplete_data() {
    let mut builder = ContextBuilder::new();

    // Add low priority error first
    builder.with_validation_error(DashboardError::IncompleteData {
        details: "Only 5 days available".to_string(),
    });

    // Add high priority error
    builder.with_warning(DashboardError::ApiError {
        details: "Server returned error 500".to_string(),
    });

    let context = builder.context;

    // Should display the HIGH priority ApiError (red), not the LOW priority IncompleteData (yellow)
    assert_eq!(context.diagnostic_message, "API error -> Stale Data");
    assert!(
        context.diagnostic_icons_svg.contains("code-red.svg"),
        "Expected red icon for ApiError in cascading SVG"
    );
    assert_eq!(
        context.diagnostic_visibility,
        ElementVisibility::Visible.to_string()
    );
}

#[test]
fn test_medium_priority_overrides_low_priority() {
    let mut builder = ContextBuilder::new();

    // Add low priority error
    builder.with_validation_error(DashboardError::IncompleteData {
        details: "Only 5 days available".to_string(),
    });

    // Add medium priority error
    builder.with_warning(DashboardError::NoInternet {
        details: "Using cached data".to_string(),
    });

    let context = builder.context;

    // Should display the MEDIUM priority NoInternet (orange), not the LOW priority IncompleteData (yellow)
    assert_eq!(context.diagnostic_message, "API unreachable -> Stale Data");
    assert!(
        context.diagnostic_icons_svg.contains("code-orange.svg"),
        "Expected orange icon for NoInternet in cascading SVG"
    );
}

#[test]
fn test_order_doesnt_matter_highest_priority_wins() {
    let mut builder1 = ContextBuilder::new();
    let mut builder2 = ContextBuilder::new();

    // Add in one order
    builder1.with_validation_error(DashboardError::IncompleteData {
        details: "Issue 1".to_string(),
    });
    builder1.with_warning(DashboardError::ApiError {
        details: "Issue 2".to_string(),
    });

    // Add in reverse order
    builder2.with_warning(DashboardError::ApiError {
        details: "Issue 2".to_string(),
    });
    builder2.with_validation_error(DashboardError::IncompleteData {
        details: "Issue 1".to_string(),
    });

    // Both should show the same highest priority error
    assert_eq!(
        builder1.context.diagnostic_message,
        builder2.context.diagnostic_message
    );
    assert_eq!(
        builder1.context.diagnostic_icons_svg,
        builder2.context.diagnostic_icons_svg
    );
    assert!(builder1
        .context
        .diagnostic_icons_svg
        .contains("code-red.svg"));
}

#[test]
fn test_multiple_errors_same_priority_shows_first() {
    let mut builder = ContextBuilder::new();

    // Add two low priority errors
    builder.with_validation_error(DashboardError::IncompleteData {
        details: "Issue 1".to_string(),
    });
    builder.with_validation_error(DashboardError::IncompleteData {
        details: "Issue 2".to_string(),
    });

    let context = builder.context;

    // Both have same priority, so should display one of them
    assert_eq!(context.diagnostic_message, "Incomplete Data");
    assert!(context.diagnostic_icons_svg.contains("code-yellow.svg"));
}

#[test]
fn test_realistic_scenario_api_stale_and_incomplete_data() {
    // This simulates the real-world scenario where:
    // 1. API is unreachable, so we use cached data (Medium priority)
    // 2. The cached data is incomplete (Low priority)
    // Result: Should display the API unreachable warning (higher priority)

    let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 10, 15, 10, 0, 0).unwrap());
    let mut builder = ContextBuilder::new();

    // Simulate API warning (from provider)
    builder.with_warning(DashboardError::NoInternet {
        details: "Could not reach API server".to_string(),
    });

    // Simulate incomplete daily forecast data
    let incomplete_daily_data: Vec<DailyForecast> = vec![
        // Only 3 days instead of 7
    ];
    builder.with_daily_forecast_data(incomplete_daily_data, &clock);

    let context = builder.context;

    // Should show API warning (orange) instead of incomplete data warning (yellow)
    assert_eq!(context.diagnostic_message, "API unreachable -> Stale Data");
    assert!(context.diagnostic_icons_svg.contains("code-orange.svg"));
}

#[test]
fn test_no_errors_hides_warning_display() {
    let builder = ContextBuilder::new();
    let context = builder.context;

    // No errors added, should be hidden
    assert_eq!(
        context.diagnostic_visibility,
        ElementVisibility::Hidden.to_string()
    );
}

#[test]
fn test_cascading_icons_svg_generated_for_multiple_errors() {
    let mut builder = ContextBuilder::new();

    // Add multiple errors of different priorities
    builder.with_warning(DashboardError::NoInternet {
        details: "Network issue".to_string(),
    });
    builder.with_validation_error(DashboardError::IncompleteData {
        details: "Missing days".to_string(),
    });
    builder.with_warning(DashboardError::ApiError {
        details: "Server error".to_string(),
    });

    let context = builder.context;

    // Should show highest priority message (ApiError)
    assert_eq!(context.diagnostic_message, "API error -> Stale Data");

    // Should have SVG for all 3 icons
    assert!(
        context.diagnostic_icons_svg.contains("code-red.svg"),
        "Should include red icon for ApiError"
    );
    assert!(
        context.diagnostic_icons_svg.contains("code-orange.svg"),
        "Should include orange icon for NoInternet"
    );
    assert!(
        context.diagnostic_icons_svg.contains("code-yellow.svg"),
        "Should include yellow icon for IncompleteData"
    );

    // Should have 3 image tags (one for each error)
    let image_count = context.diagnostic_icons_svg.matches("<image").count();
    assert_eq!(image_count, 3, "Should have 3 image tags for 3 diagnostics");
}

#[test]
fn test_cascading_icons_are_sorted_by_priority() {
    let mut builder = ContextBuilder::new();

    // Add in reverse priority order (low to high)
    builder.with_validation_error(DashboardError::IncompleteData {
        details: "Issue 1".to_string(),
    });
    builder.with_warning(DashboardError::NoInternet {
        details: "Issue 2".to_string(),
    });
    builder.with_warning(DashboardError::ApiError {
        details: "Issue 3".to_string(),
    });

    let context = builder.context;

    // Icons should appear in REVERSE order in SVG markup (lowest priority first)
    // so that lowest priority renders in back, highest priority in front
    let svg = &context.diagnostic_icons_svg;
    let red_pos = svg.find("code-red.svg").unwrap();
    let orange_pos = svg.find("code-orange.svg").unwrap();
    let yellow_pos = svg.find("code-yellow.svg").unwrap();

    assert!(
        yellow_pos < orange_pos,
        "Yellow icon (low priority) should appear first in SVG (renders in back)"
    );
    assert!(
        orange_pos < red_pos,
        "Orange icon (medium priority) should appear before red in SVG (middle layer)"
    );
    // This ensures red (high priority) renders last and appears in front
}

#[test]
fn test_single_error_shows_one_icon() {
    let mut builder = ContextBuilder::new();

    builder.with_validation_error(DashboardError::IncompleteData {
        details: "Only issue".to_string(),
    });

    let context = builder.context;

    // Should have exactly 1 image tag
    let image_count = context.diagnostic_icons_svg.matches("<image").count();
    assert_eq!(image_count, 1, "Should have 1 image tag for 1 diagnostic");

    assert!(context.diagnostic_icons_svg.contains("code-yellow.svg"));
}

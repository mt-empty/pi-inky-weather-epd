//! Unit tests for snowfall detection logic
//!
//! These tests verify the `is_primarily_snow()` method which determines
//! whether precipitation is primarily snow based on the snowfall amount
//! and total precipitation using the 1.43 conversion factor (7cm snow = 10mm water).
//!
//! The 60% threshold determines the classification:
//! - >= 60% snow water equivalent -> Snow icon
//! - < 60% snow water equivalent -> Rain icon

use pi_inky_weather_epd::domain::models::Precipitation;

// ============================================================================
// Core Snow Detection Tests
// ============================================================================

#[test]
fn test_is_primarily_snow_with_high_snow_ratio() {
    // 10cm snow = ~7mm water, total precip = 8mm -> 87.5% snow
    let precip = Precipitation::new_with_snowfall(
        Some(80),
        Some(7),
        Some(9),
        Some(100), // 10.0cm snowfall
    );

    assert!(
        precip.is_primarily_snow(),
        "10cm snow with 8mm total should be primarily snow (87.5%)"
    );
}

#[test]
fn test_is_primarily_snow_boundary_case_60_percent() {
    // Exactly 60% snow (boundary case)
    // 8.6cm snow = ~6mm water, total = 10mm -> 60% snow
    let precip = Precipitation::new_with_snowfall(
        Some(75),
        Some(9),
        Some(11),
        Some(86), // 8.6cm snowfall
    );

    assert!(
        precip.is_primarily_snow(),
        "8.6cm snow with 10mm total should be primarily snow (exactly 60%)"
    );
}

#[test]
fn test_is_not_primarily_snow_below_threshold() {
    // 3cm snow = ~4.29mm water, total = 10mm -> 42.9% snow
    let precip = Precipitation::new_with_snowfall(
        Some(70),
        Some(9),
        Some(11),
        Some(3), // 3cm snowfall
    );

    assert!(
        !precip.is_primarily_snow(),
        "3cm snow with 10mm total should NOT be primarily snow (42.9%)"
    );
}

#[test]
fn test_is_primarily_snow_just_above_threshold() {
    // 9cm snow = ~6.45mm water, total = 10mm -> 64.5% snow
    let precip = Precipitation::new_with_snowfall(
        Some(75),
        Some(9),
        Some(11),
        Some(9), // 9cm snowfall
    );

    assert!(
        precip.is_primarily_snow(),
        "9cm snow with 10mm total should be primarily snow (64.5%)"
    );
}

#[test]
fn test_is_not_primarily_snow_just_below_threshold() {
    // 4cm snow = ~5.72mm water, total = 10mm -> 57.2% snow
    let precip = Precipitation::new_with_snowfall(
        Some(75),
        Some(9),
        Some(11),
        Some(4), // 4cm snowfall
    );

    assert!(
        !precip.is_primarily_snow(),
        "4cm snow with 10mm total should NOT be primarily snow (57.2%)"
    );
}

// ============================================================================
// Edge Cases and Special Scenarios
// ============================================================================

#[test]
fn test_no_snowfall_returns_false() {
    let precip = Precipitation::new(Some(80), Some(10), Some(20));
    assert!(
        !precip.is_primarily_snow(),
        "Precipitation without snowfall field should not be snow"
    );
}

#[test]
fn test_zero_snowfall_returns_false() {
    let precip = Precipitation::new_with_snowfall(Some(60), Some(5), Some(10), Some(0));
    assert!(
        !precip.is_primarily_snow(),
        "Zero snowfall should not be classified as snow"
    );
}

#[test]
fn test_all_snow_no_rain() {
    // 14.3cm snow = ~10mm water, total = 10mm -> 100% snow
    let precip = Precipitation::new_with_snowfall(
        Some(90),
        Some(9),
        Some(11),
        Some(143), // 14.3cm snowfall
    );

    assert!(
        precip.is_primarily_snow(),
        "Pure snow (100%) should be classified as primarily snow"
    );
}

#[test]
fn test_light_snow_with_heavy_rain() {
    // 1cm snow = ~1.43mm water, total = 20mm -> 7.15% snow
    let precip = Precipitation::new_with_snowfall(
        Some(85),
        Some(18),
        Some(22),
        Some(1), // 1cm snowfall
    );

    assert!(
        !precip.is_primarily_snow(),
        "Light snow with heavy rain (7.15% snow) should be classified as rain"
    );
}

// ============================================================================
// Realistic Weather Scenarios
// ============================================================================

#[test]
fn test_winter_storm_scenario() {
    // Heavy snowstorm: 20cm snow = ~14mm water, total = 15mm -> 93% snow
    let precip = Precipitation::new_with_snowfall(
        Some(95),
        Some(14),
        Some(16),
        Some(200), // 20cm snowfall
    );

    assert!(
        precip.is_primarily_snow(),
        "Heavy snowstorm scenario should be classified as snow"
    );
}

#[test]
fn test_mixed_precipitation_scenario() {
    // Mixed: 3cm snow = ~4.29mm water, total = 12mm -> 35.75% snow
    let precip = Precipitation::new_with_snowfall(
        Some(80),
        Some(11),
        Some(13),
        Some(3), // 3cm snowfall
    );

    assert!(
        !precip.is_primarily_snow(),
        "Mixed precipitation (35.75% snow) should be classified as rain"
    );
}

#[test]
fn test_light_flurries_scenario() {
    // Light flurries: 1cm snow = ~1.43mm water, total = 1mm -> 143% snow (all snow)
    let precip = Precipitation::new_with_snowfall(
        Some(40),
        Some(0),
        Some(1),
        Some(1), // 1cm snowfall
    );

    assert!(
        precip.is_primarily_snow(),
        "Light snow flurries (100% snow) should be classified as snow"
    );
}

// ============================================================================
// Helper Method Tests
// ============================================================================

#[test]
fn test_has_snow_returns_true_with_snowfall() {
    let precip = Precipitation::new_with_snowfall(Some(50), Some(5), Some(10), Some(10));

    assert!(
        precip.has_snow(),
        "has_snow() should return true when snowfall > 0"
    );
}

#[test]
fn test_has_snow_returns_false_without_snowfall() {
    let precip = Precipitation::new(Some(50), Some(5), Some(10));

    assert!(
        !precip.has_snow(),
        "has_snow() should return false when snowfall is None"
    );
}

#[test]
fn test_has_snow_returns_false_with_zero_snowfall() {
    let precip = Precipitation::new_with_snowfall(Some(50), Some(5), Some(10), Some(0));

    assert!(
        !precip.has_snow(),
        "has_snow() should return false when snowfall is 0"
    );
}

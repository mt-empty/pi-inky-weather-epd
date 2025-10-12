# Pi Inky Weather Display - AI Coding Agent Instructions

## Project Overview

A Rust application that generates weather dashboards for Raspberry Pi with 7.3" e-paper displays. Supports multiple weather APIs (BOM, Open-Meteo) through a provider pattern, renders SVG templates with TinyTemplate, and converts to PNG using resvg for display on Inky Impression 7.3" hardware.

**Core Flow**: Provider Factory → Fetch API data → Convert to domain models → Build context → Render SVG template → Convert to PNG → Display via Python script

**Entry Point**: `src/main.rs` calls `run_weather_dashboard()` in `src/lib.rs`, which orchestrates dashboard generation and optional self-updates.

## Architecture & Key Components

### Provider Pattern (NEW in v0.5.5)

The application uses a **domain-driven architecture** with pluggable weather providers:

```
Provider Factory → WeatherProvider Trait → Domain Models → Template Context
```

- **Domain Layer** (`src/domain/`): API-agnostic models (`HourlyForecastAbs`, `DailyForecastAbs`, `Temperature`, `Wind`, `Precipitation`)
- **Provider Layer** (`src/providers/`): 
  - `WeatherProvider` trait - common interface for all APIs
  - `Fetcher` - shared HTTP client with caching/fallback
  - `BomProvider`, `OpenMeteoProvider` - concrete implementations
  - `factory.rs` - creates provider based on `CONFIG.api.provider`
- **API Layer** (`src/apis/`): Provider-specific response models with `From` traits for domain conversion

### Module Structure
- **`src/weather_dashboard.rs`**: Main orchestrator (25 lines) - uses provider factory, builds context, renders
- **`src/providers/`**: Provider pattern implementation
  - `mod.rs` - `WeatherProvider` trait definition
  - `factory.rs` - `create_provider()` based on config
  - `fetcher.rs` - Shared `Fetcher` with `FetchOutcome<T>` enum
  - `bom.rs` - BOM provider (Australia only, geohash-based)
  - `open_meteo.rs` - Open-Meteo provider (worldwide, lat/lon)
- **`src/domain/`**: Domain models and icon logic
  - `models.rs` - Core domain types with conversion traits
  - `icons.rs` - `Icon` trait implementations for domain models
- **`src/dashboard/context.rs`**: `ContextBuilder` - transforms domain models to template variables
- **`src/dashboard/chart.rs`**: SVG path generation for temperature/rain graphs
- **`src/apis/`**: API-specific models
  - `bom/models.rs` + `bom/utils.rs` - BOM API structures
  - `open_metro/models.rs` - Open-Meteo API structures with direct domain conversions
- **`src/configs/settings.rs`**: Layered config system with `Providers` enum
- **`src/weather/icons.rs`**: Icon enum definitions and `Icon` trait
- **`src/update.rs`**: Self-update logic from GitHub releases

### Data Flow Pattern
```
Config (provider = "bom" | "open_meteo")
    ↓
Provider Factory
    ↓
WeatherProvider::fetch_hourly_forecast() / fetch_daily_forecast()
    ↓
Fetcher::fetch_data() → HTTP GET → API
    ↓ (success)
FetchOutcome::Fresh(data)
    ↓ (failure, cached exists)
FetchOutcome::Stale { data, error }
    ↓
From trait → Domain Models (HourlyForecastAbs, DailyForecastAbs)
    ↓
ContextBuilder → Template Context
    ↓
TinyTemplate → SVG
    ↓
resvg → PNG
```

**Stale Data Handling**: `FetchOutcome<T>` enum distinguishes fresh vs cached data. Falls back to cached JSON on network failure with warning context.

### Configuration System
Hierarchical merge via `config` crate:
1. `config/default.toml` (base)
2. `~/.config/pi-inky-weather-epd.toml` (user)
3. `config/{RUN_MODE}.toml` (environment - default: "development")
4. `config/local.toml` (dev override, gitignored)
5. Environment variables with `APP_` prefix (e.g., `APP_API__PROVIDER=bom`)

**Global access**: `CONFIG` lazy static in `src/lib.rs` - initialized on first access, panics if invalid

**Provider selection**: `CONFIG.api.provider` (enum: `Bom` or `OpenMeteo`)

**Environment variable format**: Use double underscores for nested keys: `APP_API__PROVIDER`, `APP_DEBUGGING__DISABLE_PNG_OUTPUT`

### Type Safety Patterns

**Nutype wrapper types** (`src/configs/settings.rs`): Validated newtypes for domain values
```rust
#[nutype(sanitize(trim, lowercase), validate(len_char_min = 6, len_char_max = 6))]
pub struct GeoHash(String);
```

**From traits for conversion**: API models → domain models
```rust
impl From<apis::bom::models::HourlyForecast> for domain::models::HourlyForecastAbs {
    fn from(bom: apis::bom::models::HourlyForecast) -> Self { /* ... */ }
}
```

**Custom deserializers** (`src/apis/bom/utils.rs`): Temperature conversion during deserialization:
```rust
pub fn de_temp_celsius<'de, D>(deserializer: D) -> Result<Temperature, D::Error>
```

## Critical Implementation Details

### SVG Templating with TinyTemplate
- Templates use `{variable}` syntax for substitution
- **Important**: `format_unescaped` formatter is set globally - all variables render as raw SVG/HTML
- Template file: `dashboard-template-min.svg`
- Context fields match variable names exactly (e.g., `{current_hour_actual_temp}`)

### resvg Quirks & Workarounds
Multiple SVG files contain this comment indicating known issues:
```xml
<!-- Avoid using tspan with text-anchor, `dx` or `dy`, resvg doesn't handle it properly -->
<!-- see https://github.com/linebender/resvg/issues/583 -->
```
**Workaround**: Manually offset positions in template to compensate for rendering bugs

**Font loading**: Custom fonts in `static/fonts/` must be loaded into `fontdb` before rendering (`src/utils.rs::load_fonts`)

### Cross-Compilation Setup
Uses `cross` for ARM targets (Pi Zero, Pi 4). Targets defined in `.github/workflows/release.yml`:
- `arm-unknown-linux-gnueabihf` (Pi Zero)
- `aarch64-unknown-linux-gnu` (Pi 4+)
- `x86_64-unknown-linux-gnu` (x86 dev)

**Build command**: `cross build --release --target=<TARGET>`

OpenSSL is vendored (`Cargo.toml` feature) to avoid cross-compilation issues.

## Development Workflows

### Local Development
```bash
# Create local config (gitignored)
cp config/development.toml config/local.toml
# Edit config/local.toml with your geohash/coordinates
cargo run
```

**Debug flags** in config:
- `disable_weather_api_requests = true`: Use cached JSON (requires one successful fetch first)
- `disable_png_output = true`: Skip PNG generation for faster iteration

### Testing

**Run all tests**:
```bash
RUN_MODE=test cargo test
```

**Run specific test suites**:
```bash
# Snapshot tests with Open-Meteo provider (default)
RUN_MODE=test cargo test --test snapshot_provider_test

# BOM provider snapshot test (requires override)
RUN_MODE=test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test snapshot_bom_dashboard -- --ignored

# DST/timezone tests (12 comprehensive tests)
cargo test --test daylight_saving_test

# Clock abstraction tests (fixed time injection)
cargo test --test clock_integration_test
```

**Review snapshot changes**:
```bash
RUN_MODE=test cargo test --test snapshot_provider_test
cargo insta review  # Interactive review of SVG snapshots
```

**Environment variables for testing**:
- `RUN_MODE=test`: Loads `config/test.toml` (disables API calls, uses fixtures)
- `APP_API__PROVIDER=bom|open_meteo`: Override provider (requires `__` separator for nested keys)

**Test structure**:
- `tests/fixtures/`: Pre-captured API responses for deterministic tests
- `tests/snapshots/`: Insta snapshot files for SVG output verification
- `cached_bom_data/`: Development cache directory (not used in tests)

**No live API integration tests** - all tests use fixtures or cached data to ensure reproducibility.

## API Integration Patterns

### BOM API (Australian Bureau of Meteorology)
- Geohash-based location (6 chars via `src/utils.rs::encode`)
- Two endpoints: daily forecast, hourly forecast
- Error model: `BomError` with array of error details
- Custom deserializers handle temperature unit conversion

### Open-Meteo API (Global)
- Lat/lon based, no authentication required
- Single endpoint returns all forecast data
- Direct conversion to domain models via `From` trait implementations in `src/apis/open_metro/models.rs`

## Provider System Architecture

### Adding a New Weather Provider

1. **Create API models** in `src/apis/your_api/models.rs`
2. **Implement `From` traits** to convert API response → domain models
3. **Create provider** in `src/providers/your_provider.rs`:
   - Struct with `Fetcher` field
   - Implement `WeatherProvider` trait
   - Use `Fetcher::fetch_data()` for HTTP + caching
4. **Register in factory** (`src/providers/factory.rs`)
5. **Add enum variant** to `Providers` in `src/configs/settings.rs`

Example provider implementation:
```rust
pub struct YourProvider {
    fetcher: Fetcher,
}

impl WeatherProvider for YourProvider {
    fn fetch_hourly_forecast(&self) -> Result<Vec<HourlyForecastAbs>, Error> {
        match self.fetcher.fetch_data::<YourApiResponse, ()>(
            endpoint, "hourly_forecast.json", None
        )? {
            FetchOutcome::Fresh(data) => Ok(data.into()),
            FetchOutcome::Stale { data, error } => {
                eprintln!("Warning: {:?}", error);
                Ok(data.into())
            }
        }
    }
}
```

### Fetcher Usage Pattern

The shared `Fetcher` handles:
- HTTP requests with `reqwest::blocking::Client`
- Automatic cache directory creation
- Respects `disable_weather_api_requests` debug flag
- Falls back to cached JSON on network errors
- Optional API-specific error checking via callback

**BOM-specific error checking**:
```rust
fn check_bom_error(body: &str) -> Result<(), DashboardError> {
    if let Ok(api_error) = serde_json::from_str::<BomError>(body) {
        return Err(DashboardError::ApiError(/* ... */));
    }
    Ok(())
}

// Usage in provider:
self.fetcher.fetch_data::<Response, BomError>(
    endpoint, "file.json", Some(check_bom_error)
)
```

**OpenMeteo (no custom errors)**:
```rust
self.fetcher.fetch_data::<Response, ()>(
    endpoint, "file.json", None  // No error checker needed
)
```

**Current branch (`open_meteo`)**: Experimenting with Open-Meteo as BOM alternative. Check `src/weather_dashboard.rs` for commented-out switches.

## Common Patterns to Follow

### Clock/Time Abstraction (Critical for Testing)
**Problem**: Time-dependent logic must be testable with fixed times for deterministic snapshots.

**Solution**: `Clock` trait in `src/clock.rs` enables dependency injection:
- `SystemClock`: Production - returns actual current time
- `FixedClock`: Testing - returns predetermined DateTime
- `MockClock`: Advanced testing - custom closure-based time

**Usage pattern**:
```rust
// Production (main.rs, lib.rs)
generate_weather_dashboard() // uses SystemClock internally

// Testing (tests/*)
let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 10, 10, 1, 0, 0).unwrap());
generate_weather_dashboard_with_clock(&clock)
```

**Thread through all time-dependent functions**: Chart rendering, context building, hour filtering all accept `&dyn Clock`. Never use `chrono::Local::now()` directly - always use `clock.now_local()`.

**DST Testing**: 12 comprehensive tests in `tests/daylight_saving_test.rs` verify UTC→Local conversions during Australian DST transitions (AEST ↔ AEDT).

### Error Handling
- `DashboardError` enum for user-facing errors with icons (`src/errors.rs`)
- Descriptive errors via `anyhow::Error` with context
- Display error indicators in dashboard corner with icon + message

### Icon System
- All icons are enums implementing `Display` trait (e.g., `UVIndexIcon`, `WindIconName`)
- Icon paths constructed via `to_string()` → matches filename in `static/fill-svg-static/`
- Moon phase logic: replaces clear-night icon when `use_moon_phase_instead_of_clear_night = true`
- Domain models implement `Icon` trait in `src/domain/icons.rs` for polymorphic icon resolution

### Graph Rendering
Manual SVG path construction in `src/dashboard/chart.rs`:
- Convert data points to coordinates
- Generate Bézier curves for smooth lines (temperature)
- Linear segments for stepped data (rain chance)
- Axis positioning respects `x_axis_always_at_min` config

## Dependencies & Constraints

**Hardware-specific**: Designed for 7.3" Inky Impression 7-color e-paper display
- Supported colors: black, white, green, blue, red, yellow, orange
- Output resolution scaled 2x in PNG generation for clarity
- Final display handled by Pimoroni's Python library (external to this project)

**Color validation**: `Colour` nutype validates hex/named colors in config (`src/configs/validation.rs`)

## Self-Update Mechanism
When `update_interval_days > 0`:
1. Checks GitHub releases API for newer semver tag
2. Downloads architecture-specific ZIP from releases
3. Extracts and replaces current binary
4. Updates tracked in `last_checked` file

**Important**: Binary name must match `CARGO_PKG_NAME` for replacement logic

## Notes for AI Agents

- **Entry point**: `src/main.rs` → `run_weather_dashboard()` → `generate_weather_dashboard()` in `src/weather_dashboard.rs`
- **Temperature handling**: Always check `CONFIG.render_options.temp_unit` - conversion happens at deserialization
- **Time zones**: API returns UTC, convert to local using `src/utils.rs::utc_timestamp_to_local_*` functions
- **Clock injection**: For any time-dependent code, accept `&dyn Clock` parameter and use `clock.now_local()/now_utc()` instead of `chrono::Local::now()`
- **Config changes**: Modify TOML files, not hardcoded values - system merges configs hierarchically
- **SVG debugging**: Set `disable_png_output = true` and inspect `dashboard.svg` directly
- **Adding weather icons**: Create SVG in `static/fill-svg-static/`, add enum variant in `src/weather/icons.rs`, implement `Icon` trait in `src/domain/icons.rs` if needed
- **Geohash**: Use https://geohash.softeng.co for location codes (Australia + BOM API only)
- **Test environment**: Always set `RUN_MODE=test` when running tests to load `config/test.toml` with fixtures
- **Snapshot testing**: Use `insta` crate - review changes with `cargo insta review` after modifying dashboard generation
- **Provider testing**: Default tests use Open-Meteo; override with `APP_API__PROVIDER=bom` for BOM-specific tests

## Current Development Focus

Branch `open_meteo` is exploring global weather support via Open-Meteo API to supplement/replace BOM (Australia-only) API. Key changes are in `src/weather_dashboard.rs` with mapping logic in `src/apis/open_metro/models.rs`.

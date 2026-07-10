# Pi Inky Weather Display - Agent Instructions

This file is intentionally concise. Keep project rules here minimal and actionable, and link to the detailed docs already in this repository.

## Quick Commands

- Build: `cargo build --bin=pi-inky-weather-epd`
- Run: `cargo run --bin=pi-inky-weather-epd`
- Format check: `cargo fmt -- --check`
- Lint: `cargo clippy -- -D warnings`
- Tests: `cargo test` (covers all providers and render options in one run)
- Snapshot review after rendering changes: `cargo insta review`

## Critical Runtime and Test Context

- Tests need no env vars and run fully in parallel: configuration is a plain value — build one with `tests/helpers/test_utils.rs` (`test_settings`, `open_meteo_settings`, `bom_settings`, `open_meteo_settings_in_tz`) and pass it into the code under test. There is no global config and no `#[serial]`.
- For the application (not tests), use nested config env vars with double underscores, for example `APP_API__PROVIDER=bom`, and `RUN_MODE` to select the config file set.
- The provider values are lowercase strings: `bom` and `open_meteo`.
- CLI simulation is feature-gated. Use `--features cli` when testing simulated time.

## Architecture in 30 Seconds

- Entrypoint flow: `src/main.rs` -> `run_weather_dashboard()` in `src/lib.rs` -> orchestration in `src/weather_dashboard.rs`.
- Provider flow: provider factory in `src/providers/factory.rs` returns `Box<dyn WeatherProvider>`.
- Data pipeline: provider/fetcher -> domain models -> dashboard context -> TinyTemplate SVG -> resvg PNG.

## Project-Specific Coding Rules

- Time-dependent logic must use the `Clock` abstraction (`src/clock.rs`) for testability.
- Do not call local time directly in business logic; thread a `&dyn Clock` through time-sensitive paths.
- Distinguish fetcher and provider result types:
  - Fetcher returns `FetchOutcome<T>` (`Fresh` or `Stale`).
  - Provider returns `FetchResult<T>` (data plus optional warning).
- Preserve stale-data diagnostics by propagating warnings to context builders.

## Backward Compatibility

The binary updates in place on unattended Pi devices, so a newer binary routinely reads state written by an older one.

- Cached API responses: `fetcher.rs::load_cached` falls back to the last raw JSON written to disk on a failed fetch — it can predate any schema change.
- User config files (`local.toml`/`development.toml`) persist across upgrades and are not regenerated.
- Additive fields on serde models need `#[serde(default)]` (or `Option`) so old cached JSON without the field still deserializes instead of hard-failing the fetch. A rename/removal needs a real migration, not just an updated fixture.
- Reads of new/changed fields must tolerate absence — prefer `.get(i)`/`.unwrap_or_default()` over direct indexing on `Vec` fields.
- Add a test that deserializes JSON (or loads config) with the field missing and asserts the fallback, not just the happy path.

## Known Pitfalls

- `APP_API_PROVIDER` is wrong; use `APP_API__PROVIDER`.
- `APP_*` env vars do not affect tests; tests use `config/test.toml` plus per-test overrides.
- Snapshot tests can fail after intentional SVG changes until snapshots are reviewed/accepted.
- `resvg` has text quirks with some `tspan` combinations; follow existing SVG comments/workarounds.

## Read Next (Link, Do Not Duplicate)

- Project guide and setup: [readme.md](readme.md)
- Rust conventions: [rust.instructions.md](.github/instructions/rust.instructions.md)
- GitHub Actions conventions: [github-actions-ci-cd-best-practices.instructions.md](.github/instructions/github-actions-ci-cd-best-practices.instructions.md)
- Test workflow guardrails: [test-workflow-guardrails.instructions.md](.github/instructions/test-workflow-guardrails.instructions.md)
- Config loading and validation: [settings.rs](src/configs/settings.rs), [validation.rs](src/configs/validation.rs)
- Clock abstraction: [clock.rs](src/clock.rs)
- Provider interfaces and fetcher behavior: [providers/mod.rs](src/providers/mod.rs), [providers/fetcher.rs](src/providers/fetcher.rs)
- Error priorities and diagnostics: [errors.rs](src/errors.rs)
- Snapshot maintenance workflow (Copilot custom agent): [.github/agents/snapshot-maintainer.agent.md](.github/agents/snapshot-maintainer.agent.md)

## When Editing This File

- Keep it short and high-signal.
- Prefer links to in-repo docs over embedded long-form explanations.
- Focus on non-obvious project conventions that improve agent success rate.

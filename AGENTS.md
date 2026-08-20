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

## Internationalization (i18n)

This project renders in 5 languages — `en`, `fr`, `de`, `es`, `ja` (`render_options.language` in config; validated in `configs/validation.rs`) — including one non-Latin script (Japanese, via the bundled `NotoSansJP-Weather-Regular.ttf` font). Any change touching rendered text, dashboard templates, or fonts needs to account for all 5, not just English:

- All user-visible strings are translated via `translate()`/`TranslationKey` in `src/i18n.rs` — never hardcode English text in templates or `context.rs`.
- Word length/width varies a lot by language and script (e.g. French "comme" vs German "wie" vs Japanese "温度"), so layout that assumes English-sized text will misalign or clip for other locales. Prefer computing positions from measured render output (see `utils::measure_stacked_label_dx`/`measure_label_to_number_gap_dx`) over hardcoded per-language pixel offsets — those require manual recalibration every time wording or fonts change and silently rot.
- After any template/font/layout change, regenerate and eyeball all locales: `bash scripts/generate-showcase.sh` writes `misc/languages/dashboard-{en,fr,de,es,ja}.png`.
- `tests/i18n_integration_test.rs` covers locale-specific rendering; snapshot tests also exist per locale (e.g. `snapshot_test__localization__french_dashboard`) — review with `cargo insta review` after intentional changes.
- If you extend the render-and-measure functions in `utils.rs`: `usvg`'s `Text::bounding_box().width()` is glyph ink extent, not cursor advance width — they differ by several pixels (side bearings), so don't use it to predict where the SVG cursor lands after a tspan; rasterize and measure actual pixel positions instead. Also, any measurement function must render through the same `shared_font_db()` used by `convert_svg_to_png` — separate font databases can resolve a bundled font differently than a system-installed one with the same name, silently making measured layout diverge from what actually renders.

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

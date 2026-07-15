# Snapshot Test Consolidation Plan

Companion to `docs/test-suite-review.md` (Finding 4: compile-time cost from fragmentation)
and `docs/test-suite-migration-plan.md` Step 4.2, which explicitly deferred this until the
migration was done and the real shape of `tests/` was visible. It's visible now: 3 files,
779 lines, 16 tests, all legitimate end-to-end snapshot tests (they drive the real
`generate_weather_dashboard_injection` pipeline against `insta` snapshots — nothing here is
misplaced, this is pure compile-time housekeeping).

## Why this saves build time

Every file directly under `tests/` compiles as its own separate crate/binary linked against
the library. All three files here declare `mod helpers;`, so `tests/helpers/{mod,
test_utils, wiremock_setup}.rs` currently gets compiled from scratch 3 times, and `cargo
test`/`cargo insta test` links 3 separate binaries instead of 1.

**Important nuance**: the build-time win comes from merging into *one binary* — that's what
cuts the redundant `helpers` compilation and the extra link steps. Nesting the merged file's
tests into `mod` submodules (`mod provider`, `mod precipitation`, `mod prefer_codes`) is
purely organizational — it doesn't cost anything extra, but it doesn't independently save
build time either. It's still worth doing here, both because the three files have genuinely
distinct concerns (their own doc comments say so) and because it matches the nested-`mod`
convention this whole migration already established in `src/domain/models.rs` and
`src/domain/icons.rs`. But don't expect the submodule structure itself to be where the speed
comes from — that's the single-binary merge.

## The one real risk: insta snapshot naming

`insta` derives each snapshot's filename from `<module_path>` + the test function name, with
`::` replaced by `__`. Verified directly against the current repo:

```
$ ls tests/snapshots/
snapshot_provider_test__snapshot_bom_dashboard.snap
snapshot_provider_test__snapshot_open_meteo_dashboard.snap
...
```

Today there's no `mod` nesting inside these files, so `module_path!()` at a top-level test
function is just the crate name — which, per Cargo's default `tests/*.rs` convention, equals
the file's stem. That's why every snapshot today is named `<file_stem>__<fn_name>.snap`.

Once these are merged into one file with nested `mod` blocks, the module path changes to
`<new_file_stem>::<mod_name>`, so **every one of the 16 `.snap` files must be renamed** to
match, or `cargo insta test` will report all 16 as "new" snapshots (with the old 16 orphaned
as unreferenced) instead of recognizing them as unchanged. Renaming them wrong is silently
recoverable but noisy: insta will just show you a spurious diff to review. The mapping below
is computed exactly, so this should be a non-event if followed precisely — but **step 4's
verification is what actually proves it's right**, not the derivation alone.

---

## Step 1: Create `tests/snapshot_test.rs`

Merge all three source files into one, with nested `mod` blocks matching their original
concerns. Each original file's top-level `//!` (inner) doc comment becomes a `///` (outer)
doc comment on its `mod` declaration instead, since a single file can only have one
module-level `//!` block at its true top (before any items) — same pattern already used for
every nested test module added earlier in this migration
(`src/domain/models.rs`'s `mod snow_detection`, `mod daylight_saving`, etc.).

Structure:

```rust
//! Snapshot tests driving the full dashboard-generation pipeline against `insta`
//! snapshots. See docs/snapshot-test-consolidation-plan.md for why this file merges
//! what were three separate top-level test files.

mod helpers;

use helpers::test_utils;
use helpers::wiremock_setup;
use pi_inky_weather_epd::{clock::FixedClock, generate_weather_dashboard_injection};
use std::fs;
use std::path::Path;

/// Provider-specific snapshot tests
///
/// [... paste snapshot_provider_test.rs's original //! block here as /// lines ...]
mod provider {
    use super::*;

    // ... all 10 #[tokio::test] fns from tests/snapshot_provider_test.rs, unchanged ...
}

/// Snapshot tests for precipitation glyph rendering
///
/// [... paste snapshot_precipitation_test.rs's original //! block here as /// lines ...]
mod precipitation {
    use super::*;

    // ... both #[tokio::test] fns from tests/snapshot_precipitation_test.rs, unchanged ...
}

/// Snapshot tests for Open-Meteo with `prefer_weather_codes = true`
///
/// [... paste snapshot_open_meteo_prefer_codes_test.rs's original //! block here as /// lines ...]
mod prefer_codes {
    use super::*;
    use pi_inky_weather_epd::configs::settings::Providers;

    // ... the run_prefer_codes_snapshot() helper and all 4 #[tokio::test] fns, unchanged ...
}
```

Notes:
- `use super::*;` inside each `mod` picks up the file-level imports — same pattern as the
  rest of this migration.
- `prefer_codes` needs its own extra `use pi_inky_weather_epd::configs::settings::Providers;`
  (the only import not shared by all three original files).
- Port every test function **body as-is** — this step is a pure move, not a rewrite. (The
  three files are quite repetitive internally — nearly every `provider`/`precipitation` test
  repeats the same wiremock-setup/clock/render/read-file boilerplate that `prefer_codes`
  already factored into `run_prefer_codes_snapshot()`. Deduplicating that is a legitimate
  follow-up, but do it as a *separate* change after this consolidation lands, so a snapshot
  rename and a behavior-preserving refactor aren't both landing in the same diff.)
- The pasted `//!`→`///` doc blocks contain "Running These Tests" examples like
  `cargo test --test snapshot_provider_test`. Don't paste these verbatim — that binary no
  longer exists after the merge. Update each to `cargo test --test snapshot_test` (optionally
  with a `provider::` / `precipitation::` / `prefer_codes::` name filter), since a `mod`-level
  doc comment referencing a now-nonexistent test binary would be actively misleading even
  though it's never executed as a doctest (integration-test binaries aren't doc-tested).
- Delete the three old files (`git rm`) only after Step 1's new file compiles.

**Verify**: `cargo build --test snapshot_test` compiles clean before moving on.

---

## Step 2: Rename the 16 snapshot files

Exact mapping (old → new), computed from the module structure in Step 1. Use `git mv` (not
`mv`) so history follows the rename:

| Old name | New name |
|---|---|
| `snapshot_provider_test__snapshot_open_meteo_dashboard.snap` | `snapshot_test__provider__snapshot_open_meteo_dashboard.snap` |
| `snapshot_provider_test__snapshot_open_meteo_midnight_boundary.snap` | `snapshot_test__provider__snapshot_open_meteo_midnight_boundary.snap` |
| `snapshot_provider_test__snapshot_open_meteo_end_of_day.snap` | `snapshot_test__provider__snapshot_open_meteo_end_of_day.snap` |
| `snapshot_provider_test__snapshot_open_meteo_early_morning.snap` | `snapshot_test__provider__snapshot_open_meteo_early_morning.snap` |
| `snapshot_provider_test__snapshot_bom_dashboard.snap` | `snapshot_test__provider__snapshot_bom_dashboard.snap` |
| `snapshot_provider_test__snapshot_bom_midnight_boundary.snap` | `snapshot_test__provider__snapshot_bom_midnight_boundary.snap` |
| `snapshot_provider_test__snapshot_bom_local_midnight.snap` | `snapshot_test__provider__snapshot_bom_local_midnight.snap` |
| `snapshot_provider_test__snapshot_bom_early_morning.snap` | `snapshot_test__provider__snapshot_bom_early_morning.snap` |
| `snapshot_provider_test__snapshot_open_meteo_ny_6pm_before_gmt_boundary.snap` | `snapshot_test__provider__snapshot_open_meteo_ny_6pm_before_gmt_boundary.snap` |
| `snapshot_provider_test__snapshot_open_meteo_ny_7pm_after_gmt_boundary.snap` | `snapshot_test__provider__snapshot_open_meteo_ny_7pm_after_gmt_boundary.snap` |
| `snapshot_precipitation_test__snapshot_open_meteo_alaska_snow.snap` | `snapshot_test__precipitation__snapshot_open_meteo_alaska_snow.snap` |
| `snapshot_precipitation_test__snapshot_open_meteo_mixed_precip.snap` | `snapshot_test__precipitation__snapshot_open_meteo_mixed_precip.snap` |
| `snapshot_open_meteo_prefer_codes_test__snapshot_open_meteo_dashboard_prefer_codes.snap` | `snapshot_test__prefer_codes__snapshot_open_meteo_dashboard_prefer_codes.snap` |
| `snapshot_open_meteo_prefer_codes_test__snapshot_open_meteo_midnight_boundary_prefer_codes.snap` | `snapshot_test__prefer_codes__snapshot_open_meteo_midnight_boundary_prefer_codes.snap` |
| `snapshot_open_meteo_prefer_codes_test__snapshot_open_meteo_end_of_day_prefer_codes.snap` | `snapshot_test__prefer_codes__snapshot_open_meteo_end_of_day_prefer_codes.snap` |
| `snapshot_open_meteo_prefer_codes_test__snapshot_open_meteo_early_morning_prefer_codes.snap` | `snapshot_test__prefer_codes__snapshot_open_meteo_early_morning_prefer_codes.snap` |

All 16 old files map 1:1 to new names — nothing is added or dropped.

---

## Step 3: Delete the three old test files

```
git rm tests/snapshot_provider_test.rs tests/snapshot_precipitation_test.rs \
       tests/snapshot_open_meteo_prefer_codes_test.rs
```

---

## Step 4: Verify — this is what actually proves the rename mapping was right

```
cargo insta test
```

Expected: all 16 tests pass with **zero** pending/new/unreferenced snapshots reported. If
insta reports a snapshot as "new," the corresponding old file wasn't renamed to match (or was
renamed to the wrong name) — fix that specific mapping and re-run rather than accepting a
"new" snapshot, since blindly accepting could paper over an actual rendering change smuggled
in alongside the pure rename.

Then run the full suite to confirm nothing else was affected:

```
cargo test
```

Expect the same total pass count as before this change (this is a pure rename/relocation,
not a test-count change — unlike the earlier migration phases, nothing here should add or
remove any test).

---

## Step 0 (before Step 1): Optional — capture the "before" build-time baseline

Not required, but if you want a before/after number for the commit message, capture the
**before** measurement now, prior to Step 1 — by the time Step 3 deletes the three old files,
`--test snapshot_provider_test` etc. no longer exist as targets and the command below will
error out. Either take this measurement first, or recover the pre-merge tree afterward (e.g.
`git stash` / checking out the prior commit) before running it.

```
touch tests/helpers/test_utils.rs && time cargo test --no-run \
  --test snapshot_provider_test --test snapshot_precipitation_test \
  --test snapshot_open_meteo_prefer_codes_test
```

(`--no-run` isolates compile+link time from actual test execution, which is what this
consolidation affects — the number of tests running doesn't change.)

## Step 5: Optional — measure the "after" build time and compare

```
touch tests/helpers/test_utils.rs && time cargo test --no-run --test snapshot_test
```

Compare against the Step 0 baseline to quantify the win from linking one binary instead of
three.

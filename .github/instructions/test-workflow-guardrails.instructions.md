---
description: "Use when writing, debugging, or fixing Rust tests, snapshot tests, fixtures, or cargo test commands in this repository. Enforces RUN_MODE=test, APP_API__PROVIDER naming, deterministic test patterns, and cargo insta review workflow."
applyTo: "tests/**/*.rs"
---

# Test Workflow Guardrails

- Run tests with RUN_MODE=test so config/test.toml is loaded.
- For provider overrides, use APP_API__PROVIDER with a double underscore.
- Provider values are lowercase: bom and open_meteo.
- Prefer existing workspace tasks for common test runs.

## Snapshot Workflow

- If rendering output changes, run the relevant snapshot test first, then run cargo insta review.
- Treat snapshot diffs as behavior changes. Accept only intentional visual/output updates.
- Mention snapshot file changes in PR notes when applicable.

## Reliability Rules

- Keep tests deterministic: avoid real API calls and rely on test fixtures/config.
- For time-sensitive logic, use the Clock abstraction and fixed/mock clocks in tests.
- Do not introduce environment variable names that conflict with the nested APP_*__* pattern.

## Read Next

- Main project instructions: [copilot-instructions.md](../copilot-instructions.md)
- Rust conventions: [rust.instructions.md](rust.instructions.md)
- Test setup source of truth: [config/test.toml](../../config/test.toml)

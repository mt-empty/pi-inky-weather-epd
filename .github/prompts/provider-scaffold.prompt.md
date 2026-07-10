---
name: "Provider Scaffold"
description: "Use when adding or planning a new weather provider integration. Generates an implementation checklist and file scaffold across API models, provider/factory wiring, config, tests, and docs."
argument-hint: "Provider name, API source, and any special constraints"
agent: "agent"
tools: [read, search, edit]
---

Create a repository-specific scaffold plan for adding a new weather provider.

Inputs to infer from the argument:
- Provider name and config enum value
- Endpoint style (single endpoint or hourly/daily endpoints)
- Any special fields or unsupported fields

Generate output in this order:

1. Scope Summary
- One paragraph describing the provider and expected constraints.

2. File Checklist
- List files to create or update with one-line purpose each.
- Include at least:
  - src/apis/<provider>/mod.rs
  - src/apis/<provider>/models.rs
  - src/providers/<provider>.rs
  - src/providers/factory.rs
  - src/providers/mod.rs
  - src/configs/settings.rs
  - src/constants.rs
  - config/default.toml
  - tests/<provider>_provider_test.rs
  - tests/<provider>_integration_test.rs

3. Implementation Steps
- Ordered steps covering API model parsing, domain conversion, provider behavior, warning propagation, and config wiring.
- Explicitly distinguish fetcher FetchOutcome from provider FetchResult usage.

4. Test Plan
- Commands to run (plain `cargo test`; per-test settings values via tests/helpers/test_utils.rs select the provider).
- Include snapshot workflow notes when output can affect rendered SVG.

5. Done Criteria
- A concise acceptance checklist for PR readiness.

Rules:
- Use project conventions from AGENTS.md.
- Link to existing files rather than duplicating large docs.
- Keep the scaffold concise and actionable.

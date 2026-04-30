---
name: "PR Review"
description: "Use when reviewing a pull request or branch diff for bugs, regressions, and missing tests. Produces severity-ordered findings with file references and validation guidance."
argument-hint: "PR number, branch, or focus area to review"
agent: "agent"
tools: [read, search, execute]
---

Perform a code review of the current changes with a bug-risk mindset.

Priorities:
- Identify behavioral regressions, logic bugs, edge cases, and test gaps.
- Focus on correctness and risk before style.
- Keep summary brief and findings detailed.

Repository-specific checks:
- Verify test commands use RUN_MODE=test when applicable.
- Check nested env var naming for config overrides (APP_API__*).
- For rendering or template changes, require snapshot workflow coverage.
- For time-dependent code, verify Clock abstraction usage instead of direct local time calls.
- For provider/fetcher changes, verify FetchOutcome and FetchResult usage remains correct.

Output format:

1. Findings (ordered by severity)
- Severity: High, Medium, Low
- For each finding include:
  - Title
  - Why it matters
  - Evidence with file references
  - Suggested fix direction

2. Open questions or assumptions
- List any unclear intent or missing context affecting confidence.

3. Testing gaps
- List missing tests or insufficient assertions.
- Include exact commands to validate fixes.

4. Brief change summary
- 2 to 5 bullets maximum.

If no findings are discovered:
- Explicitly say no actionable findings were found.
- Still report residual risks and any untested paths.

Do not make code changes unless explicitly requested.

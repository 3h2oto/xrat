---
id: TASK-42
title: Add Environment Variables Port
status: Done
assignee:
  - codex
created_date: '2026-07-05 14:43'
updated_date: '2026-10-03 06:42'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 22
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/22-env-vars-port.md`

# Add Environment Variables Port

## Finding

### [Priority: Low] Add an environment variables abstraction

**Files involved:**

- `src/app/commands/proxy/shell.rs`
- `src/app/commands/proxy/desktop.rs`
- `src/app/commands/daemon_install.rs`
- `src/app/commands/output.rs`
- `src/app/config/secret.rs` (already abstracted via closure)

**Problem:** Beyond `app_paths` and `secret.rs`, 5 production locations read
environment variables directly: `SHELL`, `HTTP_PROXY`/`HTTPS_PROXY`,
`XDG_CURRENT_DESKTOP`, `DESKTOP_SESSION`, `XDG_CONFIG_HOME`, `HOME`, and
`NO_COLOR`. Each uses `std::env::var` or `std::env::var_os` independently.

**Why this change is needed:** Environment-dependent behavior cannot be tested
without modifying the actual process environment, which affects other tests
running in the same process. `secret.rs` already shows the correct pattern (an
injectable closure), but the other sites have not been updated.

**How to implement it:** Introduce an `EnvVars` trait with a `get` method.
Provide a `SystemEnvVars` production adapter and a `HashMapEnvVars` test
adapter. Replace direct `std::env::var` calls in all production locations.
Consider standardizing on the closure pattern used by `secret.rs` instead of a
trait if the trait feels like over-engineering.

**Positive effect on the codebase:** Environment-conditional behavior (proxy
detection, desktop environment detection, color output, install paths) becomes
deterministic in tests.

**Suggested target architecture:** `EnvVars` port in `src/support/` or reuse the
closure-injection pattern from `secret.rs`. Injected into proxy, output, and
install services.

**Risk / migration notes:** Very low risk. The `secret.rs` pattern is well
understood. Prefer the simple closure approach unless the trait provides clear
value for polymorphism.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All production environment reads use the shared adapter or existing injectable secret resolver
- [x] #2 Environment-dependent policy accepts fake values without modifying the process environment
- [x] #3 Existing defaults and precedence remain compatible; focused tests and workspace gates pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add OsString-preserving EnvVars port and system/map adapters in xrat-support; migrate all environment policy consumers through injectable variants while preserving default wrappers and secret API; verify precedence and missing/empty/non-Unicode cases.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented in 6941831. Existing entry points use production adapters; injected policy variants and test fakes preserve prior behavior. CARGO_INCREMENTAL=0 just fmt ci passed: 901 Rust tests, three Python version tests, formatting and strict Clippy. Log: /tmp/xrat-ports-small-ci.log. Native non-Linux execution was not verified.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Extracted the host boundary with production defaults and injectable policy tests; workspace gates passed.
<!-- SECTION:FINAL_SUMMARY:END -->

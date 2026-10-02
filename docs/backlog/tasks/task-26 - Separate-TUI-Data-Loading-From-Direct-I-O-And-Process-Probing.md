---
id: TASK-26
title: Separate TUI Data Loading From Direct I/O And Process Probing
status: Done
assignee:
  - '@codex'
created_date: '2026-07-05 14:43'
updated_date: '2026-10-02 19:59'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-3
dependencies: []
priority: medium
ordinal: 11
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/2-use-cases/11-tui-data-loading-boundaries.md`

# Separate TUI Data Loading From Direct I/O And Process Probing

## Finding

### [Priority: Medium] Separate TUI data loading from direct I/O and process probing

**Files involved:**

- `src/tui/data/mod.rs`
- `src/tui/run/tasks/data.rs`
- `src/tui/run/tasks/version_check.rs`
- `src/tui/run/tasks/source.rs`
- `src/tui/run/tasks/runtime.rs`

**Problem:** `TuiData::load` performs repository queries, runtime status checks,
log loading, network address derivation, daemon IPC status, and view-model
construction in one function. `probe_engines` runs runtime binaries from the TUI
data module, and `version_check` performs direct HTTP requests with silent
`ok()?` failure paths.

**Why this change is needed:** TUI data loading should be a thin adapter over
application read models. Direct I/O inside TUI data modules makes update/render
tests harder and can freeze or silently degrade the UI if an external dependency
behaves unexpectedly.

**How to implement it:** Create `DashboardService` or `OverviewUseCase` in
application code to assemble configs, sources, runtime, tests, logs, daemon
info, and API URL facts. Move engine probing behind a `RuntimeEngineProbe` port.
Move latest-version checks behind an update service shared with the CLI
upgrade/update path. Keep TUI data types as view models converted from
application overview results.

**Positive effect on the codebase:** TUI update/render tests can use pure data
fixtures. Startup and refresh failures become easier to isolate, and the same
overview data can support HTTP or CLI status dashboards.

**Suggested target architecture:** TUI tasks call application overview/update
services; TUI app state stores view models; TUI views render only.

**Risk / migration notes:** Medium risk because TUI startup behavior is
user-visible. Extract read-only data assembly first, then move engine and
version probes behind ports.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Application dashboard services assemble config/source, runtime, test history, logs, daemon and API facts without depending on TUI types; TuiData conversion performs no I/O.
- [x] #2 Engine availability/version probing uses an injectable RuntimeEngineProbe port with bounded production probes and fake-based tests.
- [x] #3 TUI release checks and CLI upgrade share an injectable release service; optional TUI failures remain nonfatal and emit diagnostics.
- [x] #4 Dashboard startup and refresh preserve sorting, deleted visibility, counts, log limits, runtime ownership, GeoIP cache and background enrichment behavior.
- [x] #5 Pure conversion and service regressions plus just fmt ci pass; documentation, task evidence and handoff are updated.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Extract interface-neutral dashboard snapshots and log loading; keep TUI conversion and rendering pure. 2. Move GeoIP cache/enrichment operations behind application services while preserving asynchronous batching. 3. Add engine and release probe ports and share latest-release fetching with CLI upgrade. 4. Verify fake probes, pure conversion, cache/log behavior and existing TUI regressions; run just fmt ci, update docs/handoff and commit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Extracted dashboard snapshots, logs and GeoIP cache/background enrichment into application service modules; TUI loading is a thin adapter followed by pure conversion. Runtime bind addresses remain intact in snapshots; a separately discovered local-address fact supplies display labels. Engine and release providers are injectable, with process/HTTP adapters outside the TUI. CLI upgrade and TUI use the same release service. Runtime ownership factory remains unchanged. Existing unrelated Backlog completed-task moves and TASK-130 are preserved outside this task.

Focused service run exercised 35 tests: 34 passed, including cache/empty-cache behavior, enrichment batching/persistence, log limits, fake providers and the two-second hung-engine timeout. Corrected a migrated placeholder fixture to the original loopback_ipv4 value; final workspace verification is now running. Strict workspace Clippy and formatting have passed.

Final verification passed: CARGO_INCREMENTAL=0 just fmt ci completed with 890 Rust tests, 3 Python version-check tests, strict workspace Clippy and formatting. All extracted service regressions and pure TUI conversion tests now pass, including the corrected migrated enrichment policy test. Eight new regressions cover cache visibility/TTL, log limits, enrichment batching/empty persistence, fake engine and release providers, banner parsing, hung-process timeout and offline view conversion. Implementation is committed as a684347. No hosted CI, live GitHub API or interactive TUI run was claimed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Moved dashboard assembly, log loading, network-address facts and GeoIP cache/enrichment from TUI adapters into application services. TuiData loading delegates to DashboardService and converts DashboardSnapshot without I/O. Added injectable RuntimeEngineProbe and ReleaseProvider ports; production engine probing remains bounded to two seconds, and CLI upgrade/TUI now share ReleaseService. Optional TUI failures emit debug diagnostics and remain nonfatal. Existing runtime ownership, refresh pathways, display/counts, cache policy and log limits are preserved. Documentation and handoff updated; full workspace gate passed (890 Rust tests and 3 version tests). Next task is TASK-31.
<!-- SECTION:FINAL_SUMMARY:END -->

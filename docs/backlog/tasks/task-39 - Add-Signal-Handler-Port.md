---
id: TASK-39
title: Add Signal Handler Port
status: In Progress
assignee:
  - codex
created_date: '2026-07-05 14:43'
updated_date: '2026-10-03 08:04'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 19
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/19-signal-handler-port.md`

# Add Signal Handler Port

## Finding

### [Priority: Low] Add a signal handler port for graceful shutdown

**Files involved:**

- `src/server/mod.rs:39`
- `src/app/commands/logs.rs:144`
- `src/app/commands/test/handlers/ping.rs:35`
- `src/xray/process_mgmt/signals.rs`

**Problem:** `tokio::signal::ctrl_c()` is called in 3 places for graceful
shutdown (HTTP server, log follow, ping cancel). Process signal sending
(SIGTERM, SIGKILL via `kill` command) is in `signals.rs`. None of these have
test seams.

**Why this change is needed:** Ctrl-C handlers cannot be tested. Signal-based
process termination cannot be tested without real system processes. Extracting a
port would enable shutdown and termination tests, but the value is low since
these paths are stable and rarely change.

**How to implement it:** Introduce a `SignalHandler` trait with methods for
shutdown notification and process signal sending. Provide a production
`OsSignalHandler` and a test no-op implementation. Inject into server and
process lifecycle code.

**Positive effect on the codebase:** Server shutdown and process termination
become testable. Signal handling is centralized rather than duplicated.

**Suggested target architecture:** `SignalHandler` port in `src/support/` or
`src/app/ports/`. Used by server, log follow, ping cancel, and process
termination.

**Risk / migration notes:** Very low risk. Consider deferring until
`ProcessSpawner` and runtime lifecycle ports are in place, since signal handling
is tightly coupled to process management.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Shutdown registration and process signaling are isolated in production adapters
- [ ] #2 Injected shutdown and signal dependencies reach their consumers without replacing daemon shutdown channels
- [ ] #3 Graceful stop/escalation and shutdown failure regressions pass with workspace gates
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add separate shutdown-notification and typed process-signal ports with OS adapters; preserve daemon shutdown channels and runtime TERM/KILL escalation; inject into server/log-follow/ping and runtime lifecycle; fake tests verify cancellation and signal outcomes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementation committed in 3eba21f. Separate shutdown and process-signal ports reach server/log-follow/ping and runtime stop; daemon shutdown channels are preserved. Fake signal tests cover absent processes, graceful exit, KILL escalation and send errors. CARGO_INCREMENTAL=0 just fmt ci passed with 916 Rust tests and 3 Python tests. Remains In Progress pending shutdown consumer cancellation/registration-failure regressions and final acceptance audit.
<!-- SECTION:NOTES:END -->

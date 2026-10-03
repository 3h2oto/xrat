---
id: TASK-36
title: Extract Shared Port Waiter Abstraction
status: Done
assignee:
  - codex
created_date: '2026-07-05 14:43'
updated_date: '2026-10-03 08:42'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 16
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/16-port-waiter-abstraction.md`

# Extract Shared Port Waiter Abstraction

## Finding

### [Priority: Medium] Extract shared TCP port-waiting abstraction

**Files involved:**

- `src/xray/process_mgmt/process.rs:74-99`
- `src/xray/process/spawn.rs:63-83`
- `src/singbox/process_mgmt.rs:89-113`

**Problem:** The same `TcpStream::connect` + `Instant::now` polling loop is
duplicated in three engine startup implementations. Each polls a set of TCP
ports until they accept a connection or a timeout elapses. The logic is
identical but independently implemented: same retry interval, same overall
timeout calculation, same error reporting.

**Why this change is needed:** Three copies of the same polling loop means bug
fixes or tuning (retry interval, timeout behavior, error messages) must be
applied in three places. It also prevents testing readiness logic without
binding real TCP ports. The duplication is a clear violation of DRY in a
critical startup path.

**How to implement it:** Extract a `PortWaiter` trait and a default production
implementation. Provide methods for waiting on a single port or multiple ports.
Replace the three inline polling loops with calls to this shared abstraction.
Add a `MockPortWaiter` that returns simulated latency or timeout for testing.

**Positive effect on the codebase:** Engine startup tests can verify timeout
behavior, success timing, and partial-failure handling without real ports. The
three engine modules become smaller and easier to reason about.

**Suggested target architecture:** `PortWaiter` as a utility port in
`src/support/` or `src/app/ports/`. Used by `ProcessSpawner` consumers during
the readiness-check phase.

**Risk / migration notes:** Low risk. Pure extraction with no behavior change.
Add tests for the shared implementation first, then replace each inline loop one
at a time, verifying startup still works after each replacement.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Managed and temporary Xray/sing-box startup uses one child-aware TCP readiness implementation
- [x] #2 Injected readiness and process dependencies propagate through engine and runtime startup
- [x] #3 Success, early exit, timeout and failed-start cleanup regressions pass with workspace gates
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Share child-aware TCP readiness polling across managed and probe engine startup; inject process and readiness ports together; retain polling intervals, error mappings and cleanup; fake and loopback tests cover readiness, process exit and bounded timeout.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementation committed in 3eba21f. Four managed/temporary engine readiness loops use the shared child-aware waiter; fake tests cover retries, stalled connections, partial readiness, process exit and poll-error policy. CARGO_INCREMENTAL=0 just fmt ci passed with 916 Rust tests and 3 Python tests. Remains In Progress pending consumer-level fake startup/cleanup regressions and final acceptance audit.

Completed in ccdd6c4. Fake consumer tests exercise all four managed/temporary Xray and sing-box startup paths: success, missing binary, early exit, readiness timeout, inspection failure and cancellation. Managed success detaches ownership; failed/cancelled startup kills and reaps; temporary process drop removes its config and reaps the child. RuntimeService fake tests prove readiness dependencies reach startup for both engines and failed sessions are persisted. Existing paused-time waiter tests and real local runtime fixtures also pass. CARGO_INCREMENTAL=0 just fmt ci passed with formatting, strict workspace Clippy, 939 Rust tests and 3 Python tests.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
All four engine startup paths share child-aware readiness polling and injectable ports. Consumer and runtime tests verify success, failure, deadlines and cancellation cleanup; full workspace gates pass.
<!-- SECTION:FINAL_SUMMARY:END -->

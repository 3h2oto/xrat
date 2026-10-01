---
id: TASK-23
title: Improve Observability For Swallowed Async Errors
status: In Progress
assignee:
  - '@codex'
created_date: '2026-07-05 14:43'
updated_date: '2026-10-01 06:20'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-2
dependencies: []
priority: medium
ordinal: 9
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/1-foundation/9-async-observability.md`

# Improve Observability For Swallowed Async Errors

## Finding

### [Priority: Medium] Improve observability for swallowed async errors

**Files involved:**

- `src/tui/run/tasks/runtime.rs`
- `src/tui/run/tasks/test_batch.rs`
- `src/tui/run/tasks/data.rs`
- `src/app/daemon/ipc/handler/mod.rs`
- `src/app/daemon/ipc/handler/io.rs`
- `src/app/daemon/supervisor/handlers/runtime/runtime_status_connect.rs`

**Problem:** Several async paths intentionally ignore send failures or
persistence failures with `let _ = ...` or `.ok()`. Some paths have comments
explaining best-effort behavior, but many do not emit structured tracing fields
when a task channel, IPC connection, reload, metadata update, or event record
fails.

**Why this change is needed:** The project has daemon, TUI, process, and IPC
workflows where failures can be timing-dependent. Silent drops make production
issues hard to debug and make tests unable to assert important failure handling.

**How to implement it:** Add a small observability helper or consistent pattern
for best-effort failures: log with `tracing::debug!` or `tracing::warn!` using
operation, task kind, config id, session id, and error fields. Keep event
recording best-effort, but record trace logs when it fails. Add spans around TUI
spawned tasks, daemon IPC request dispatch, runtime connect/disconnect/replace,
and batch test runs.

**Positive effect on the codebase:** Async failures become diagnosable without
changing primary operation behavior. Logs can connect user-visible failures to
daemon, IPC, runtime, or TUI task steps.

**Suggested target architecture:** Use structured tracing consistently at
adapter boundaries and application use-case boundaries; reserve database event
records for user-facing operational history.

**Risk / migration notes:** Low risk if logs are added without changing control
flow. Keep log levels conservative to avoid noisy TUI or daemon output.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Remaining daemon supervisor best-effort metadata and response-send failures emit structured tracing
- [ ] #2 Best-effort control flow is preserved
- [ ] #3 just fmt ci passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Finish the observability slice: audit remaining app/daemon/supervisor/handlers/ for best-effort let _ = / .ok() drops, add minimal structured tracing::debug!/warn! with operation, ids, and error fields. No control-flow change.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
This PR covers the focused adapter slice: TUI runtime, batch-test, data reload/enrichment channel failures; daemon IPC request/socket failures; runtime status/connect response drops and metadata write failures. Added spans for IPC handling and runtime connect/disconnect/replace plus TUI runtime and batch-test tasks. Best-effort control flow is retained. Other daemon supervisor handlers remain for the broader follow-up.

Validation for the focused slice: CARGO_INCREMENTAL=0 just fmt ci passed. Remaining broader scope includes other daemon supervisor handlers with best-effort metadata and response sends; task stays In Progress for a later focused pass.
<!-- SECTION:NOTES:END -->

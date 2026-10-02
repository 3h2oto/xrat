---
id: TASK-31
title: Keep Daemon Supervisor Handlers Thin
status: Done
assignee:
  - codex
created_date: '2026-07-05 14:43'
updated_date: '2026-10-02 20:19'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-3
dependencies: []
priority: medium
ordinal: 5
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/2-use-cases/5-thin-daemon-supervisor-handlers.md`

# Keep Daemon Supervisor Handlers Thin

## Finding

### [Priority: High] Keep daemon supervisor handlers thin

**Files involved:**

- `src/app/daemon/supervisor/handlers/runtime/runtime_status_connect.rs`
- `src/app/daemon/supervisor/handlers/runtime/runtime_lifecycle/replace.rs`
- `src/app/daemon/supervisor/handlers/runtime/runtime_lifecycle/disconnect.rs`
- `src/app/daemon/supervisor/types.rs`
- `src/app/runtime_service/replace_flow`

**Problem:** Daemon supervisor handlers do more than dispatch. They update
runtime transition metadata, maintain rotation state fields, record events,
construct IPC payloads, and translate runtime failures. This logic is
interleaved with supervisor message handling.

**Why this change is needed:** Rotation and runtime transition rules are
application behavior. Keeping them in daemon handlers makes them hard to reuse
from CLI/TUI/API flows and hard to unit-test without supervisor channels. It
also makes debugging harder because state changes, event recording, and IPC
response construction happen in one async path.

**How to implement it:** Extract runtime transition and rotation orchestration
into application services, for example `RuntimeTransitionService` and
`RotationService`. These services should accept typed requests, update metadata,
call `RuntimeService`, record events, and return typed outcomes. Keep daemon
handlers responsible for channel receive/send, daemon-specific state fields, and
mapping outcomes to IPC payloads.

**Positive effect on the codebase:** Daemon behavior becomes easier to reason
about, rotation can be tested without IPC plumbing, and future interfaces can
reuse the same transition metadata and event behavior.

**Suggested target architecture:** Supervisor code manages scheduling and
channels; application services own runtime/rotation use-cases; event persistence
is best-effort inside the use-case layer with structured results.

**Risk / migration notes:** Medium risk because rotation state is subtle. Add
regression tests around manual replace, timer replace, health cooldown, and
metadata updates before moving logic.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Runtime connect, disconnect and replacement orchestration lives in application services with typed outcomes and no supervisor or IPC response dependencies
- [x] #2 Supervisor retains scheduling, state updates and IPC mapping; manual, timer and health cooldown behavior remains unchanged
- [x] #3 Transition metadata and operational events remain best effort and lifecycle regressions plus just fmt ci pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Preserve existing lifecycle and cooldown regression coverage.
2. Extract runtime transitions and rotation orchestration into application services.
3. Run focused lifecycle tests and just fmt ci; update handoff and close task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Deferred in this refactor run. Rotation/transition orchestration extraction from daemon supervisor handlers is the highest-risk item (rotation state is subtle). Needs its own focused pass with regression tests for manual replace, timer, health cooldown, and metadata updates. Not attempted in P4 to keep behavior-preserving guarantees.

Completed in 917b022. RuntimeTransitionService owns connect/disconnect metadata, runtime events, shutdown and health-failure persistence. RotationService owns replacement metadata/events and typed failure classification. RotationTrigger moved to application services with IPC re-export preserving wire format. Supervisor retains daemon state, scheduling, thresholds/probes and response mapping. Existing 13 supervisor regressions passed before and after extraction; three direct service tests cover owner/failure metadata and events, typed no-candidate outcomes and failed event persistence. CARGO_INCREMENTAL=0 just fmt ci passed: 893 Rust tests, three Python version tests, strict workspace Clippy and formatting. Log: /tmp/xrat-task31-ci.log. Local verification only; hosted CI and deployed engines were not verified.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Extracted runtime transition and rotation use cases from daemon handlers with typed outcomes and best-effort persistence. Preserved scheduling, cooldown and IPC behavior; all workspace gates passed.
<!-- SECTION:FINAL_SUMMARY:END -->

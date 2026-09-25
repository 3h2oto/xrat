---
id: TASK-30
title: Introduce Shared Runtime-Control Abstraction
status: Done
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-09-25 02:18'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-3
dependencies: []
priority: medium
ordinal: 4
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/2-use-cases/4-runtime-control-abstraction.md`

# Introduce Shared Runtime-Control Abstraction

## Finding

### [Priority: High] Introduce a runtime-control abstraction shared by CLI, TUI, and daemon

**Files involved:**

- `src/app/commands/connect.rs`
- `src/app/commands/disconnect.rs`
- `src/app/commands/status/mod.rs`
- `src/tui/run/tasks/runtime.rs`
- `src/app/runtime_service`
- `src/app/daemon/ipc`

**Problem:** Runtime operations are split across direct daemon IPC in CLI
commands and direct `RuntimeService` usage in TUI tasks and daemon supervisor
code. The CLI connect, disconnect, and status commands resolve socket paths,
call IPC functions, interpret daemon-unreachable errors, and format daemon
payloads themselves. The TUI starts and stops runtime sessions locally with
`RuntimeService`, which can diverge from daemon-managed behavior.

**Why this change is needed:** The project wants one runtime application core
with multiple thin adapters, but current adapters choose different control
paths. This creates inconsistent runtime semantics, duplicated
unreachable-daemon handling, and harder testing because IPC, process management,
and presentation are mixed together.

**How to implement it:** Create a `RuntimeControl` trait or enum-backed service
with methods `status`, `connect`, `disconnect`, and `replace`. Provide
implementations for daemon IPC control and local in-process control. Add a
factory that chooses the implementation from app settings and runtime mode.
Update CLI and TUI tasks to call `RuntimeControl` instead of raw IPC or raw
`RuntimeService`. Keep `RuntimeService` as the process/session core used by the
daemon/local implementation.

**Positive effect on the codebase:** Runtime behavior becomes consistent across
interfaces, and tests can inject a fake runtime controller without sockets or
subprocesses. Daemon unreachable messages and fallback policy become
centralized.

**Suggested target architecture:** `RuntimeService` owns local process/session
mechanics; `RuntimeControl` is the application-facing port; CLI, TUI, daemon
IPC, and future HTTP endpoints call the same control interface.

**Risk / migration notes:** Medium risk because runtime control is user-visible.
Start with status and disconnect, then migrate connect and replace. Preserve
existing CLI daemon behavior unless a setting explicitly selects local control.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
P4 DONE (commit ac90e4c). Added RuntimeControl trait (src/app/services/runtime_control/control.rs) with connect/disconnect/replace and shared outcome structs RuntimeConnectOutcome/RuntimeReplaceOutcome. Impls: DaemonRuntimeControl (IPC, preserves daemon-unreachable hint as AppError::InvalidArgument) and LocalRuntimeControl (RuntimeService in-process). factory exports daemon_control()/local_control(). CLI connect/disconnect now use daemon_control; TUI runtime start/stop/restart use local_control. Behavior unchanged; 858 tests pass.

Deferred: CLI status still reads the daemon payload directly because its output shape is daemon-specific (RuntimeStatusPayload). Unifying status output is follow-up work.
<!-- SECTION:NOTES:END -->

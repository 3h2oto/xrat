---
id: TASK-22
title: Centralize Application Factories And Test Setup
status: Done
assignee:
  - '@codex'
created_date: '2026-07-05 14:43'
updated_date: '2026-10-01 06:38'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-2
dependencies: []
priority: medium
ordinal: 8
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/1-foundation/8-application-factories-test-setup.md`

# Centralize Application Factories And Test Setup

## Finding

### [Priority: Medium] Centralize application factories and test setup

**Files involved:**

- `src/main.rs`
- `src/app/context.rs`
- `src/server/mod.rs`
- `src/server/tests/mod.rs`
- `src/app/commands/connect.rs`
- `src/app/commands/disconnect.rs`
- `src/app/commands/status/mod.rs`

**Problem:** There is a basic `AppContext::build` and `build_router`, but test
setup is duplicated across command and server tests. Multiple tests manually
create temp roots, database configs, runtime paths, and `AppContext` values.
Command tests for connect, disconnect, and status repeat nearly identical
`test_context` helpers.

**Why this change is needed:** Duplicated setup makes tests noisy and
inconsistent. It also discourages adding tests for new use-cases because
creating a valid app context requires copying boilerplate.

**How to implement it:** Add a shared test support module with builders such as
`TestAppBuilder`, `TestContext`, `TestDatabase`, and `TestRouter`. Provide
defaults for temp paths, SQLite database setup, seeded config nodes, runtime
paths, and server state. For production, add explicit factories such as
`build_app_context`, `build_router_from_context`, `build_daemon_runner`, and
`build_cli_runner` so composition is centralized.

**Positive effect on the codebase:** Tests become shorter, setup behavior stays
consistent, and new architecture services can be validated with less
boilerplate.

**Suggested target architecture:** Production factories wire concrete
dependencies; test factories wire temporary or fake dependencies; individual
tests focus on behavior.

**Risk / migration notes:** Low risk. Start by deduplicating command test
context setup, then server/router fixtures. Avoid changing production behavior
during the first pass.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 TestAppBuilder is the single test-context constructor used by command, runtime-service, supervisor, and TUI import tests
- [x] #2 test_node/test_source fixtures are centralized instead of duplicated
- [x] #3 Production factories build_app_context and build_router_from_context are named and used by AppContext::build and server
- [x] #4 A non-CLI AppContext constructor exists for SDK/test construction
- [x] #5 just fmt ci passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Slice 1 (test support, behavior-preserving):
1. Add TestAppBuilder in xrat-app test support with temp_root(prefix), with_default_ports(), build() -> AppContext, into_parts() -> (TempDir, AppContext).
2. Centralize test_node/test_source fixtures used by supervisor and runtime-service tests.
3. Replace duplicated test_context helpers in commands/{status,connect,disconnect,lifecycle,resolve,geoip}, runtime_service/tests, daemon/supervisor/handlers/tests, tui/run/tasks/import.
4. Verify: cargo test -q --locked green, helper-only diff.
Slice 2 (production factories):
5. Add build_app_context(args) and build_router_from_context(&ctx); route AppContext::build and server through them.
6. Add a non-CLI AppContext constructor taking resolved paths+config+db for SDK/test use.
7. Verify: just fmt ci green.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Slice 1: added crates/xrat-app/src/app/tests/{mod.rs,fixtures.rs} with TestAppBuilder (build/build_with_root/with_default_ports) and shared test_node/test_node_with/test_source fixtures. Replaced duplicated test_context/test_node/test_source helpers in commands/{status,connect,disconnect,lifecycle,resolve,geoip}, runtime_service/tests, daemon/supervisor/handlers, and tui/run/tasks/import. Slice 2: added AppContext::from_parts and build_app_context factory; server::build_router_from_context. Verified with cargo test -p xrat-app --lib (578 passed) and just fmt ci (exit 0).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Centralized app test setup and production factories. Added TestAppBuilder + shared node/source fixtures in crates/xrat-app/src/app/tests/, replacing seven duplicated test_context helpers across commands, runtime_service, daemon supervisor, and TUI import tests. Added AppContext::from_parts plus build_app_context and server::build_router_from_context as named composition seams. Verified: cargo test -p xrat-app --lib (578 passed, +4 new builder tests) and just fmt ci exit 0. Behavior-preserving; net -283 lines.
<!-- SECTION:FINAL_SUMMARY:END -->

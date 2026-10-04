---
id: TASK-19
title: Introduce Newtype Identifiers For Config And Subscription IDs
status: Done
assignee:
  - '@codex'
created_date: '2026-07-05 14:43'
updated_date: '2026-10-02 18:58'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-2
dependencies: []
priority: medium
ordinal: 25
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/1-foundation/25-newtype-ids.md`

# Introduce Newtype Identifiers For Config And Subscription IDs

## Finding

### [Priority: Medium] Replace raw ID primitives with newtype wrappers

**Files involved:**

- `src/model/`
- `src/db/record/`
- `src/db/repository/configs/`
- `src/db/repository/subscriptions.rs`
- `src/app/commands/resolve.rs`
- `src/app/commands/lifecycle.rs`
- `src/server/routes/configs.rs`
- `src/tui/data/configs.rs`

**Problem:** Config and subscription identifiers are passed around as raw
primitives (`i64` database row ids, `String` prefix/identifier tokens) through
repositories, command handlers, resolvers, HTTP routes, and TUI rows. Nothing in
the type system distinguishes a config id from a subscription id, or a resolved
numeric id from an unresolved user-supplied prefix string.

**Why this change is needed:** Raw primitive ids invite argument-swap bugs (a
function taking `(config_id: i64, subscription_id: i64)` accepts them in either
order), and they make the repository-trait signatures planned in
`01-config-query-use-cases`, `02-config-lifecycle-service`, and the
`ConfigRepository` port self-documenting only by parameter name. They also blur
the resolve step: `app/commands/resolve.rs` turns a user-supplied prefix into a
concrete id, but both ends are `String`/`i64`, so "resolved" vs "unresolved" is
not visible at call sites.

**How to implement it:** Add small newtypes in `src/model/` such as `ConfigId`,
`SubscriptionId`, and a `ConfigRef` (raw user token before resolution). Derive
the usual traits (`Copy`/`Clone`, `Eq`, `Hash`, `Display`, `serde`,
`sqlx::Type`) so they pass through repositories, DTOs, and TUI rows with no extra
mapping. Resolution (`resolve.rs`) takes a `ConfigRef` and returns a `ConfigId`,
making the transition explicit. Migrate signatures bottom-up: repositories first,
then use-cases, then adapters.

**Positive effect on the codebase:** Eliminates a class of id-mixup bugs at
compile time. Makes the upcoming repository/use-case trait signatures
(`01`, `02`, `06`, `07`) readable without comments. Encodes the resolve step in
the type system, shrinking a source of `unwrap()`s flagged in
`24-audit-production-panics`.

**Suggested target architecture:** Domain ids are newtypes owned in `src/model/`;
repositories, use-cases, DTOs, and view models all speak in newtypes; only the
SQL layer and CLI parse boundary convert to/from primitives.

**Risk / migration notes:** Low risk, mechanical, but touches many signatures. Do
it before or alongside `01`/`06` so the new use-case and read-model signatures
adopt newtypes from the start rather than being migrated twice. Keep `From`/`Into`
conversions at the SQL and CLI edges to localize the churn.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 ConfigId, SubscriptionId, and ConfigRef exist in xrat-model
- [x] #2 Repositories, services, and DTOs speak in newtypes; From/Into only at SQL and CLI edges
- [x] #3 resolve.rs maps ConfigRef -> ConfigId explicitly
- [x] #4 just fmt ci passes with no behavior change
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Complete ConfigRef-to-ConfigId resolution across command and service boundaries.
2. Carry ConfigId and SubscriptionId through HTTP DTOs and TUI state/tasks; preserve transparent JSON and displayed output.
3. Migrate test fixtures and assertions, remove migration-created unused imports.
4. Verify workspace all-target compilation, resolution and serialization regressions, SQLite/PostgreSQL paths, and just fmt ci.
5. Record acceptance evidence and finish TASK-19 before TASK-23.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Completed ConfigRef resolution, generic typed RefMatch results, HTTP DTOs, TUI state/tasks, daemon IPC and test fixture migration. Standalone cargo check -p xrat-model --features sqlx --locked passed after adding the SQLx macro feature. just fmt ci passed with local socket access: 972 Rust tests plus 3 version-tool tests; all-target Clippy -D warnings clean. Restricted run failed only socket-dependent fixtures (555 app tests passed, 23 failed). Real PostgreSQL verification exposed existing import SQL using boolean literals against integer is_deleted; source comparison confirms unchanged by this migration. User approved separate repair before repeating the backend gate.

Final backend verification: CARGO_INCREMENTAL=0 just test-postgres passed against the real local PostgreSQL database after the separately approved TASK-129 compatibility repair. Final workspace gate passed with 876 Rust tests and 3 version tests.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Config and subscription IDs are distinct newtypes throughout repositories, services, HTTP DTOs, TUI and daemon/runtime paths. ConfigRef resolves explicitly to ConfigId; ref-match results retain the ID type. Transparent serialization and Display preserve output contracts. Workspace checks and just fmt ci pass; existing PostgreSQL import SQL failure is handled separately.
<!-- SECTION:FINAL_SUMMARY:END -->

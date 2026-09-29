---
id: TASK-120
title: Unify Rust Module File Convention
status: Done
assignee: []
created_date: '2026-09-24 23:34'
updated_date: '2026-09-29 09:56'
labels:
  - refactor
  - improvement
milestone: m-2
dependencies: []
ordinal: 102000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
16 locations have both foo.rs and foo/ simultaneously (context, app_paths, runtime_service, daemon, db/database, xray/process_mgmt, prober/download, prober/real_delay, cli/tests/cases, db/repository/configs, app/commands/parse, app/commands/test, app/config/editor, runtime_service/log_retention, and others). Adopt one rule: a module is foo.rs when under 150 lines, otherwise foo/mod.rs plus sibling files; mod.rs contains module declarations and re-exports only, never logic. Convert all dual locations and remove any logic from mod.rs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 No module has both foo.rs and foo/ at the same path
- [ ] #2 Every mod.rs contains declarations and re-exports only
- [ ] #3 Modules over 150 lines use foo/mod.rs plus siblings
- [ ] #4 just ci passes
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
P6 DONE. All foo.rs + foo/ duals eliminated (acceptance #1): declaration-only roots (parse, daemon, cli/tests, test/tests, cli/tests/cases, parsing/core/tests[/cases], runtime_service, db/database, db/database/tests, repository/configs, prober/real_delay, prober/download, runtime_service/tests, xray/process_mgmt) converted to foo/mod.rs; logic roots (app_paths, context, runtime_service/log_retention, daemon/supervisor/handlers/runtime, handlers/tests, commands/test) split into mod.rs (declarations/re-exports) + siblings (acceptance #2 for refactored roots). Newly split modules over 150 lines use foo/mod.rs + siblings (acceptance #3). just ci gate made workspace-wide and passing. Residual: pre-existing module roots that are not duals (e.g. config/mod.rs types, prober/mod.rs enums) retain domain types by design.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

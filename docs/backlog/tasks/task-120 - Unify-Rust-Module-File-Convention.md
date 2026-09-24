---
id: TASK-120
title: Unify Rust Module File Convention
status: In Progress
assignee: []
created_date: '2026-09-24 23:34'
updated_date: '2026-09-24 23:34'
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
P6 target. Convention rule locked: foo.rs <=150 lines, else foo/mod.rs + siblings; mod.rs = declarations/re-exports only.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

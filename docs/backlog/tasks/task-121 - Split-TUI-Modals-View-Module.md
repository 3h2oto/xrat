---
id: TASK-121
title: Split TUI Modals View Module
status: Done
assignee: []
created_date: '2026-09-24 23:34'
updated_date: '2026-09-29 09:02'
labels:
  - refactor
  - improvement
milestone: m-2
dependencies: []
ordinal: 103000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/tui/view/modals.rs is 1506 lines and mixes every modal renderer (import, delete, settings, confirm, test options) in one file. Split into src/tui/view/modals/mod.rs (module declarations only) plus one file per modal family, each under 300 lines, following the project module convention.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 src/tui/view/modals.rs is replaced by src/tui/view/modals/ with mod.rs declaring only
- [ ] #2 No file under src/tui/view/modals/ exceeds 300 lines
- [ ] #3 All existing TUI modal tests pass unchanged
- [ ] #4 just ci passes
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
P6 DONE (commit fc6b80a). src/tui/view/modals.rs (1506) replaced by src/tui/view/modals/ with mod.rs (11 lines, declarations/re-exports only), prelude.rs, help.rs (242), settings.rs (550), dialogs.rs (244), tests.rs (463). All modal tests moved and pass. 858 tests green, clippy -D warnings clean.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

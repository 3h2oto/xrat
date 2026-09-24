---
id: TASK-126
title: Split Setup Cores Module
status: In Progress
assignee: []
created_date: '2026-09-24 23:34'
updated_date: '2026-09-24 23:34'
labels:
  - refactor
  - improvement
milestone: m-2
dependencies: []
ordinal: 108000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/app/commands/setup/cores.rs is 1142 lines and mixes engine install planning, download orchestration, version resolution, and CLI presentation. Split into src/app/commands/setup/cores/mod.rs (declarations only) plus capability files, each under 300 lines, keeping install/download orchestration out of the CLI adapter where practical.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 src/app/commands/setup/cores.rs is replaced by a cores/ directory with mod.rs declaring only
- [ ] #2 No file under setup/cores/ exceeds 300 lines
- [ ] #3 Existing setup tests pass unchanged
- [ ] #4 just ci passes
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
P6 target.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

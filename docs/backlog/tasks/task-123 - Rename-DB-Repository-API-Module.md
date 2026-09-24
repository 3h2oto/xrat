---
id: TASK-123
title: Rename DB Repository API Module
status: In Progress
assignee: []
created_date: '2026-09-24 23:34'
updated_date: '2026-09-24 23:34'
labels:
  - refactor
  - improvement
milestone: m-2
dependencies: []
ordinal: 105000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/db/repository/api/ holds persistence primitives but is named like the HTTP API and collides conceptually with src/server/. Rename to a persistence-oriented name such as db/repository/primitives/ so the module describes what it provides rather than which frontend once consumed it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 db/repository/api no longer exists
- [ ] #2 New module name reflects persistence primitives
- [ ] #3 just ci passes
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

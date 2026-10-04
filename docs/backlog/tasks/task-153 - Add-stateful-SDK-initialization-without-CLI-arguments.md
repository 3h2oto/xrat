---
id: TASK-153
title: Add stateful SDK initialization without CLI arguments
status: To Do
assignee: []
created_date: '2026-10-04 06:55'
labels: []
milestone: m-9
dependencies:
  - TASK-152
priority: high
ordinal: 135000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deferred after stateless release. Define SDK-owned settings and initialize SQLite/PostgreSQL repositories without CLI argument structures, global logging setup or hidden filesystem state.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Public initialization APIs expose every required type via xrat-sdk
- [ ] #2 SQLite and PostgreSQL integration examples work without direct internal-crate dependencies
- [ ] #3 Initialization errors and lifecycle ownership are documented and tested
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

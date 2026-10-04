---
id: TASK-154
title: Expose saved-config and subscription services through SDK
status: To Do
assignee: []
created_date: '2026-10-04 06:55'
labels: []
milestone: m-9
dependencies:
  - TASK-153
priority: high
ordinal: 136000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deferred. Build on stateful SDK initialization to expose import listing lookup export enable disable delete restore and subscription management using SDK-owned domain records rather than raw database rows.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Supported operations are callable solely through xrat-sdk public types
- [ ] #2 SQLite and PostgreSQL tests cover persistence errors and soft-delete behavior
- [ ] #3 Standalone example documents config and subscription operations
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

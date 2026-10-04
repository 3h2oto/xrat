---
id: TASK-155
title: Expose managed runtime lifecycle through SDK
status: To Do
assignee: []
created_date: '2026-10-04 06:55'
labels: []
milestone: m-9
dependencies:
  - TASK-153
  - TASK-154
priority: high
ordinal: 137000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deferred. Expose connect status disconnect with explicit binary paths, persisted session ownership, diagnostic events and deterministic cleanup. No CLI context required.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 SDK-only consumer can connect inspect and disconnect supported engines
- [ ] #2 Tests cover startup failure reattachment idempotent stop and owned-process cleanup
- [ ] #3 Public documentation defines ownership side effects and platform requirements
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

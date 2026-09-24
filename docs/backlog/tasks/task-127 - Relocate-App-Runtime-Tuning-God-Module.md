---
id: TASK-127
title: Relocate App Runtime Tuning God Module
status: In Progress
assignee: []
created_date: '2026-09-24 23:34'
updated_date: '2026-09-24 23:34'
labels:
  - refactor
  - improvement
milestone: m-3
dependencies: []
ordinal: 109000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/app/runtime_tuning.rs is 630 lines at the application root and bridges both Xray and sing-box generation options, routing, DNS, and listen-interface resolution. Move it under a capability directory (for example src/app/services/engines/ or src/app/config/engine_bridge/) split into Xray and sing-box translation files, each under 300 lines, with mod.rs declaring only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 src/app/runtime_tuning.rs no longer exists at the app root
- [ ] #2 Replacement files are each under 300 lines with a declarations-only mod.rs
- [ ] #3 Generated Xray and sing-box config output is unchanged for existing tests
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

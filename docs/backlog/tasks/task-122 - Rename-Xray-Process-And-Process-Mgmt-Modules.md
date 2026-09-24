---
id: TASK-122
title: Rename Xray Process And Process-Mgmt Modules
status: In Progress
assignee: []
created_date: '2026-09-24 23:34'
updated_date: '2026-09-24 23:34'
labels:
  - refactor
  - improvement
milestone: m-2
dependencies: []
ordinal: 104000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/xray/process/ and src/xray/process_mgmt/ (plus src/xray/process_mgmt.rs) overlap in name and responsibility: one holds low-level spawn diagnostics, the other holds managed process control and signals. Rename to intent-revealing names such as xray/probe_process/ and xray/runtime_process/ so probe and managed runtime paths are distinct.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 xray/process and xray/process_mgmt no longer both exist
- [ ] #2 New names describe probe versus managed runtime purpose
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

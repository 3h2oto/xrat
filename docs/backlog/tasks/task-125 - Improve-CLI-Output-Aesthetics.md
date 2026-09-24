---
id: TASK-125
title: Improve CLI Output Aesthetics
status: In Progress
assignee: []
created_date: '2026-09-24 23:34'
updated_date: '2026-09-24 23:34'
labels:
  - improvement
  - cli
milestone: m-2
dependencies: []
ordinal: 107000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
CLI output should be more polished and consistent without breaking --format contracts. Apply consistent table headers, column alignment, spacing, status glyphs, section spacing, and color through the existing output::Style helpers. Keep table, tsv, csv, and json formats behavior-compatible; only visual chrome changes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Table and human-readable output uses consistent headers, alignment, and spacing
- [ ] #2 Status and color helpers are reused instead of ad hoc terminal formatting
- [ ] #3 table, tsv, csv, and json output contracts stay compatible with existing tests
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

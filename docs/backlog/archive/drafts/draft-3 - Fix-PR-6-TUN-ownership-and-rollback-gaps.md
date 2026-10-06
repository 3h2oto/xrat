---
id: DRAFT-3
title: Fix PR 6 TUN ownership and rollback gaps
status: In Progress
assignee:
  - '@codex'
created_date: '2026-10-06 07:26'
labels:
  - bug
  - runtime
dependencies: []
references:
  - 'https://github.com/mhyrzt/xrat/pull/6'
priority: high
ordinal: 140000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Address the remaining reviewed PR 6 safety gaps without claiming the full TUN roadmap complete.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Cleanup refuses interfaces without a verified positive recorded kernel index and retains ownership through stop until cleanup succeeds.
- [ ] #2 Connect and daemon rotation validate ownership before teardown and attempt restoration on cleanup or startup failure.
- [ ] #3 TUN status reports effective running daemon privileges and setup handles drop-in errors with accurate restart guidance.
- [ ] #4 Focused regressions and just fmt ci pass; changes are pushed to the existing PR branch.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add ownership and handoff regression tests. 2. Enforce verified identity and preserve ownership through teardown; cover cleanup errors with rollback. 3. Wire effective daemon status and reliable systemd setup. 4. Fix test initializers, run focused checks and just fmt ci, then push to PR 6.
<!-- SECTION:PLAN:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

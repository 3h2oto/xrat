---
id: TASK-158
title: Accept neutral WebSocket and REALITY share-link parameters
status: Done
assignee:
  - '@codex'
created_date: '2026-10-04 17:28'
updated_date: '2026-10-04 17:31'
labels: []
dependencies: []
ordinal: 140000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported stored configs fail Xray generation on legacy headerType and allowInsecure parameters. Accept only values that do not change transport or security behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Neutral WebSocket headerType values generate unchanged configs; meaningful unsupported values still fail
- [x] #2 REALITY neutral insecure flags are validated without weakening authentication
- [x] #3 Regression tests and local gates pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect stored values and generator. 2. Add narrow validated handling and regressions. 3. Run focused tests and workspace gates.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
History: August 23 commit 625566e introduced per-branch consumption validation before the workspace refactor; previous global allowlist ignored these combinations. Stored WS headerType=none and REALITY allowInsecure=1 reproduced. Added baseline-equivalence regressions with malformed/conflicting rejection and documented compatibility behavior. just ci passed with socket access; initial sandbox run failed socket permission tests. just build passed; rebuilt CLI generated runtime JSON from all five actual raw configs using a temporary database copy. No remote connectivity tested; installed executable unchanged.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Accept neutral WebSocket headerType and validate/ignore REALITY legacy insecure flags without changing authentication. Focused tests, workspace CI and generation for all five reported configs passed.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

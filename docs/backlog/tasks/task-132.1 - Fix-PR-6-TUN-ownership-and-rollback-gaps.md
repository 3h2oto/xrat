---
id: TASK-132.1
title: Fix PR 6 TUN ownership and rollback gaps
status: Done
assignee:
  - '@codex'
created_date: '2026-10-06 07:31'
updated_date: '2026-10-06 07:43'
labels:
  - bug
  - runtime
dependencies: []
references:
  - 'https://github.com/mhyrzt/xrat/pull/6'
parent_task_id: TASK-132
priority: high
ordinal: 140000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Address remaining PR 6 safety findings; the full parent TUN roadmap remains open.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cleanup requires a verified positive kernel index and retains ownership through teardown.
- [x] #2 Connect and rotation preserve healthy sessions on validation failure and attempt restoration on handoff cleanup or startup failure.
- [x] #3 TUN status reports running daemon privileges and systemd setup failures are actionable.
- [x] #4 Focused regressions and just fmt ci pass; a focused fix is pushed to PR 6.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add ownership and connect/rotation regression tests. 2. Require verified identity, preserve ownership, and cover cleanup failures with rollback. 3. Wire effective daemon status and reliable systemd setup. 4. Run just fmt ci and push the fix.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The original missing-index regression failed before the fix. 33 TUN-focused tests passed after requiring verified identity, retaining ownership across stop, and sharing rollback handling. Added mock handoff coverage for connect and rotation with cleanup/startup failures and identity changes; added daemon socket peer privilege inspection. Formatting and workspace Clippy pass; full workspace tests are running. Privileged live TUN deletion is not exercised.

Formatting, version checks, Python tooling tests and workspace Clippy passed via just fmt ci. The sandbox initially denied TCP/Unix socket creation; rerunning just test with local socket access passed the full workspace suite, including all 644 xrat-app tests and the daemon peer privilege regression. Code commit e45acb203aa76d48ec1c97562c89cfdc093ee399 was pushed to f02xygen/xrat feat/runtime-tun-capture. Archived DRAFT-3 is superseded by this subtask to avoid reusing an existing task ID on master. Privileged live TUN cleanup remains outside these mock regressions.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Closed missing-index cleanup authorization and post-stop cleanup rollback gaps for connect and rotation. Ownership is preserved through teardown; effective daemon privilege status, systemd setup errors and restart guidance are corrected. Added deterministic regressions and repaired CI test initializers. Local fmt/Clippy/workspace tests pass; code pushed to PR 6. Live privileged TUN traffic/deletion is not claimed.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

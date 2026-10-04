---
id: TASK-156
title: Add validated V2Ray SDK JSON generation and probing
status: To Do
assignee: []
created_date: '2026-10-04 06:55'
labels: []
milestone: m-9
dependencies:
  - TASK-150
priority: high
ordinal: 138000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deferred by user. Add a separate V2Ray target rather than relabeling Xray output. Research and document supported protocol transport TLS DNS routing and tuning combinations against a pinned official core, then expose typed generation and probing. Coordinate with TASK-134 without implementing TUN here.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Supported matrix and compatibility baseline are documented from official sources
- [ ] #2 Generated JSON passes pinned native V2Ray validation and unsupported options fail explicitly
- [ ] #3 SDK-only example generates JSON and probes with an explicit V2Ray binary
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

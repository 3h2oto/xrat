---
id: TASK-150
title: Expose stateless SDK APIs and gate experimental services
status: Done
assignee:
  - '@codex'
created_date: '2026-10-04 06:55'
updated_date: '2026-10-04 07:22'
labels: []
milestone: m-9
dependencies: []
priority: high
ordinal: 132000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deliver parsing, normalized models, typed Xray/sing-box generators and executable probes through xrat-sdk without internal-crate imports. Preserve existing generation defaults and put stateful services behind an optional services feature.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Default SDK exposes parsing, serializable models, engine generators and all required option types plus five executable probe functions
- [x] #2 Default dependency tree excludes xrat-app, xrat-db, CLI, TUI and HTTP server dependencies; services feature preserves experimental exports
- [x] #3 Existing generator/prober behavior is preserved and unsupported input remains rejected
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add curated engines modules and prober exports; make xrat-app optional; verify defaults and services builds plus parser-to-generator regression tests.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented SDK engine modules and supporting config/option exports, five executable probe exports and optional experimental services feature. Default and services SDK tests and examples pass; native validators accept all seven supported protocol fixtures after TASK-157 mapping fix.

Final local gates passed: just fmt ci, just sdk-check and just sdk-native /usr/local/bin/xray /tmp/sing-box-1.13.21-linux-amd64/sing-box.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published API implementation exposes parsers, normalized models, typed Xray/sing-box config generation and executable probes. Experimental service imports require services; default dependency tree excludes application/database/UI layers. Verified default/services builds, strict lint, doctest and standalone consumer.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

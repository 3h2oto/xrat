---
id: TASK-150
title: Expose stateless SDK APIs and gate experimental services
status: Done
assignee:
  - '@codex'
created_date: '2026-10-04 06:55'
updated_date: '2026-10-04 07:37'
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
Expose curated stateless generators/probes and optional services. Gate xrat-engines traffic stats behind a default-on stats feature for direct engine consumers; disable engine defaults in workspace dependencies and explicitly enable stats in root/application packages. Verify default SDK excludes tonic/Axum as well as app/database/UI dependencies, with full application behavior unchanged.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented SDK engine modules and supporting config/option exports, five executable probe exports and optional experimental services feature. Default and services SDK tests and examples pass; native validators accept all seven supported protocol fixtures after TASK-157 mapping fix.

Final local gates passed: just fmt ci, just sdk-check and just sdk-native /usr/local/bin/xray /tmp/sing-box-1.13.21-linux-amd64/sing-box.

Final dependency audit found tonic and Axum still pulled through the engine TUI stats module. Reopened the dependency-boundary criterion to gate those unused stats dependencies before release; this completes the originally planned server-dependency exclusion.

Final boundary audit completed: engine stats is default-on for direct engine users, disabled for workspace stateless consumers, and explicitly enabled by xrat/root and xrat-app. Strengthened checks exclude Axum/Tonic/Prost in both workspace and standalone consumer. just fmt ci, just sdk-check and pinned native tests pass after this change.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented stateless parsing/model/generator/probe exports and optional experimental services. Default SDK excludes application/database/UI and unused stats RPC/server dependencies. Existing application stats remains enabled. Verified workspace, feature matrix, rustdoc, independent consumer and pinned native lifecycle/config checks.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

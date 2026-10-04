---
id: TASK-151
title: SDK consumer examples documentation and conformance checks
status: Done
assignee:
  - '@codex'
created_date: '2026-10-04 06:55'
updated_date: '2026-10-04 07:22'
labels: []
milestone: m-9
dependencies:
  - TASK-150
priority: high
ordinal: 133000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Provide runnable parsing, node JSON, engine JSON, TCP and real-delay examples and a standalone consumer outside workspace membership. Reuse TASK-102 and TASK-109 for full sing-box conformance; this task validates representative SDK outputs only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Standalone consumer directly depends only on xrat-sdk and ordinary third-party crates
- [x] #2 Examples and doctests compile and deterministic probing tests cover success failures timeouts and cancellation cleanup
- [x] #3 Representative generated outputs pass pinned native Xray and sing-box checks; feature matrix and dependency exclusions are checked
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add SDK examples and standalone consumer; exercise public APIs and compile every supporting type; add local probe fixtures and opt-in pinned native validation/cleanup tests; wire SDK and consumer checks into Justfile and CI. Full sing-box conformance remains TASK-102/TASK-109.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added six runnable examples, standalone consumer with only SDK/serde_json/Tokio dependencies, SDK/public dependency checks and pinned CI native job. Native checks pass on Xray 26.3.27 and sing-box 1.13.21, including HTTP success/rejection, request timeout, download/upload, cancellation, startup failures and process cleanup. Full workspace and independent-consumer gates still finishing.

Final local gates passed: just fmt ci, just sdk-check and just sdk-native /usr/local/bin/xray /tmp/sing-box-1.13.21-linux-amd64/sing-box.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added six examples, independent consumer, SDK feature/dependency/rustdoc gate and pinned native CI. just sdk-check, just fmt ci and native Xray 26.3.27/sing-box 1.13.21 fixtures pass. Local probe lifecycle coverage includes startup failure, timeout, HTTP rejection, download/upload and cancellation cleanup. Broader engine conformance remains deferred tasks.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

---
id: TASK-157
title: Fix Xray outbound protocol names for Shadowsocks and SOCKS
status: Done
assignee:
  - '@codex'
created_date: '2026-10-04 07:04'
updated_date: '2026-10-04 07:22'
labels: []
milestone: m-9
dependencies: []
priority: high
ordinal: 139000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Native SDK validation against Xray 26.3.27 discovered that normalized ss and socks5 protocol names were emitted directly. Xray requires shadowsocks and socks; correct only the engine mapping, preserve normalized models and add native/regression coverage.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Shadowsocks and SOCKS generated runtime and probe configs use native Xray protocol names
- [x] #2 Both outputs pass pinned native validation and normalized protocol names remain unchanged
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Correct explicit protocol mapping in node_to_outbound, assert mapping through SDK generation fixtures and run native Xray validator.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Pinned native Xray validation reproduced unknown config id ss before the fix. Corrected mappings to shadowsocks and socks with a normalized-model-preserving regression test. All seven protocol native fixtures pass after the correction.

Final local gates passed: just fmt ci, just sdk-check and just sdk-native /usr/local/bin/xray /tmp/sing-box-1.13.21-linux-amd64/sing-box.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed only Xray protocol-name mapping for normalized Shadowsocks/SOCKS nodes; normalized model values remain unchanged. Regression test and pinned native runtime validation pass for both affected protocols and all supported fixture protocols.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

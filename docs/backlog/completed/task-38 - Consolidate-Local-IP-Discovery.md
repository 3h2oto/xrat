---
id: TASK-38
title: Consolidate Local IP Discovery
status: Done
assignee:
  - codex
created_date: '2026-07-05 14:43'
updated_date: '2026-10-03 06:42'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 18
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/18-local-ip-resolver-port.md`

# Consolidate Local IP Discovery

## Finding

### [Priority: Low] Consolidate local IP discovery into shared helper

**Files involved:**

- `src/support/net.rs:13-15`
- `src/app/commands/runtime_output.rs:22-25`

**Problem:** The same `UdpSocket::bind("0.0.0.0:0")` + `connect("8.8.8.8:80")`

- `local_addr()` trick is implemented in two places: once as a shared helper in
  `support/net.rs` and once inlined in `runtime_output.rs`. Both are concrete
  system calls with no test seam.

**Why this change is needed:** The duplication is unnecessary. When the
`runtime_output.rs` version drifts from the shared version, behavior becomes
inconsistent.

**How to implement it:** Remove the inline duplication and have
`runtime_output.rs` use `support::net::primary_ip()`. Optionally extract a
`LocalIpResolver` trait with a `MockLocalIpResolver` for tests if this function
needs to be testable (currently low value).

**Positive effect on the codebase:** One less piece of duplicated socket logic.
The primary IP resolution is in one place.

**Suggested target architecture:** Keep in `src/support/net.rs` as a concrete
helper. A trait is only warranted if tests need to control the resolved IP.

**Risk / migration notes:** Very low risk. Mechanical replacement of inline code
with a function call.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 One production UDP primary-address discovery implementation serves CLI and dashboard
- [x] #2 Injectable resolver covers success, unavailable address and specific bind behavior
- [x] #3 Existing output behavior and focused tests pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Consolidate UDP local-address discovery in xrat-support; expose a typed LocalIpResolver with a production adapter and preserve existing formatting/default wrappers; use injected resolver in dashboard and CLI tests.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented in 6941831. Existing entry points use production adapters; injected policy variants and test fakes preserve prior behavior. CARGO_INCREMENTAL=0 just fmt ci passed: 901 Rust tests, three Python version tests, formatting and strict Clippy. Log: /tmp/xrat-ports-small-ci.log. Native non-Linux execution was not verified.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Extracted the host boundary with production defaults and injectable policy tests; workspace gates passed.
<!-- SECTION:FINAL_SUMMARY:END -->

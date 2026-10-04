---
id: TASK-18
title: Audit Production unwrap() And expect() Panics
status: Done
assignee:
  - '@codex'
created_date: '2026-07-05 14:43'
updated_date: '2026-10-02 18:58'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-2
dependencies: []
priority: medium
ordinal: 24
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Audit current production unwrap()/expect() sites under crates/ using compiler diagnostics, excluding cfg(test) modules and test support. The historical estimate of 123 sites under src/ is obsolete after the workspace refactor.

Baseline compiler audit: cargo clippy --locked --workspace --lib --message-format=json -- -W clippy::unwrap_used -W clippy::expect_used reports four sites:
- xrat-support/src/engine_log.rs: date.take().unwrap() after setting Some; eliminate the optional-state round trip.
- xrat-app/src/app/services/runtime_tuning/singbox.rs: expect after unconditional insertion of a local fallback whenever no final server was selected; provable invariant.
- xrat-app/src/app/config/editor/values.rs: expect after replacing any non-table parent with a Table; provable invariant.
- xrat-app/src/app/commands/setup/cores/release.rs: parse of the compile-time pinned version; provable invariant covered by setup tests.

Retain documented invariant expects, remove unnecessary unwraps, and guard already-clean daemon/runtime/log-parser modules against new unwrap/expect sites. Do not add repository-wide panic lint policy or alter recoverable error semantics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Current production compiler audit is recorded and every unwrap/expect site is triaged
- [x] #2 Unnecessary log-parser unwrap is removed; retained expects document provable invariants
- [x] #3 Scoped production Clippy guards prevent new unwrap/expect in audited daemon, runtime and log parsing modules
- [x] #4 Focused log parsing regressions and just fmt ci pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Refresh production Clippy audit across workspace libraries and binaries; classify sites.
2. Remove redundant optional-date unwrap and verify dated, date-only, and undated log parsing.
3. Add non-test unwrap/expect deny attributes only to audited clean modules.
4. Run focused regressions, repeat audit, and just fmt ci; record remaining invariant sites and close.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Compiler audit across production workspace libraries and binaries found four sites. Removed redundant log-parser unwrap; three documented invariant expects remain (fallback DNS server, ensured TOML table, pinned version). Added production-only Clippy deny guards to daemon, runtime service and engine log parsing. Two parser regressions pass; final audit has zero unwrap diagnostics and three invariant expect diagnostics. CARGO_INCREMENTAL=0 just fmt ci passed (876 Rust tests and 3 version tests).
<!-- SECTION:FINAL_SUMMARY:END -->

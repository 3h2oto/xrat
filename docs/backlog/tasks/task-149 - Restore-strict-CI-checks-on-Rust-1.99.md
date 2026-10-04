---
id: TASK-149
title: Restore strict CI checks on Rust 1.99
status: Done
assignee:
  - '@codex'
created_date: '2026-10-03 12:50'
updated_date: '2026-10-04 06:09'
labels: []
dependencies: []
priority: high
ordinal: 131000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
PR 4 fails strict Clippy on Rust 1.99 because async-trait generates redundant must_use attributes and fetch_update is deprecated. Fix the causes without changing runtime or test expectations and preserve Rust 1.95 compatibility.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Strict workspace Clippy passes on Rust 1.99 without relaxing CI warning gates
- [x] #2 Runtime behavior and existing test expectations are preserved
- [x] #3 Required formatting and workspace tests pass and the updated PR receives green CI
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reproduce the CI diagnostics with Rust 1.99. 2. Apply narrowly scoped macro-compatible and atomic-helper fixes. 3. Run required checks on local and CI toolchains. 4. Commit the fix separately, push the pending branch commits and verify PR CI.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Resolved Rust 1.99 double_must_use diagnostics with documented exceptions limited to async-trait-decorated traits; no global warning gates changed. Replaced deprecated fetch_update in FakeConnector with a semantically equivalent saturating compare_exchange loop retaining SeqCst ordering and old-value behavior. No production runtime logic or test expectations changed. Local verification: Rust 1.99 just fmt ci and Rust 1.95 just ci both passed, each including 939 Rust tests and 3 release-tool Python tests. Source fix 71c66bb was pushed together with the seven previously pending commits to origin/refactor-r1-layering. Remote verification: PR 4 CI run 37181792060 passed all steps on 71c66bbbaa2e01084e8e304dacf5a9599f4703d3; Docs run 37181792078 also passed its build. This task completion record is committed separately from the code fix. No user-facing behavior change or migration.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Restored strict CI on Rust 1.99 while preserving Rust 1.95 compatibility. All local workspace checks and GitHub PR CI/docs checks passed. Published the CI fix and pending split commits to PR 4; no merge performed.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

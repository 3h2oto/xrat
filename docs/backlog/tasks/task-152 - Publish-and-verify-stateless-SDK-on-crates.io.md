---
id: TASK-152
title: Publish and verify stateless SDK on crates.io
status: In Progress
assignee:
  - '@codex'
created_date: '2026-10-04 06:55'
updated_date: '2026-10-04 07:22'
labels: []
milestone: m-9
dependencies:
  - TASK-151
  - TASK-157
priority: high
ordinal: 134000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prepare shared v0.21.0 release notes and package metadata, verify archives and registry dependencies, publish through the existing workflow and prove normal cargo add xrat-sdk from a fresh project.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Required workspace and SDK checks pass and package contents are verified
- [ ] #2 Shared release publishes all required crates in dependency order
- [ ] #3 Fresh registry-only consumer adds xrat-sdk and runs parsing JSON generation and TCP examples without Git or path dependencies
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Update workspace to 0.21.0 using just set-version; update release notes and SDK metadata; run workspace SDK and pinned native gates; inspect packages; commit in focused splits, push and verify PR CI; publish annotated shared release tag through existing workflow; verify registry-only consumer and record exact release state.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Version synchronized to 0.21.0 via just set-version; release notes cover changes since 0.20.0 and services opt-in. SDK package file listing contains README, all six examples, public source and tests. GitHub CARGO_REGISTRY_TOKEN secret exists; v0.21.0 remote tag is not yet present. Publication and registry-only consumer remain pending.

just fmt ci passed: strict workspace lint, 946 Rust tests/doctests and 3 Python tests. Offline SDK archive preparation cannot resolve the unpublished optional xrat-app 0.21.0 dependency; file listing is verified, and actual archive preparation/verification must follow dependency-order publication in the release workflow.

just sdk-check passed including default/services API tests, all six example builds, strict all-features lint, rustdoc -D warnings and standalone parsing/JSON/TCP execution. Registry-only verification remains pending publication.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

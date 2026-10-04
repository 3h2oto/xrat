---
id: TASK-152
title: Publish and verify stateless SDK on crates.io
status: Done
assignee:
  - '@codex'
created_date: '2026-10-04 06:55'
updated_date: '2026-10-04 08:30'
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
- [x] #2 Shared release publishes all required crates in dependency order
- [x] #3 Fresh registry-only consumer adds xrat-sdk and runs parsing JSON generation and TCP examples without Git or path dependencies
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

PR #5 merged after final head 5bc46827bee095795e3578d4f6749815dfbd7abe passed GitHub Check, SDK native engines and Docs Build. Annotated v0.21.0 tag points to merge 5adb4d65f23311866fb3bdfced9bbddcc2826e72. Release run https://github.com/mhyrzt/xrat/actions/runs/37186576140 passed CI and version gates and is building all four Linux/macOS archives; publication and registry verification remain pending.

GitHub release and all four archives/checksums plus Docker image published successfully. crates.io accepted xrat-model/support/config/engines/db 0.21.0, then rate-limited creation of xrat-prober (HTTP 429: retry after 2026-10-04 08:00:02 UTC). The failed publication job exhausted its short retry loop; resume only that job after the supplied deadline, skipping existing versions. No source/tag changes are needed.

Publication rerun accepted xrat-prober 0.21.0 and verified the packaged xrat-app successfully, but creation of xrat-app hit the same HTTP 429 quota with a new deadline of 2026-10-04 08:10:02 UTC. Resume the failed publication job after that deadline. This is a registry quota, not a source/package build failure.

Third publication attempt accepted xrat-app 0.21.0. xrat-sdk packaged verification passed, but its first upload hit the new-crate quota with deadline 2026-10-04 08:20:02 UTC. All seven SDK dependency crates are now live; only SDK upload, existing xrat version upload and registry consumer remain. Resume the failed job after the final supplied cooldown.

All nine workspace crates at 0.21.0 are published. The fresh registry consumer successfully ran parsing, normalized JSON, Xray JSON, sing-box JSON and TCP. The original release job then falsely rejected the local consumer package as a path dependency. Fixed verification to exclude workspace root packages while requiring registry sources for XRAT dependencies; added regression coverage for local consumers and rejected path/Git dependencies. just sdk-registry 0.21.0 now passes. The historical tagged release run remains failed at its old checker; publication succeeded and the corrected independent verification passes. Future releases use the corrected checker.

Final corrected checker validation: just fmt ci passed (workspace formatting, strict Clippy, Rust tests/doctests and five Python regression tests); just sdk-registry 0.21.0 passed with registry sources verified for every XRAT dependency.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published the shared v0.21.0 release and all nine crates, including xrat-sdk with empty default features and optional experimental services. Normal cargo add xrat-sdk works without Git; fresh registry parsing/JSON/probing verification passes. SDK native conformance, workspace gates and PR CI passed. Stateful embedding and V2Ray remain deferred in TASK-153 through TASK-156. Corrected a release-verifier false positive without changing published library artifacts.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

---
id: TASK-128
title: Repair final refactor review regressions and adapter drift
status: Done
assignee:
  - '@codex'
created_date: '2026-09-29 12:57'
updated_date: '2026-09-30 07:02'
labels: []
dependencies: []
priority: high
ordinal: 110000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Fix the final branch review findings: crate migration packaging, Docker workspace inputs, numeric ref lookup, release version synchronization, and remaining CLI/TUI service drift.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Isolated database builds include embedded migrations and Docker source context resolves workspace manifests
- [x] #2 Config identifiers resolve consistently through CLI and HTTP including numeric ref prefixes
- [x] #3 Version bump workflow updates workspace package and internal dependency versions together
- [x] #4 TUI lifecycle operations, test execution, and config mapping use shared application services and models
- [x] #5 Runtime control ownership policy is explicit and tested; packaging config symlink resolves
- [x] #6 Workspace checks and focused regression checks pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Repair package assets and release version workflow. 2. Unify identifier and lifecycle services. 3. Move test execution and config mapping into application services. 4. Document and test runtime control policy. 5. Run focused regressions, package checks, and workspace CI.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Moved 45 unchanged migration assets into xrat-db and kept a root symlink; included workspace crates in Docker source context; unified numeric/ref resolution; added atomic version bump tooling; routed TUI mutations, tests, and read models through services; selected daemon ownership for TUI when reachable. Validation: just fmt ci and just build passed; isolated offline xrat-db check without root migrations passed; Docker COPY-context offline workspace check passed; temporary 0.21.0 bump plus locked workspace check passed; package lists include migrations and app templates; git diff --check passed. A direct cargo package --offline verify cannot resolve unpublished xrat-config from the registry, so package list and isolated workspace build were used instead. No Docker image build or PostgreSQL integration run.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed packaging, Docker inputs, ref lookup, version workflow, TUI service drift, test execution ownership, shared config mapping, runtime ownership policy, and packaging symlink. Verified by workspace CI, build, package contents, isolated source-context checks, and temporary version bump.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

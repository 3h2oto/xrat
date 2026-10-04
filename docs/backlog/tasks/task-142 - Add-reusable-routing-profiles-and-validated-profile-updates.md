---
id: TASK-142
title: Add reusable routing profiles and validated profile updates
status: To Do
assignee: []
created_date: '2026-10-03 09:34'
updated_date: '2026-10-04 17:35'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies:
  - TASK-137
references:
  - 'https://throneproj.github.io/guides/routing/'
  - 'https://www.happ.su/main/dev-docs/routing'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 131000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

Happ and Throne expose reusable/shareable routing profiles. XRAT has global Direct/Block lists but no typed named routing policy with explicit activation and update lifecycle. Add local named profiles first, then an explicit bounded import/update contract; never accept a full engine root config as a routing profile.

V2Ray and Xray profile activation replaces the generated routing.rules/domainStrategy and default/catch-all outbound. Their field spellings and rule semantics require separate compilers.

sing-box profile activation replaces route.rules/final and referenced route.rule_set assets; dns.rules changes only when the profile explicitly contains a supported DNS policy. Rules targeting saved nodes/chains require validated generated outbound references.

Profile metadata, activation, source URL and update timing are XRAT state, not core JSON fields. Updates should stage and validate atomically before replacing last-known-good policy; define whether applying a profile requires a supervised restart. Existing asset work remains TASK-63/TASK-100.1/TASK-117. Happ deeplink formats are comparison evidence, not an instruction to invent XRAT compatibility.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Named profiles can be selected, cloned and exported using a documented typed format, preserving rule order and effective default routing.
- [ ] #2 Imports/updates reject unsupported matches, missing tags/assets and invalid schemas while retaining the last-known-good profile.
- [ ] #3 Native generation fixtures for all engines and CLI/TUI docs make activation and restart/update behavior explicit.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

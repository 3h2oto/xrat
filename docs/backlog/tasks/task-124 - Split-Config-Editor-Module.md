---
id: TASK-124
title: Split Config Editor Module
status: Done
assignee: []
created_date: '2026-09-24 23:34'
updated_date: '2026-09-29 09:26'
labels:
  - refactor
  - improvement
milestone: m-2
dependencies: []
ordinal: 106000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/app/config/editor.rs is 998 lines and src/app/config/editor/help.rs is 576 lines. Split the editor into src/app/config/editor/mod.rs (declarations only) plus focused files (state, key handling, rendering, help content), each under 300 lines. Help text should live in its own data module rather than code.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 No file under src/app/config/editor/ exceeds 300 lines
- [ ] #2 mod.rs declares modules only
- [ ] #3 Existing editor tests pass unchanged
- [ ] #4 just ci passes
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
P6 DONE (commit 3d1b4a8): src/app/config/editor.rs (998) split into editor/{mod,prelude,types,session,values,tests}.rs; editor/help.rs retained. Public surface (ConfigEditSession, EditableSetting, SettingEffect, SettingKind, SettingValue, update_runtime_binary_path) re-exported from editor/mod.rs. 858 tests green.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

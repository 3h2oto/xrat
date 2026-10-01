---
id: TASK-21
title: Split Large Command And Schema Files
status: Done
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-09-29 10:26'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-2
dependencies: []
priority: medium
ordinal: 27
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Sizes refreshed 2026-09-25 after re-measure (previous citations were stale):**

- src/app/commands/validate.rs — 1492 lines (was 442; grew 3x), still mixes diagnostics model, config checks, and CLI rendering
- src/tui/view/modals.rs — 1506 lines
- src/app/commands/setup/cores.rs — 1142 lines
- src/app/config/editor.rs — 998 lines (plus editor/help.rs 576)
- src/app/commands/list.rs — 737 lines
- src/db/schema.rs — 702 lines
- src/app/commands/daemon_install.rs — 675 lines
- src/app/commands/rotate.rs — 659 lines
- src/app/runtime_tuning.rs — 630 lines (at app root, bridges xray+singbox)
- src/app/commands/proxy/desktop.rs — 618 lines, shell.rs — 614 lines
- src/tui/run/mod.rs — 599 lines
- src/app/commands/logs.rs — 568 lines

Target: each split file under 300 lines, most under 150. Module convention: foo.rs <=150 lines, else foo/mod.rs + siblings, with mod.rs holding declarations and re-exports only. Companion split tasks: TASK-120 (convention), TASK-121 (modals), TASK-124 (editor), TASK-126 (setup cores), TASK-127 (runtime_tuning).
<!-- SECTION:DESCRIPTION:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
P6 DONE: god files split (validate, tui/view/modals, setup/cores, app/config/editor, db/schema, runtime_tuning relocation, commands/list). Remaining >300-line files are cohesive units or test files and can be split incrementally under TASK-120 follow-ups.
<!-- SECTION:NOTES:END -->

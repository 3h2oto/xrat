---
id: TASK-148
title: Keep crate module files declarative and extract large tests
status: Done
assignee:
  - '@codex'
created_date: '2026-10-03 11:53'
updated_date: '2026-10-03 12:06'
labels: []
dependencies: []
ordinal: 130000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Move implementation out of mod.rs throughout workspace crates, preserve module paths and visibility, and extract large inline test modules into adjacent tests.rs files.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Crate mod.rs files contain only module declarations, imports, re-exports and module attributes
- [x] #2 Inline test modules of at least 80 lines are extracted without changing test behavior
- [x] #3 Workspace formatting, strict Clippy and tests pass with public paths preserved
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inventory module implementations and large test blocks. 2. Move implementations into named sibling modules while preserving visibility and child paths. 3. Extract large inline tests. 4. Audit module contents and run the workspace CI checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Audited all 158 crate mod.rs files; 67 originally contained implementation or inline modules and now contain declarations/imports/re-exports only. Extracted all 16 inline tests modules of at least 80 lines, plus smaller inline modules from mod.rs. Used a temporary Rust syntax audit outside the repository to compare all 3189 function signatures and bodies against HEAD, accounting for formatting and adjusted super paths; no functions were added, removed or behaviorally edited. Added the module/test layout rules to AGENTS.md. Validation: cargo check --locked --workspace --all-targets passed; just fmt ci passed, including strict Clippy, 939 Rust tests and 3 release-tool Python tests. Existing root src facade cleanup was preserved. No user-facing behavior change or commits.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Moved module implementation into named sibling files, preserved public exports and module paths, and extracted large inline tests into nearby tests.rs files. All crate mod.rs files are declarative. Formatting, strict lint and all workspace tests passed. Changes remain uncommitted.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

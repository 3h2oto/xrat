---
id: TASK-41
title: Add Clipboard Port
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
ordinal: 21
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/21-clipboard-port.md`

# Add Clipboard Port

## Finding

### [Priority: Low] Add a clipboard abstraction for TUI share

**Files involved:**

- `src/tui/run/tasks/share.rs:6,83-103`

**Problem:** `arboard::Clipboard::new()` + `set_text()` is called directly in
the TUI share task. The clipboard is a system GUI dependency that fails in
headless environments and cannot be mocked.

**Why this change is needed:** The TUI share test either requires a display
server or is skipped. An abstraction would let tests verify copy behavior
without a clipboard backend.

**How to implement it:** Introduce a `Clipboard` trait with a `copy_text`
method. Provide a `ArboardClipboard` production adapter and a `MockClipboard`
test adapter that stores the last copied text in memory.

**Positive effect on the codebase:** TUI share feature becomes testable in
headless CI. Clipboard failures (common in SSH sessions) can be handled
gracefully with a fallback that logs the text instead of crashing.

**Suggested target architecture:** `Clipboard` port in `src/support/` or
`src/tui/ports/`. Injected into the TUI share task.

**Risk / migration notes:** Very low risk. Minor feature, small surface area.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Native clipboard access is confined to its production adapter and injected into TUI copy handling
- [x] #2 Headless tests cover successful copying, unavailable backend, write error and multibyte preview
- [x] #3 Clipboard ownership and existing TUI feedback remain compatible
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Extract Clipboard port and thread-local owning Arboard adapter; preserve TUI copy feedback and inject fake clipboard into share tests; fix multibyte preview truncation and verify headless failures.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented in 6941831. Existing entry points use production adapters; injected policy variants and test fakes preserve prior behavior. CARGO_INCREMENTAL=0 just fmt ci passed: 901 Rust tests, three Python version tests, formatting and strict Clippy. Log: /tmp/xrat-ports-small-ci.log. Native non-Linux execution was not verified.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Extracted the host boundary with production defaults and injectable policy tests; workspace gates passed.
<!-- SECTION:FINAL_SUMMARY:END -->

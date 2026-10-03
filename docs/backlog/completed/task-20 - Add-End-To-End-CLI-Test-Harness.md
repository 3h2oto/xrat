---
id: TASK-20
title: Add End-To-End CLI Test Harness
status: Done
assignee:
  - '@codex'
created_date: '2026-07-05 14:43'
updated_date: '2026-09-30 09:17'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-2
dependencies: []
priority: medium
ordinal: 26
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/1-foundation/26-end-to-end-cli-tests.md`

# Add End-To-End CLI Test Harness

## Finding

### [Priority: Medium] Add black-box end-to-end CLI tests

**Files involved:**

- `tests/` (new top-level integration test directory)
- `src/cli/tests/` (existing inline parser tests — stay as-is)
- `Cargo.toml` (`[dev-dependencies]`: `assert_cmd`, `predicates`, `tempfile`)

**Problem:** There is no top-level `tests/` directory. All ~88 test modules are
inline `#[cfg(test)]` units close to the code they validate. That is correct for
parser/normalization/repository units, but it leaves the actual CLI wiring —
argument parsing → command dispatch → context build → repository → output —
untested as a whole. A regression in how `main.rs`/`src/cli` routes a command to
its handler, or in exit codes and stdout format, is not caught by any inline
test.

**Why this change is needed:** The architecture goal is thin adapters over shared
use-cases. As use-cases get extracted (`01`–`05`) and adapters get rewired, the
risk shifts from "is the logic correct" (covered by units) to "is the command
wired to the right use-case and rendering the right output". Black-box tests pin
that contract. They also give a safety net for the larger refactors in this
backlog: run the same CLI flow before and after and assert identical output.

**How to implement it:** Add `tests/` integration tests using `assert_cmd` to
invoke the built binary against a temp `XRAT` home (`tempfile`), asserting exit
codes and stdout/stderr with `predicates`. Cover the core lifecycle end-to-end:
`init` → `import <fixture>` → `list` (table + `--format json`) → `show config` →
`delete config` → `purge`. Reuse the seeding/fixture helpers from
`08-application-factories-test-setup` so unit and e2e tests share setup. Keep
these tests network- and daemon-free; gate anything requiring xray/sing-box
binaries behind a feature or skip.

**Positive effect on the codebase:** Catches CLI-wiring and output-format
regressions that inline unit tests structurally cannot. Gives the use-case
extraction work a behavior-preserving harness. Documents the supported CLI flows
by example.

**Suggested target architecture:** Inline `#[cfg(test)]` modules cover units;
`tests/` covers the binary as a black box over a temp home; both draw fixtures
from the shared test-support builders.

**Risk / migration notes:** Low risk — additive, no production change. Start with
the read-only flows (`init`, `list`, `show`) which need no external binaries,
then add mutation flows. Keep e2e tests fast and deterministic so they can stay in
the default `cargo test` run.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add a black-box CLI integration test over a temporary XRAT_PATH covering init, import, list table/JSON, show, delete, and purge; run it in the default cargo test gate.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added a network- and daemon-free black-box integration test under tests/ using the built xrat binary and temporary XRAT_PATH. It covers init, import, table and JSON listing, show config JSON, soft delete, deleted filter, purge, exit status, and stdout. Uses only existing tempfile plus serde_json dev dependency. Validation: CARGO_INCREMENTAL=0 just fmt ci passed, including the new integration test.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added a complete CLI config lifecycle smoke test against a temporary home; full workspace CI passed.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-25
title: Consolidate Export And Subscription Rendering Logic
status: Done
assignee:
  - '@codex'
created_date: '2026-07-05 14:43'
updated_date: '2026-10-02 19:27'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-3
dependencies: []
priority: medium
ordinal: 10
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/2-use-cases/10-export-subscription-rendering.md`

# Consolidate Export And Subscription Rendering Logic

## Finding

### [Priority: Medium] Consolidate export and subscription rendering logic

**Files involved:**

- `src/server/routes/json.rs`
- `src/server/routes/b64.rs`
- `src/server/routes/pac.rs`
- `src/app/commands/list.rs`
- `src/app/commands/proxy/pac.rs`

**Problem:** HTTP export routes and CLI formatting paths each assemble output
data directly. The `json` route and `b64` route duplicate filter construction
and top-limit validation. PAC endpoint extraction is embedded in the Axum route
module, while PAC rendering helpers and proxy command code live separately.

**Why this change is needed:** Export behavior is a product feature, not an
Axum-only concern. Duplicated filter and rendering decisions can cause `/json`,
`/b64`, CLI listing, and proxy/PAC support to drift.

**How to implement it:** Create export use-cases such as `ExportConfigsUseCase`
and `PacFileUseCase`. Move top-limit validation, raw-config selection, JSON
summary selection, active proxy endpoint extraction, and PAC rule assembly into
those use-cases. Keep HTTP handlers responsible for auth, query extraction,
headers, and body encoding. Keep CLI commands responsible for terminal
formatting.

**Positive effect on the codebase:** Exports become reusable by CLI, HTTP, TUI
sharing actions, and tests. Adding a new export filter or output route requires
less duplicated work.

**Suggested target architecture:** Application export services return strings or
read models; adapters handle transport details such as headers, base64 encoding,
and terminal output.

**Risk / migration notes:** Low to medium risk. Migrate `/json` and `/b64` first
because they already share query semantics, then extract PAC endpoint selection
separately.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Export filtering defaults, top validation, summary selection and newline subscription payload assembly live in application services shared by HTTP adapters.
- [x] #2 JSON and base64 exports preserve authentication, enabled/protocol filters, deleted exclusion, top limits and ordering, empty results, response schema and encoding.
- [x] #3 HTTP PAC and CLI PAC use shared active runtime endpoint extraction and PAC rule rendering; host checks, disabled behavior and DIRECT fallback remain unchanged.
- [x] #4 CLI listing continues to use the shared config selection service while retaining existing table, TSV and JSON output.
- [x] #5 Focused service and adapter regressions and just fmt ci pass; task notes and HANDOFF.md record verified results.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reuse ConfigService and the existing PAC renderer; move export defaults, validation and payload assembly into a small service module.
2. Centralize running-session endpoint extraction and PAC generation for HTTP and CLI adapters.
3. Verify selection and top-order parity, invalid limits, disabled/deleted and empty results, authentication precedence, active PAC output and existing CLI formats.
4. Run just fmt ci, review the diff, update SDK documentation and HANDOFF.md, then close with separate implementation and task-metadata commits.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Audit found CLI listing already uses ConfigService and PAC rule rendering already lives in proxy_pac. Reused these services rather than adding parallel use-case abstractions. Added ConfigExportRequest and summary/subscription methods, moved active endpoint extraction into proxy_pac, and exposed the export request through the SDK. HTTP auth, host checks, DTO mapping and base64 remain in adapters. No CLI flags, response schema, database schema or default behavior changes.

Validation: CARGO_INCREMENTAL=0 just fmt ci passed with 882 Rust tests and 3 Python version-check tests, strict workspace Clippy and formatting. Six new regressions cover shared export filtering/empty results, service-level top bounds, JSON/base64 top ordering and content type, authentication precedence, active-session PAC parity and invalid endpoints. Existing CLI list formatting and PAC policy tests also pass. Implementation committed as 2f8b137. No database or engine configuration changes; hosted CI and live browser PAC execution were not verified.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Consolidated export policy and subscription text in ConfigService using ConfigExportRequest, available through the SDK. HTTP routes retain transport and authentication responsibilities. Shared proxy_pac services now resolve active runtime endpoints and generate PAC for HTTP and CLI; CLI listing already used the shared config service and retains its output. Preserved filters, limits, ordering, deleted exclusion, response formats and PAC host policy. Full local workspace gate passed (882 Rust tests and 3 version tests). Updated SDK docs and handoff; next m-3 task is TASK-26.
<!-- SECTION:FINAL_SUMMARY:END -->

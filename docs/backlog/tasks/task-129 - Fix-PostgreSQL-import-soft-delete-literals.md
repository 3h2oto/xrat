---
id: TASK-129
title: Fix PostgreSQL soft-delete flag compatibility
status: Done
assignee:
  - '@codex'
created_date: '2026-10-02 18:29'
updated_date: '2026-10-02 18:58'
labels:
  - bug
  - database
dependencies: []
priority: high
ordinal: 111000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Real PostgreSQL verification fails during import with SQLSTATE 42804: is_deleted is an INTEGER column in released migration 0015, but shared upsert and PostgreSQL reconciliation SQL use boolean literals. Preserve the released schema and use integer flag literals consistently. Discovered while validating TASK-19; user approved a separate repair.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Initial import, subscription refresh removal, and re-add work with integer soft-delete flags on SQLite and PostgreSQL
- [x] #2 just test-postgres and just fmt ci pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Match released integer soft-delete storage: SQL literals 0/1 and i32 decoding for the INTEGER is_deleted column.
2. Extend existing backend reconciliation tests to cover remove and re-add with stable IDs/refs.
3. Run SQLite/PostgreSQL backend verification and just fmt ci; commit separately.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
After correcting boolean literals, the real backend exposed a second mismatch on the same released INTEGER column: map_config_row decoded is_deleted as i64/INT8. Decode this 0/1 flag as i32/INT4 for PostgreSQL and SQLite compatibility; leave BIGINT active/enabled fields and released migrations unchanged.

The backend harness also incorrectly expected delete_config to remove the row, although it now soft-deletes. Updated verification to assert retained/deleted/inactive state, then explicitly purge and verify absence. The real backend now reaches this final check after import, ref lookup, refresh, test history, runtime sessions and GeoIP checks.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Aligned soft-delete SQL with released INTEGER storage (0/1) and decoded the flag as i32. Shared backend regression covers removal/re-add with stable IDs/refs; harness asserts soft delete followed by explicit purge. CARGO_INCREMENTAL=0 just fmt ci passed (876 Rust tests, 3 version tests); just test-postgres passed against the local PostgreSQL database. No schema or CLI/API change; no additional user documentation needed.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

---
id: TASK-116
title: Investigate and reduce xrat list configs latency
status: Done
assignee:
  - '@codex'
created_date: '2026-09-12 19:13'
updated_date: '2026-09-30 09:17'
labels:
  - performance
  - cli
dependencies: []
priority: medium
ordinal: 97000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
User reports that xrat list configs feels slightly slow on installed v0.19.1. The supplied listing includes a subscription with 98 configs, saved probe results, GeoIP columns, and an active runtime. No timings or cause have been established. Measure command startup and listing work, identify the bottleneck, and improve responsiveness without dropping output data. TASK-24 covers a related listing refactor but is not a prerequisite for this focused performance task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Record reproducible baseline timings for cold and repeated runs on a representative dataset, including config and history sizes and GeoIP settings.
- [x] #2 Identify the dominant source of latency with profiling or stage timings; distinguish bootstrap, database queries, enrichment, and rendering before choosing a fix.
- [x] #3 Demonstrate reduced command latency with before/after measurements on the same fixture and environment.
- [x] #4 Preserve config filters, refs, active state, saved metrics, GeoIP fields, and table/TSV/JSON output; run focused correctness checks for the changed path.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Build a representative 98-config benchmark fixture with saved test history; measure cold/repeated CLI latency and stage timings; fix the dominant bottleneck; compare before/after on the same fixture and verify filters and all output formats.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Benchmark fixture: 98 VLESS configs, one subscription, 4,900 saved connection_tests (50/config), saved country/location/ASN, default GeoIP disabled. On the same host with debug stage timing, 9 repeated table runs: baseline cold 42.4 ms, warm median 41.5 ms; paths 1 ms, database bootstrap 15.5 ms, list query 14 ms, enrichment 0 ms, subscriptions 1 ms, render 1.5 ms. After checksum-repair batching and latest-test index: cold 41.1 ms (first migration), warm median 33.3 ms; database 7.5 ms, query 10.5 ms. SQLite EXPLAIN changed from per-config TEMP B-TREE ORDER BY to covering idx_connection_tests_config_latest. Table/TSV/JSON retained saved metrics and GeoIP fields; filter checks passed for active, enabled, deleted, all, and subscription.

Validation: CARGO_INCREMENTAL=0 just fmt ci passed (including workspace tests); cargo package -p xrat-db --list includes both new migration files. No PostgreSQL test URL was configured, so the PostgreSQL migration was compile/package checked but not exercised against a live server. First-run timing includes applying migration 23; warm medians measure repeated process invocations with OS caches available.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Reduced repeated xrat list configs latency on a 98-config/4,900-history fixture from 41.5 ms to 33.3 ms median by avoiding unchanged migration checksum writes and indexing latest-test lookup. Preserved list filters and output fields; full workspace CI passed.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

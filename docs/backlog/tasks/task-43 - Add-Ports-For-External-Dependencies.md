---
id: TASK-43
title: Add Ports For External Dependencies
status: In Progress
assignee: []
created_date: '2026-07-05 14:44'
updated_date: '2026-09-25 00:42'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 7
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Plan revised 2026-09-25.** Start with three ports only to avoid trait sprawl: Clock, Filesystem, ConfigRepository. Add HttpClient, ProcessSpawner, NetworkProbe, DaemonClient, RuntimeControl in later phases once the first seams prove value.

Gate: repository methods must be used from at least two call sites or need a test fake before becoming a port.

Prerequisite: TASK-17 typed port errors must land first, since a port cannot own HttpError/ProcessError while AppError #[from]s the concrete library.

Execution phase: P2 of refactor/r1-layering.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
P2 DONE (commit cb89237, branch refactor/r1-layering).

Added:
- src/app/ports/{clock,filesystem,config_repository}.rs — three ports only, per plan. Clock + SystemClock, Filesystem + RealFilesystem, ConfigRepository trait.
- src/app/read_models/{mod,config}.rs — ConfigSummary, LatestTestSummary, EndpointLocation mapped once from ConfigWithLatestTest.
- src/app/services/{mod,configs/*}.rs — AppServices with from_context() (no cli::Cli dependency) and with_ports(); ConfigService owns list/detail/resolve_id/subscriptions and shared enrich_endpoint_locations.
- src/app/services/configs/repository.rs — DatabaseConfigRepository adapter keeping db layer unaware of app ports.
- src/app/services/test_support.rs — FakeClock, InMemoryFilesystem, sample rows.

Verification: 858 tests pass (+5 new: fake clock, in-memory fs, read-model mapping, filter defaults). clippy -D warnings clean.

Next: P3 wires CLI/server/TUI to ConfigService and deletes duplicated mapping.
<!-- SECTION:NOTES:END -->

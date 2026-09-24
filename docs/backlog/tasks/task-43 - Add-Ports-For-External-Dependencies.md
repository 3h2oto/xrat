---
id: TASK-43
title: Add Ports For External Dependencies
status: To Do
assignee: []
created_date: '2026-07-05 14:44'
updated_date: '2026-09-24 23:34'
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

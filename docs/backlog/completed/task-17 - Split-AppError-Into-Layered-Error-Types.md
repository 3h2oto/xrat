---
id: TASK-17
title: Split AppError Into Layered Error Types
status: Done
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-09-29 10:26'
labels:
  - refactor
  - improvement
  - legacy-import
milestone: m-2
dependencies: []
priority: medium
ordinal: 23
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Confirmed 2026-09-25.** AppError has 26 variants with #[from] on std::io::Error, toml::de::Error, DbError, DecodeError, reqwest::Error, serde_json::Error, SecretError, tokio::task::JoinError (8 #[from] attributes).

Concrete layering inversions found: 9 files outside src/app reference crate::app::AppError:
- src/db/record/configs.rs (node_from_record returns app::AppError)
- src/support/geoip/backend/{mod,validation}.rs, src/support/geoip/mod.rs
- src/xray/process_mgmt/{process,signals}.rs (AppError::XraySpawn/Exited/StartupTimeout)
- src/singbox/process_mgmt.rs, src/singbox/version.rs (Singbox variants)
- src/server/mod.rs

Decision 2026-09-25: use typed port errors (each boundary owns HttpError, ProcessError, FsError, GeoipError). AppError keeps domain/application rules and composes typed layer errors via explicit From, never #[from] on third-party crates.

Execution phases: P1 in branch refactor/r1-layering.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
P1 DONE: AppError reduced to composed typed layer errors; xray/singbox/db use their own error types; crate::app references removed from engine/db layers.
<!-- SECTION:NOTES:END -->

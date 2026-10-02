# HANDOFF — Refactor: Use Cases progress

Last updated: 2026-10-02
Branch: `refactor/r1-layering`

## Outcome

Milestone m-2, **Refactor: Foundation**, is complete and archived through Backlog.
TASK-18, TASK-19, TASK-20, TASK-22 and TASK-23 are Done. Previously completed
TASK-21 was verified against the split module layout. TASK-125 remains In Progress
outside m-2.

## Delivered changes

- TASK-19: ConfigId, SubscriptionId and ConfigRef propagate through repositories,
  services, resolution, runtime/daemon boundaries, HTTP DTOs and TUI state. JSON
  IDs remain numeric and refs remain strings. The approved optional SQLx feature
  in xrat-model provides transparent database types.
- TASK-23: supervisor query fallbacks, metadata failures and dropped response
  receivers emit structured debug tracing while preserving best-effort behavior.
- TASK-18: removed the unnecessary log-parser unwrap and added production-only
  Clippy guards to daemon, runtime service and engine log parsing. The compiler
  audit of workspace libraries and binaries leaves three documented invariant
  expects: fallback DNS server, ensured TOML table and pinned version parsing.
- TASK-129: separately approved PostgreSQL repair uses 0/1 soft-delete SQL and
  i32 decoding for the released INTEGER column. Regression coverage verifies
  remove/re-add with stable IDs and refs, soft deletion and explicit purge.
  Released migrations were not changed.

## Verification

- `CARGO_INCREMENTAL=0 just fmt ci`: passed, including strict workspace Clippy,
  formatting, 876 Rust tests and three Python version-check tests.
- `CARGO_INCREMENTAL=0 just test-postgres`: passed against the real local
  PostgreSQL database; the backend test was executed rather than skipped.
- All 13 supervisor handler tests passed, including tracing and fallback tests.
- `cargo check -p xrat-model --features sqlx --locked`: passed independently.
- Production Clippy unwrap/expect audit: zero unwraps and three invariant expects.

These are local source/backend checks; hosted CI and deployed engine behavior
were not verified in this work.

## Focused implementation commits

- `9f6a30a`: typed config/subscription identifiers (Rust API breaking change).
- `e9943ec`: PostgreSQL soft-delete compatibility repair.
- `7e9b343`: supervisor best-effort failure tracing.
- `c6489af`: scoped production panic guards and log-parser regression coverage.

Backlog closure and this handoff are committed separately from source changes.

## TASK-25 complete

TASK-25 is Done, implemented in `2f8b137`.

- ConfigExportRequest centralizes enabled defaults, protocol filtering and top
  validation for summary and subscription exports. ConfigService returns summaries
  or newline-delimited raw links; HTTP adapters retain auth, DTOs and encoding.
- The SDK exposes ConfigExportRequest alongside ConfigService.
- Shared proxy_pac services resolve active endpoints and render PAC for both HTTP
  and CLI; PAC host validation remains in HTTP. CLI list selection already used
  ConfigService, so its table/TSV/JSON formatting was preserved.
- Six regression tests cover filter/empty selection, top bounds, JSON/base64 top
  ordering, auth precedence, active PAC parity and invalid runtime endpoints.
- `CARGO_INCREMENTAL=0 just fmt ci` passed: 882 Rust tests, three version tests,
  strict Clippy and formatting. Log: `/tmp/xrat-task25-ci.log`.

No HTTP schema, CLI flag, database schema or engine setting changed. Hosted CI
and live browser PAC execution were not verified. TASK-130 separately tracks
high-priority IPv6 preference with IPv4 fallback and remains To Do.

## Next work

Milestone m-3, **Refactor: Use Cases**, now has TASK-25 Done. Remaining work:

1. TASK-26: separate TUI data loading from direct I/O and process probing.
2. TASK-31: keep daemon supervisor handlers thin.

Begin with TASK-26 by auditing current TUI data loading and defining acceptance
criteria before implementing. Neither remaining task has started in this session.

## Local environment

The repo PostgreSQL Compose service was started for backend verification.
Incremental build artifacts were removed to recover disk space; use
`CARGO_INCREMENTAL=0` while space remains tight. Validation logs are available
in `/tmp/xrat-foundation-ci.log` and `/tmp/xrat-foundation-postgres.log`.

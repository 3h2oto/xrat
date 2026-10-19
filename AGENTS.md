# Repository Guidelines

## Project Structure & Module Organization

`src/` contains the thin executable/library entry points. Application and domain
code live in workspace crates. Keep responsibilities separated:

- `crates/xrat-app/src/cli/` defines Clap command trees, flags, and CLI parsing
  tests under `crates/xrat-app/src/cli/tests/`.
- `crates/xrat-app/src/app/` contains app runtime bootstrap and command
  handlers. Keep command logic under `crates/xrat-app/src/app/commands/` and
  keep lifecycle concerns in `crates/xrat-app/src/app/daemon/`,
  `crates/xrat-app/src/app/runtime_service/`, and
  `crates/xrat-engines/src/xray/process_mgmt/`.
- `crates/xrat-app/src/app/events.rs` records structured operational events.
  Event recording is best-effort and must not make the primary operation fail.
- `crates/xrat-app/src/app/context/`, `crates/xrat-app/src/app/config/`, and
  `crates/xrat-app/src/app/paths/` contain application bootstrap, path
  resolution, and command-specific runtime settings.
- `crates/xrat-config/src/` contains import, parsing, normalization, and
  protocol link support. Keep external input parsing separate from runtime
  config generation.
- `crates/xrat-engines/src/xray/` contains Xray parsing, generated runtime
  config builders, and process management.
- `crates/xrat-engines/src/singbox/` contains sing-box parsing/translation
  support.
- `crates/xrat-prober/src/` contains connection test runners for TCP, ICMP,
  download, upload, and real-delay flows.
- `crates/xrat-db/src/` contains database wiring, schema helpers, records, and
  repositories, split across `crates/xrat-db/src/database/`,
  `crates/xrat-db/src/record/`, and `crates/xrat-db/src/repository/`. Keep event
  persistence in the existing `events` database/repository modules.
- `crates/xrat-model/src/` contains shared domain types.
- `crates/xrat-app/src/server/` contains HTTP API routes, auth, response, and
  server state.
- `crates/xrat-app/src/tui/` contains terminal UI state, views, keymaps, and
  data adapters.
- `crates/xrat-support/src/` contains small shared helpers for decode, GeoIP,
  cancellation, network, time, and URL handling.
- `crates/xrat-web/` contains shared browser-safe API contracts and the Dioxus
  frontend. Follow its local `AGENTS.md` for page organization and UI
  conventions.
- `crates/xrat-sdk/` exposes the curated public SDK; do not route internal
  application code through it merely to share an implementation.
- `crates/xrat-db/migrations/` holds ordered SQL migrations. The root
  `migrations` symlink supports repository commands and documentation.
- `docs/src/` holds user-facing documentation.
- `crates/xrat-app/templates/` holds the systemd, launchd, and rc.d service
  templates embedded by `xrat daemon install`; `packaging/desktop/` holds
  desktop entry and icon packaging assets.
- `install.sh` installs release archives from GitHub and runs optional first-run
  setup.
- `.github/workflows/` contains CI and release automation, including musl
  release builds, generated man pages/completions, GitHub releases, Docker image
  publishing, and crates.io publishing.
- `Dockerfile` builds the container image published to GitHub Container
  Registry.
- `testdata/` holds local fixtures such as GeoIP data.

## Build, Test, and Development Commands

- Prefer common commands from `Justfile` over manually spelling out equivalent
  `cargo`, formatting, test, Docker, docs, or packaging commands.
- `cargo build` — compile the project.
- `cargo fmt` — format Rust code.
- `cargo clippy --all-targets -- -D warnings` — run the same lint gate used by
  CI.
- `cargo test -q --locked` — run the test suite quietly with the locked
  dependency graph.
- `cargo run -- <command>` — run the CLI locally, for example:
  - `cargo run --` (defaults to the TUI)
  - `cargo run -- init`
  - `cargo run -- import <input>`
  - `cargo run -- list`
  - `cargo run -- show config <config-id>`
  - `cargo run -- show subscription <subscription-id>`
  - `cargo run -- parse <config-id>`
  - `cargo run -- test <config-id>`
  - `cargo run -- scan`
  - `cargo run -- connect <config-id>`
  - `cargo run -- disconnect`
  - `cargo run -- logs`
  - `cargo run -- serve`
  - `cargo run -- tui`
  - `cargo run -- status`
  - `cargo run -- delete config <config-id>`
  - `cargo run -- delete subscription <subscription-id>`
  - `cargo run -- purge`
  - `cargo run -- daemon install --start`
  - `cargo run -- upgrade`
  - `cargo run -- version`
  - `cargo run -- geoip status`
  - `cargo run -- manpage --output <dir>`
  - `cargo run -- completions <shell>`

Run `just fmt ci` before committing.

## Coding Style & Naming Conventions

Use standard Rust formatting via `cargo fmt` (4-space indentation, rustfmt
defaults). Prefer small modules over large files, and split by capability when
files begin mixing CLI parsing, command orchestration, and domain logic.

Naming:

- files/modules: `snake_case`
- functions: `snake_case`
- structs/enums: `PascalCase`
- CLI flags: long, explicit names such as `--database`, `--include-geoip`, and
  `--format`

Avoid one-letter variable names. Do not add code comments unless they are truly
necessary to explain non-obvious intent or constraints. A short top-of-file
comment in `mod.rs` is acceptable when it clarifies what the module is
responsible for.

## Testing Guidelines

Use Rust’s built-in test framework with `#[test]` and `#[tokio::test]`.

- Keep tests close to the code they validate.
- Keep `mod.rs` files declarative: module declarations, imports, and re-exports
  belong there; put implementation in named sibling files.
- Move large inline test modules into a nearby `tests.rs` (typically around 80
  lines or more). Small inline tests may stay with the implementation.
- Prefer focused unit tests for parser/config normalization, CLI parsing, DB
  repositories, and runtime lifecycle transitions.
- Add regression tests when fixing parsing, dedup, scanner, or runtime-session
  edge cases.
- For CLI changes, add or update parser tests in
  `crates/xrat-app/src/cli/tests/` when behavior can be validated without
  starting external services.
- For repository changes, prefer database tests that exercise both SQLite and
  Postgres paths when the existing helpers make that practical.
- Name tests descriptively, e.g. `parses_list_config_filters`.

## Commit & Pull Request Guidelines

Follow conventional commit style seen in history:

- `feat: add scanner command with cf_scan persistence`
- `feat: complete managed runtime service`
- `refactor: split modules into subfiles`
- `test: expand runtime lifecycle coverage`

Keep commits focused and descriptive. PRs should summarize behavior changes,
mention schema or CLI changes, and include example commands/output when
relevant.

## Release Guidelines

Releases are driven by `.github/workflows/release.yml` and run on pushed tags
matching `v*`.

- Before preparing a release commit, run `just fmt ci`.
- Run `just set-version <version>` to update `[workspace.package]`, internal
  dependency requirements, and `Cargo.lock` together; every crate inherits the
  workspace version and the release workflow rejects inconsistent versions or
  tags whose version does not match the tag without the leading `v`.
- Use an annotated or signed version tag such as `v0.3.0`, matching the
  workspace version `0.3.0`.
- Do not edit released migrations. Add a new ordered migration for database
  changes that ship after a release.
- Confirm release-facing assets still work when touched: `install.sh`,
  `Dockerfile`, `.github/workflows/release.yml`, packaging files, generated man
  pages, completions, and user docs.
- The release workflow builds Linux musl archives, bundles man pages,
  completions, and desktop assets, creates `SHASUMS256.txt`, publishes the
  GitHub release, publishes Docker images to GHCR, and publishes the workspace
  crates to crates.io in dependency order (`xrat-model`, `xrat-support`,
  `xrat-config`, `xrat-engines`, `xrat-db`, `xrat-prober`, `xrat-app`,
  `xrat-sdk`, then `xrat`), skipping versions that already exist.
- Prepare or inspect release notes with `gh` when publishing or validating a
  release, keeping notes focused on user-visible changes, fixes, packaging
  changes, and upgrade notes.
- Release notes come from `.github/RELEASE_NOTE.md` when that file is present:
  the workflow passes it to `gh release create --notes-file`, and its Markdown
  becomes the release body verbatim (replacing the auto-generated commit list).
  Update it before tagging; if it is absent the workflow falls back to
  `--generate-notes`.
- If release automation changes, update docs under `docs/src/` and prefer a
  focused commit separate from feature work.

## Architecture Notes

Keep `src/main.rs` thin and route behavior through `crates/xrat-app/src/cli/`
and `crates/xrat-app/src/app/`.

For new CLI behavior, usually add:

- a new or extended command file in `crates/xrat-app/src/cli/`
- a matching handler under `crates/xrat-app/src/app/commands/`
- repository/model updates in `crates/xrat-db/src/` when persistence is required
- documentation under `docs/src/02-cli/` or the relevant feature page when user
  behavior changes
- packaging or installer updates when behavior changes generated service
  templates, Docker images, desktop assets, release extras, self-upgrade, or
  first-run installation flow

Design constraints from current implementation direction:

- Do not persist full root-level Xray JSON in the database; generate runtime
  config on demand from stored normalized data.
- Do not reintroduce config selection state. The old `configs.is_selected`
  column and `select` command were removed; use enabled/deleted config state and
  persisted runtime session state instead.
- Keep managed runtime lifecycle behavior explicit and observable (start/status/
  stop flow, persisted runtime session state).
- Record daemon, runtime, rotation, health, and test events through
  `crates/xrat-app/src/app/events.rs` when behavior should appear in
  `xrat logs`. These events are diagnostic records, so failures to write them
  should be logged and swallowed.
- Keep parser and runtime-generation concerns decoupled so parse/test/scan flows
  can evolve without rewriting CLI glue.
- Keep CLI parsing, command orchestration, persistence, and external process
  control in separate layers. Avoid command handlers that directly parse raw
  subscription formats or manually assemble database rows when reusable services
  already exist.
- Reuse shared CLI output helpers in
  `crates/xrat-app/src/app/commands/output.rs` and existing `--format`
  conventions (`table`, `tsv`, `json`, plus `csv` where already used) instead of
  adding ad hoc terminal formatting.
- Prefer typed domain records and repository methods over passing raw JSON or
  loosely structured maps between layers.

## Consistency Across CLI, TUI, and Web

CLI commands, TUI actions, and Axum/Web actions are adapters to the same XRAT
application. Equivalent operations must have the same domain behavior, even when
their presentation, authentication, or interaction model differs.

- Before changing an operation, identify its existing CLI, TUI, and HTTP entry
  points and the shared service they use. Check callers and existing tests; do
  not implement a second copy in the surface being edited.
- For a new capability, assess the relevant CLI, TUI, and Web entry points.
  Expose supported operations through the shared service and make unsupported
  capabilities or permission restrictions explicit. Do not silently leave an
  existing equivalent interface with different behavior.
- Put reusable orchestration in `crates/xrat-app/src/app/services/` or the
  existing capability module. Keep parsing/normalization in `xrat-config`,
  engine translation in `xrat-engines`, probes in `xrat-prober`, and persistence
  in `xrat-db`. Extend an existing typed service before adding a parallel path.
- Adapters may parse arguments/forms, authorize requests, map typed requests and
  results, schedule work, and format output. They must not independently own
  import/dedup rules, config lifecycle transitions, runtime generation,
  connection/process management, or test execution policy.
- Preserve semantics for identifiers/references, defaults, filter matching,
  enabled/deleted state, ordering/limits, import modes, exports, probe results,
  runtime status, cancellation, and errors. An intentional difference must be
  explicit in the contract, documented, and covered by a focused check.
- Apply filters before pagination. Copy/export/subscription operations for a
  filtered view must include all matching records and respect every filter;
  selected-row operations use their explicit IDs. Reuse the shared predicates
  and ordering instead of reproducing them in a route or component.
- Runtime/daemon status comes from the shared runtime services and persisted
  session state. Keep active connection state separate from transient table
  selection. Do not infer success from a UI flag or a queued job.
- Keep side effects and operational events consistent regardless of entry point.
  Preserve transaction boundaries, cancellation, error propagation, and
  idempotence/retry behavior when extracting or extending shared services.
- Authorization belongs on the server as well as in UI capabilities. Shared
  domain behavior does not grant remote clients local CLI privileges. Preserve
  identity, role, CSRF, ownership, and credential-scope boundaries.
- Browser-safe DTOs and pure config matching currently live in `xrat-web`'s
  library modules and must compile without its `ui` feature. Do not put Dioxus,
  browser APIs, database handles, or server-only secrets into that shared layer.
  Keep public SDK compatibility in mind when changing shared domain contracts.

For changes to shared behavior, test the rule at its owner and exercise the
relevant adapters with the same fixture/expected result. Include negative and
edge cases appropriate to the change, such as empty matches, deleted/disabled
records, repeated requests, failed/cancelled jobs, and stale runtime state. Use
adapter contract tests to catch request/default/result mapping drift; avoid
copying the same domain algorithm or its entire test suite per frontend. For a
structural move, compile all affected consumers and run existing focused tests.
Do not claim parity from compilation alone: state which CLI/TUI/HTTP paths were
exercised and which remain unverified.

## Agent Behavioral Guidelines

Behavioral guidelines to reduce common LLM coding mistakes. Merge with
project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial
tasks, use judgment.

### Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them instead of picking silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop, name what is confusing, and ask.

### Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- Do not add features beyond what was asked.
- Do not add abstractions for single-use code.
- Do not add flexibility or configurability that was not requested.
- Do not add error handling for impossible scenarios.
- If a change grows far beyond the requested scope, pause and simplify.

Ask: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Do not improve adjacent code, comments, or formatting unless required.
- Do not refactor things that are not broken.
- Match existing style, even if you would do it differently.
- If you notice unrelated dead code, mention it instead of deleting it.

When changes create orphans:

- Remove imports, variables, functions, and tests that your changes made unused.
- Do not remove pre-existing dead code unless asked.

Every changed line should trace directly to the user's request.

### Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" means write tests for invalid inputs, then make them pass.
- "Fix the bug" means write or identify a reproduction, then make it pass.
- "Refactor X" means preserve behavior and run relevant tests before finishing.

For multi-step tasks, state a brief plan:

```text
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria such as "make
it work" require clarification.

These guidelines are working if diffs contain fewer unnecessary changes, fewer
rewrites are needed due to overcomplication, and clarifying questions come
before implementation rather than after mistakes.

## GitHub Project Tracking

Track tasks in https://github.com/mhyrzt/xrat/issues and group related work with
GitHub milestones. Search existing issues before creating a task; use
`gh issue list`, `gh issue view`, and `gh issue edit` for the workflow.

For substantial work, record scope, acceptance criteria, implementation notes,
validation results, and a final summary in the issue. Use labels for capability
and priority, milestones for delivery groups, and sub-issues for parent/child
work. Link dependencies explicitly. Close completed tasks after recording
verification; close abandoned tasks as not planned. Small questions and
mechanical edits do not require an issue.

Historical `TASK-*`, `DRAFT-*`, and `doc-*` identifiers are retained in migrated
issue titles. Original records and metadata are preserved in each issue or
milestone description. Search by identifier when following an older reference.
Use GitHub as the source of truth for task status and planning.

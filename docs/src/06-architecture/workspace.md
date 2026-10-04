# Cargo Workspace And Crate Boundaries

XRAT ships as a single installed `xrat` binary, built from a Cargo workspace so
that reusable proxy-management logic is separated into focused library crates
without changing user-visible behavior.

## Layout

```text
Cargo.toml              # workspace root; also the `xrat` binary package
crates/
  xrat-model/           # domain types (Node, Protocol, NodeDedupKey)
  xrat-support/         # decode, GeoIP lookups, network, platform, time, url
  xrat-engines/         # Xray + sing-box parsing and runtime config generation
  xrat-config/          # subscription import and protocol link parsing
  xrat-db/              # connection, records, repositories, migrations wiring
  xrat-prober/          # TCP/ICMP/download/upload/real-delay probing
  xrat-app/             # application services, CLI, TUI, HTTP server
  xrat-sdk/             # curated public facade for embedding
src/                    # thin main.rs + lib.rs compatibility re-exports
```

The workspace uses `resolver = "3"`. The root package re-exports the extracted
crates directly from `src/lib.rs` so existing `crate::<module>::...` paths
continue to resolve, and it re-exports the frontends from `xrat-app`
(`pub use xrat_app::{app, cli, server, tui};`).

## Dependency direction

```text
xrat-sdk      -> xrat-config + xrat-engines + xrat-model + xrat-prober
                 + optional xrat-app (services feature)
xrat-app      -> xrat-config + xrat-db + xrat-engines + xrat-model
                 + xrat-prober + xrat-support
xrat-db       -> xrat-config + xrat-model + xrat-support
xrat-config   -> xrat-model + xrat-support
xrat-prober   -> xrat-engines + xrat-model + xrat-support
xrat-engines  -> xrat-config + xrat-model + xrat-support
xrat-support  -> (external dependencies only)
xrat-model    -> (leaf)
```

`xrat-app` (app + CLI + TUI + HTTP) is one crate because those layers currently
call into each other directly — in particular `AppContext::build` takes
`cli::Cli` and frontend callbacks re-enter app handlers. Splitting them into
separate `xrat-engine` / `xrat-cli` / `xrat-tui` / `xrat-http` crates requires
inverting those edges first and is deferred.

## Gate

Formatting, clippy, and tests run workspace-wide:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -q --locked --workspace
```

The `Justfile` (`ci`, `test`, `lint`, `fmt-rust-check`) and
`.github/workflows/ci.yml` use these commands. The test contract is preserved
across crates: the same total test count passes as before the split.

## Publishing

All crates share one version, single-sourced from `[workspace.package]` in the
root `Cargo.toml`, and internal dependencies are declared once in
`[workspace.dependencies]` with both a `path` and a `version`, so publishing
never fails on a missing version requirement.

The release workflow publishes to crates.io in dependency order, skipping any
version that already exists, so a partially completed release can be re-run:

```text
xrat-model -> xrat-support -> xrat-config -> xrat-engines -> xrat-db
-> xrat-prober -> xrat-app -> xrat-sdk -> xrat
```

The service templates embedded by `xrat daemon install` live in
`crates/xrat-app/templates/` (not the repository-root `packaging/`) so that
`xrat-app` remains a self-contained, publishable package.

## Residual work

- Split `xrat-app` into interface-neutral `xrat-engine` plus thin `xrat-cli` /
  `xrat-tui` / `xrat-http` adapters. Blocker: `AppContext::build` depends on
  `cli::Cli`, and frontends re-enter app handlers.
- Trim now-unused dependencies from the root `xrat` package (the app, TUI, HTTP,
  and database stacks moved into `xrat-app`).
- Complete SDK-owned stateful initialization, saved-config services, and managed
  runtime APIs. The default stateless SDK already exposes parsing, Xray/sing-box
  generation and probing without application initialization; existing service
  exports are experimental and require the `services` feature.

## Release versions and embedded assets

Run `just set-version 0.21.0` when preparing a release. This updates the
workspace package version and internal dependency requirements together and
refreshes `Cargo.lock`. CI and release validation reject mismatched versions.

Migrations live in `crates/xrat-db/migrations/` so published database crates
embed their own SQL files. The root `migrations` symlink preserves existing
development commands. SQL contents and migration checksums are unchanged.

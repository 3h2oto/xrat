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
src/                    # thin binary + re-export shims for the root package
```

The workspace uses `resolver = "3"`. The root package re-exports the extracted
crates (`src/model.rs`, `src/support.rs`, and so on) so existing
`crate::<module>::...` paths continue to resolve, and it re-exports the
frontends from `xrat-app` (`pub use xrat_app::{app, cli, server, tui};`).

## Dependency direction

```text
xrat-sdk      -> xrat-app, xrat-config, xrat-model, xrat-prober
xrat-app      -> xrat-config + xrat-db + xrat-engines + xrat-model
                 + xrat-prober + xrat-support
xrat-db       -> xrat-config + xrat-model + xrat-support
xrat-config   -> xrat-engines + xrat-model + xrat-support
xrat-prober   -> xrat-engines + xrat-model
xrat-engines  -> xrat-model + xrat-support
xrat-support  -> xrat-model
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

## Residual work

- Split `xrat-app` into interface-neutral `xrat-engine` plus thin
  `xrat-cli` / `xrat-tui` / `xrat-http` adapters. Blocker: `AppContext::build`
  depends on `cli::Cli`, and frontends re-enter app handlers.
- Trim now-unused dependencies from the root `xrat` package (the app, TUI, HTTP,
  and database stacks moved into `xrat-app`).
- Broaden the `xrat-sdk` facade and add a non-CLI `AppContext` constructor so
  the SDK does not require CLI arguments to build.

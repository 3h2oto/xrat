# Cargo Workspace And Crate Boundaries

XRAT ships as a single installed `xrat` binary, built from a Cargo workspace so
that reusable proxy-management logic can be extracted into focused library
crates over time without changing user-visible behavior.

## Current layout

```text
Cargo.toml            # workspace root; also the `xrat` package
crates/
  xrat-model/         # shared domain types (leaf crate)
src/                  # the `xrat` package: app, cli, tui, server, engines, db
```

- The workspace is declared in the root `Cargo.toml` (`[workspace]`) with
  `resolver = "3"`.
- `xrat-model` holds the shared domain types (`Node`, `Protocol`,
  `NodeDedupKey`). The root package depends on it, and `src/model.rs` re-exports
  it so existing `crate::model::...` paths keep working.
- CI and the `Justfile` run formatting, clippy, and tests with `--workspace` so
  every member is covered by the same gate. The root `cargo test` alone would
  only cover the root package.

## Extracted crates

| Crate | Contents | Depends on |
| --- | --- | --- |
| `xrat-model` | Domain types and dedup keys | `serde`, `serde_json` |

## Extraction order

The remaining split follows the staged plan in the SDK/workspace task:

```text
xrat-bin -> xrat-cli + xrat-tui + xrat-http
xrat-cli/xrat-tui/xrat-http -> xrat-engine
xrat-sdk -> xrat-engine
xrat-engine -> xrat-runtime + xrat-config + xrat-db + xrat-prober + xrat-model
xrat-runtime -> xrat-config + xrat-db + xrat-model
```

Tier 1 (stateless, no `AppContext`/database coupling) is extracted first:
`model`, `support`, `config`, `prober`. Tier 2 (`app`) and Tier 3 (`cli`,
`tui`, `server`) follow.

## Current dependency cycles

A full crate split is currently blocked by cycles in the existing module graph.
The relevant edges are:

| Edge | Cause | Blocker for |
| --- | --- | --- |
| `support -> app` | `support::geoip::backend` uses `app::config::{AppConfig, GeoIpBackend}`, `app::context::RuntimePaths`, `app::paths::mmdb` | extracting `xrat-support` |
| `config -> xray`, `xray -> config` | `config::import` parses `xray::parsing` types; `xray` generator tests use `config::parse_link` (test-only) | extracting `xrat-config` / `xrat-runtime` |
| `prober -> app` | probing orchestration reaches into `app` state | extracting `xrat-prober` |
| `app <-> cli` | `app::context::AppContext::build(args: &cli::Cli)` and handlers take CLI types while CLI callbacks call app | splitting frontends from `xrat-engine` |
| `app <-> tui`, `app <-> server` | frontends and app call each other directly | splitting adapters from `xrat-engine` |

## Unblocking work

1. Move the app-wiring `support::geoip::backend` (which resolves mmdb paths from
   `AppConfig`/`RuntimePaths`) up into `app`, leaving `support::geoip` as pure
   lookup primitives. This removes the `support -> app` edge.
2. Give `AppContext` a programmatic constructor that does not take `cli::Cli`,
   keeping `AppContext::build(args: &cli::Cli)` as an adapter. This removes the
   `app <-> cli` construction cycle.
3. Move `xray::parsing` below the engine/generation layer (or into `xrat-config`)
   so `config` and `xray` stop depending on each other.
4. Extract `support`, `engines` (`xray` + `singbox`), `config`, `db`, `prober`,
   then `engine`, then the adapters and `sdk`.

Until those edges are inverted, only genuinely leaf crates are extracted.

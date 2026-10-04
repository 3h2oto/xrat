# Consuming The SDK

`xrat-sdk` is the curated facade for embedding xrat's proxy-management logic in
another Rust program. It re-exports a stable subset of the workspace and
deliberately hides repository rows, CLI structs, HTTP DTOs, and process
internals.

## What it exposes

- `xrat_sdk::model` — `Node`, `Protocol`, `NodeDedupKey`
- `xrat_sdk::config` — `parse_link`, `parse_text`, `parse_import`, import types
- `xrat_sdk::prober` — probe result types
- `xrat_sdk::services` — `AppServices`, `ConfigService`, `ConfigExportRequest`, lifecycle outcomes,
  read models (`ConfigSummary`, `ConfigDetail`), and `TestRunRequest`

```rust
use xrat_sdk::config::parse_link;

let node = parse_link("vless://uuid@example.com:443#edge")?
    .expect("a share link");
println!("{}:{}", node.address, node.port);
```

## Export services

`ConfigService::export_summaries` returns interface-neutral config summaries.
`ConfigService::export_subscription` returns raw config links joined by newlines,
without a trailing newline or transport encoding. Both accept
`ConfigExportRequest`: enabled-only defaults to true, `enabled = Some(false)`
includes disabled configs, `protocol` filters the protocol, and `top` selects
1–200 configs ordered by measured real delay. Deleted configs are excluded.
HTTP handlers retain authentication, response DTO mapping and base64 encoding;
CLI listing retains its own presentation over the shared config list service.

Within `xrat-app`, `app::services::proxy_pac::active_pac` combines running-session
endpoint selection with PAC rendering. HTTP and CLI PAC adapters call this same
function. PAC HTTP host validation remains in the HTTP adapter, and Shadowsocks
endpoints are excluded from PAC output.

## Dashboard loading boundaries

Within `xrat-app`, `app::services::dashboard::DashboardService` assembles a
`DashboardSnapshot` containing config and source data, runtime and daemon facts,
test results, history, logs, display addresses and pending GeoIP enrichment.
`TuiData::from(snapshot)` only converts these facts into view models. Startup
and refresh use the same loading service; log reloads use `DashboardLogs`.

GeoIP cache reads and progressive enrichment live in the dashboard service
modules. Fresh empty cache entries still suppress repeated lookups; background
work retains its concurrency, timeout and batch limits.

Engine versions are queried through `RuntimeEngineProbe`, with the production
process adapter enforcing a two-second timeout. `ReleaseService` uses an
injectable `ReleaseProvider`; the TUI and CLI upgrade share latest-release
fetching. TUI release failures are logged at debug level and remain nonfatal.
These internal services are available from `xrat-app`, rather than the curated
SDK facade.

## Adding the dependency

Once the workspace version is published:

```toml
[dependencies]
xrat-sdk = "0.20"
```

Before that version exists on crates.io, use a git or path dependency:

```toml
[dependencies]
xrat-sdk = { git = "https://github.com/mhyrzt/xrat" }
# or, from a checkout:
# xrat-sdk = { path = "path/to/xrat/crates/xrat-sdk" }
```

A git dependency resolves the in-repo path dependencies automatically, so no
extra registry setup is required.

## Compatibility

Use `just set-version <version>` to synchronize package and dependency versions.
The whole workspace shares one package version, inherited from
`[workspace.package]` in the root `Cargo.toml`. The SDK is pre-1.0: pin an exact
`=0.20.x` if you need bit-for-bit reproducibility, and expect the curated
surface (not the internal crates) to be the stable contract.

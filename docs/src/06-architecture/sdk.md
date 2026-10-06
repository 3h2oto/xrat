# Consuming The SDK

`xrat-sdk` is the public package for embedding stateless XRAT tools in another
Rust program. Add the published release without Git:

```bash
cargo add xrat-sdk
cargo add serde_json
```

## Parse a link and generate JSON

```rust
use xrat_sdk::{config::parse_link, engines::xray};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node = parse_link(
        "vless://11111111-1111-1111-1111-111111111111@example.com:443#edge",
    )?.ok_or("expected a node")?;
    let config = xray::generate_runtime_config(&node, 1080, Some(8080))?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}
```

Serializing `node` instead produces XRAT's normalized data representation,
including credentials and the original link. It is not runnable engine JSON.
Generating a runtime config does not start an engine or initialize a database.
The caller chooses listeners, DNS, routing, and engine-specific options.

For sing-box:

```rust
use xrat_sdk::engines::singbox::{
    SingboxInbound, generate_singbox_runtime_config,
};

let config = generate_singbox_runtime_config(
    &node,
    vec![SingboxInbound::socks("socks-in", "127.0.0.1", 1080, None)],
    None, // optional Clash API
    None, // optional routing
)?;
let json = serde_json::to_string_pretty(&config)?;
```

## Public modules

| Module             | Available capabilities                                                                                                       |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| `model`            | Serializable `Node`, `Protocol`, `NodeDedupKey`                                                                              |
| `config`           | `parse_link`, `parse_text`, `parse_import`, import types and errors                                                          |
| `engines::xray`    | Runtime/probe generators, config types, DNS/routing/tuning options and compatibility targets                                 |
| `engines::singbox` | Runtime/probe generators, config types, listener/DNS/routing/Clash API options                                               |
| `prober`           | `tcp_check`, `icmp_ping`, `real_delay_check`, `download_speed_check`, `upload_speed_check`, engine selector and result types |
| `services`         | Experimental stateful exports, only with the `services` feature                                                              |

The engine modules expose the supporting types needed by their public signatures
and config fields. Consumers do not need direct dependencies on internal XRAT
crates. Parsing and generation preserve the CLI's underlying implementations and
unsupported-setting errors. The SDK has no separate V2Ray target yet.

## Execute probes

Async probes require a Tokio runtime:

```rust
use std::time::Duration;
use xrat_sdk::prober::tcp_check;

let result = tcp_check("127.0.0.1", 1080, Duration::from_secs(1)).await;
if !result.success {
    eprintln!("{:?}: {:?}", result.failure_kind, result.failure_reason);
}
```

TCP probes connect directly to the supplied endpoint. ICMP probes use the
platform's `ping` executable and its permissions. Real-delay/download/upload
probes require a proxy node, caller-selected test URL, engine selector, explicit
binary path, and startup/request timeouts. They launch a temporary local SOCKS
proxy and clean it up on completion or cancellation. They do not install
engines, set up global logging, or persist results. The existing
`XrayGenOptions` argument only tunes Xray; sing-box probes use the default
sing-box probe generator.

## Runnable examples and checks

```bash
cargo run -p xrat-sdk --example parse
cargo run -p xrat-sdk --example node_json
cargo run -p xrat-sdk --example xray_json
cargo run -p xrat-sdk --example singbox_json
cargo run -p xrat-sdk --example tcp
cargo run -p xrat-sdk --example real_delay -- xray /path/to/xray 'http://127.0.0.1:8080' http://example.com/
just sdk-check
just sdk-native /path/to/xray /path/to/sing-box
just sdk-registry 0.21.0
```

The TCP example creates its own local listener. Real-delay probing needs a real
proxy and reachable test URL. Choose what credential-bearing data your app logs.

The standalone consumer under `testdata/sdk-consumer` has its own workspace and
depends directly only on the SDK and ordinary third-party crates. CI checks
normal and `services` builds, public examples, rustdoc, dependency exclusions,
and representative native engine validation with local probe lifecycle fixtures.

## Experimental services and deferred work

```toml
[dependencies]
xrat-sdk = { version = "0.21", features = ["services"] }
```

This feature preserves the previous `xrat_sdk::services` imports and pulls in
`xrat-app` with its database/UI dependencies. The default SDK does not include
those application layers or the engine traffic-stats RPC dependencies.
Application builds explicitly retain stats support. These services still need
internal initialization and are experimental; they are not a complete SDK-only
stateful contract.

Milestone **SDK: Public embedding and crates.io delivery** tracks deferred
stateful initialization ([TASK-153](https://github.com/mhyrzt/xrat/issues/176)),
saved-config/subscription services
([TASK-154](https://github.com/mhyrzt/xrat/issues/177)), managed runtime
embedding ([TASK-155](https://github.com/mhyrzt/xrat/issues/178)), and
separately validated V2Ray generation/probing
([TASK-156](https://github.com/mhyrzt/xrat/issues/179)). No implementation of
those deferred features is included in the stateless release.

## Export services

`ConfigService::export_summaries` returns interface-neutral config summaries.
`ConfigService::export_subscription` returns raw config links joined by
newlines, without a trailing newline or transport encoding. Both accept
`ConfigExportRequest`: enabled-only defaults to true, `enabled = Some(false)`
includes disabled configs, `protocol` filters the protocol, and `top` selects
1–200 configs ordered by measured real delay. Deleted configs are excluded. HTTP
handlers retain authentication, response DTO mapping and base64 encoding; CLI
listing retains its own presentation over the shared config list service.

Within `xrat-app`, `app::services::proxy_pac::active_pac` combines
running-session endpoint selection with PAC rendering. HTTP and CLI PAC adapters
call this same function. PAC HTTP host validation remains in the HTTP adapter,
and Shadowsocks endpoints are excluded from PAC output.

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

## Releases and development dependencies

The SDK shares XRAT's workspace version and tagged release workflow. The 0.21
series exposes the stateless API; commit your application's lockfile to make
dependency resolution reproducible. Since the SDK is pre-1.0, a new minor series
may change the public API.

Git and local checkouts remain optional for development:

```toml
[dependencies]
xrat-sdk = { git = "https://github.com/mhyrzt/xrat" }
# or:
# xrat-sdk = { path = "path/to/xrat/crates/xrat-sdk" }
```

Xray output follows its explicit compatibility target; sing-box output follows
the existing 1.13 schema. Representative native checks are pinned to Xray
26.3.27 and sing-box 1.13.21; this is not exhaustive conformance for every newer
version or all protocol/transport combinations.

Use `just set-version <version>` to synchronize the workspace and publish in the
existing dependency order. The release workflow verifies a fresh registry-only
SDK consumer after publication, including normal `cargo add xrat-sdk`.

# XRAT SDK

Embed XRAT's stateless proxy tools in a Rust application:

```bash
cargo add xrat-sdk
cargo add serde_json
```

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

## Available APIs

- `model`: normalized `Node`, `Protocol`, and deduplication keys. Serializing a
  node produces XRAT's data representation, not runnable engine JSON.
- `config`: parse individual proxy links, subscription text, and imports.
- `engines::xray`: typed runtime/probe generators, listener settings, DNS,
  routing, tuning options, and compatibility targets.
- `engines::singbox`: typed runtime/probe generators, listeners, DNS, routing,
  and Clash API options.
- `prober`: TCP, ICMP, real delay, download, and upload checks and result types.

Default SDK builds also exclude the engine traffic-stats RPC stack; application
builds retain their existing stats support.

JSON generation has no database, process, or configuration-file side effects.
Unsupported settings return the existing generator errors. Xray and sing-box
have different schemas and capabilities; these modules do not promise arbitrary
cross-engine conversion. There is no separate V2Ray generation target yet.

Async probes require a Tokio runtime. TCP checks connect to the supplied
endpoint; ICMP uses the platform's `ping` command and its permissions.
Real-delay and speed checks require an explicitly supplied Xray or sing-box
binary, startup/request timeouts, and test URL. They launch a temporary local
SOCKS proxy and clean it up after completion or cancellation. They do not
download engines, initialize global logging, or persist results.
`XrayGenOptions` only tunes Xray probes; sing-box probes use their existing
default generator and ignore this argument.

## Experimental services

Stateful service exports require an explicit opt-in:

```toml
[dependencies]
xrat-sdk = { version = "0.21", features = ["services"] }
```

This preserves the existing `xrat_sdk::services` module but pulls in `xrat-app`
and its database/UI dependencies. SDK-only initialization, saved-config
services, and managed connect/status/disconnect APIs are deferred. These
experimental services are not a complete standalone embedding contract.

## Examples and compatibility

From a checkout, run:

```bash
cargo run -p xrat-sdk --example parse
cargo run -p xrat-sdk --example node_json
cargo run -p xrat-sdk --example xray_json
cargo run -p xrat-sdk --example singbox_json
cargo run -p xrat-sdk --example tcp
cargo run -p xrat-sdk --example real_delay -- xray /path/to/xray 'http://127.0.0.1:8080' http://example.com/
```

The TCP example creates its own loopback listener. Real-delay probing requires
an actual proxy and caller-selected test endpoint. Probe requests and normalized
JSON may contain proxy credentials; choose what your application logs or shares.

The SDK shares release versions with XRAT and is pre-1.0. Use the `0.21` series
for this API; commit your application's lockfile for reproducible resolution.
Xray compatibility is selected by `XrayGenOptions`; sing-box output uses the
existing 1.13 schema. Representative native checks are pinned to Xray 26.3.27
and sing-box 1.13.21, rather than claiming compatibility with every engine
version.

Git remains optional for unreleased development code:

```bash
cargo add xrat-sdk --git https://github.com/mhyrzt/xrat
```

See [API documentation](https://docs.rs/xrat-sdk) and
[XRAT documentation](https://mhyrzt.github.io/xrat/06-architecture/sdk.html).

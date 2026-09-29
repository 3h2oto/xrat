# Consuming The SDK

`xrat-sdk` is the curated facade for embedding xrat's proxy-management logic in
another Rust program. It re-exports a stable subset of the workspace and
deliberately hides repository rows, CLI structs, HTTP DTOs, and process
internals.

## What it exposes

- `xrat_sdk::model` — `Node`, `Protocol`, `NodeDedupKey`
- `xrat_sdk::config` — `parse_link`, `parse_text`, `parse_import`, import types
- `xrat_sdk::prober` — probe result types
- `xrat_sdk::services` — `AppServices`, `ConfigService`, lifecycle outcomes,
  read models (`ConfigSummary`, `ConfigDetail`), and `TestRunRequest`

```rust
use xrat_sdk::config::parse_link;

let node = parse_link("vless://uuid@example.com:443#edge")?
    .expect("a share link");
println!("{}:{}", node.address, node.port);
```

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

The whole workspace shares one version, single-sourced from
`[workspace.package]` in the root `Cargo.toml`. The SDK is pre-1.0: pin an exact
`=0.20.x` if you need bit-for-bit reproducibility, and expect the curated
surface (not the internal crates) to be the stable contract.

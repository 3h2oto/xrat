## xrat v0.21.0

This release introduces the published stateless `xrat-sdk` package and completes
the Cargo workspace refactor while preserving XRAT's CLI, TUI and managed
runtime workflows.

### Features

- **Embed XRAT from crates.io.** Use `cargo add xrat-sdk` for link/subscription
  parsing, normalized nodes, typed Xray/sing-box runtime and probe JSON
  generation, and TCP, ICMP, real-delay, download and upload probes. Developers
  do not need Git, internal crate dependencies, CLI arguments or a database for
  these stateless APIs.
- **Standalone examples and native checks.** Runnable examples cover parsing,
  normalized JSON, both engine formats, TCP and real-delay probing. CI verifies
  independent consumers and representative output against Xray 26.3.27 and
  sing-box 1.13.21, including local probe success/failure and process cleanup.
- **Reusable workspace crates.** Domain, parsing, engine, persistence, probing,
  application and SDK responsibilities now live in separate published crates,
  with shared versions and dependency-ordered releases.

### Fixes and refactoring

- Generated Xray Shadowsocks and SOCKS outbounds now use the native protocol
  names `shadowsocks` and `socks`, fixing configs previously rejected by Xray.
- PostgreSQL imports use integer soft-delete values compatible with the schema.
- Config lookup and lifecycle behavior is shared between CLI, HTTP and TUI
  adapters; TUI runtime operations use the daemon when available.
- Shared services handle exports, PAC rendering, dashboard loading, testing and
  daemon transitions. Process, network, HTTP and host dependencies use explicit
  ports, with shutdown registration retained across polling.
- Rust 1.99 strict lint checks pass without weakening the workspace lint gate.

### Upgrade notes

- No new database migration is introduced by this release.
- Existing SDK stateful service imports now require the `services` feature:
  `xrat-sdk = { version = "0.21", features = ["services"] }`. They remain
  experimental and include application/database/UI dependencies.
- Default SDK APIs are stateless. SDK-only database initialization, saved-config
  management, managed connect/status/disconnect and a separate V2Ray target are
  deferred; the CLI's existing V2Ray runtime behavior is unchanged.
- Probing requires a Tokio runtime and, for proxy probes, caller-supplied engine
  binaries, URLs and timeouts. The SDK does not install engines or persist
  results.
- Representative native checks do not claim exhaustive conformance for every
  protocol/transport combination or newer engine versions.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.20.0...v0.21.0

## xrat v0.21.1

This patch restores compatibility with share links containing legacy exporter
parameters that stricter Xray config validation previously rejected.

### Fixes

- WebSocket links accept `headerType=none` or an empty header type. Unsupported
  non-neutral header types still produce an explicit error.
- REALITY links accept validated `allowInsecure`/`insecure` boolean flags as
  legacy metadata. These flags do not disable REALITY authentication or emit TLS
  settings. Malformed values and conflicting aliases remain errors.
- Regression tests verify unchanged generated configs for accepted parameters,
  including both supported Xray compatibility targets for REALITY.
- Registry-only SDK verification now excludes the local test consumer itself
  when checking for internal path dependencies, fixing a false release failure.

### Upgrade notes

- No database migration or configuration changes are required.
- The fixes apply to CLI, TUI and SDK callers using Xray config generation.
- This release verifies config generation; it does not guarantee remote server
  availability or connectivity.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.21.0...v0.21.1

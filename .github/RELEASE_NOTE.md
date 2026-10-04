## xrat v0.21.2

This maintenance release includes the proxy-shell test fix merged after v0.21.1
and the latest Backlog organization updates.

### Maintenance

- Proxy-shell status tests use an explicit color setting so assertions do not
  depend on terminal color detection. Runtime output keeps its existing color
  behavior.
- Move completed Backlog tasks to the completed directory and update pending
  task ordering and status.

### Upgrade notes

- No database migration or configuration changes are required.
- The WebSocket and REALITY share-link compatibility fixes from v0.21.1 are
  included.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.21.1...v0.21.2

## xrat v0.21.2

This maintenance release includes the proxy-shell test fix merged after v0.21.1
and the migration of project planning to GitHub issues and milestones.

### Maintenance

- Proxy-shell status tests use an explicit color setting so assertions do not
  depend on terminal color detection. Runtime output keeps its existing color
  behavior.
- Track project work through GitHub issues and milestones, including completed
  and archived tasks with their original metadata and validation notes.

### Upgrade notes

- No database migration or configuration changes are required.
- The WebSocket and REALITY share-link compatibility fixes from v0.21.1 are
  included.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.21.1...v0.21.2

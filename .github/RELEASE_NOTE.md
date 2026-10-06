## xrat v0.22.1

This release makes the opt-in TUN capture introduced in v0.22.0 easier to
configure and fixes the documentation link in TUI help.

### TUN controls

- Use `xrat tun enable` and `xrat tun disable` to save the capture setting without
  editing TOML. Commands preserve comments and unrelated settings, validate
  changes, and are safe to repeat.
- Press **U** (Shift+U) in the TUI to toggle and save the same setting. Lowercase
  `u` still refreshes subscriptions; typing in search and modals is unaffected.
- Press **,** to open settings, select `runtime.tun`, and save with **Ctrl+S**
  to configure all TUN options.

### Fixes and documentation

- Restore the docs URL in TUI help by setting the application crate's homepage.
- Document CLI and TUI TUN controls and their effect on active connections.

### Upgrade notes

- These controls save configuration; they do not immediately switch an active
  connection or grant privileges. Restart a running daemon, then reconnect to
  apply either change. Disabling the setting alone does not stop an active TUN
  interface.
- File capabilities are lost when xrat or an engine is upgraded. Run
  `xrat tun setup` as needed, then restart the daemon before using TUN.
- TUN remains disabled by default. DNS interception and V2Ray TUN remain
  unsupported. No database migration is required.

### Contributors

Thanks to [@f02xygen](https://github.com/f02xygen) for the managed TUN foundation
in [PR #6](https://github.com/mhyrzt/xrat/pull/6), released in v0.22.0.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.22.0...v0.22.1

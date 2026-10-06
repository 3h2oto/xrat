## xrat v0.23.0

TUN controls now apply to the current connection, with checks before replacement
and rollback to the previous mode when startup or saving fails. This release also
fixes oversized GitHub prerelease lookups in the core installer.

### Live TUN controls

- `xrat tun enable` reconnects the current config with TUN capture;
  `xrat tun disable` keeps the same config connected in local proxy mode.
- Engine support, privileges, interface ownership, and native config validation
  are checked before stopping the current session. Failed replacements attempt
  to restore the previous mode and connection; configuration is saved only after
  a successful change. Repeating an already applied command avoids a restart.
- When disconnected, the choice is saved for the next connection.
- Press **U** (Shift+U) in the TUI for the same live toggle. Saving only
  `runtime.tun.enabled` in settings also applies immediately; save other settings
  separately when changing that flag.
- CLI requests go through the daemon. For a standalone TUI connection, press
  **U** in that TUI instead of taking over its runtime from the CLI.

### Clearer status and installer errors

- `xrat tun status` separates the saved setting, active owned interface, engine
  support, and privilege readiness. `xrat status` also reports TUN state.
- Enable and disable support `--json`; text output explains whether the mode is
  active or saved for the next connection and gives relevant repair commands.
- Correct the minimum Xray version for managed Linux TUN to **26.7.11**.
  Unsupported or unknown versions are rejected before replacing a connection.
- Prerelease installation requests smaller GitHub release pages and follows
  pagination when needed. Metadata downloads allow 30 seconds, and timeout
  errors suggest retrying or selecting an explicit version.

### Upgrade notes

- Normal TUN toggles no longer require setup or a daemon restart. Restart a
  running daemon once after upgrading from an older version to load live toggle
  support. First-time privileges may also require one owner restart.
- Upgrading xrat or the engine removes file capabilities. Run `xrat tun setup`
  again as needed; restart a daemon or standalone TUI that lacks effective
  privileges before enabling TUN.
- TUN remains opt-in and Linux-only. Xray and sing-box are supported; DNS
  interception and V2Ray TUN remain unsupported. No database migration is needed.
- Active TUN status verifies a running session and an owned kernel interface; it
  does not prove external traffic or DNS reachability. Regression tests cover
  injected lifecycle failures; live privileged traffic capture remains unverified.

### Contributors

Thanks to [@f02xygen](https://github.com/f02xygen) for the managed TUN foundation
in [PR #6](https://github.com/mhyrzt/xrat/pull/6), released in v0.22.0.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.22.1...v0.23.0

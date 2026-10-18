## xrat v0.30.3

Fix batch-test cancellation delays and TUN startup under systemd user services.

### Fixes

- Cancelling a batch test now aborts in-flight probes without waiting for
  network timeouts, and clears TUI row spinners immediately. Contributed by
  [@f02xygen](https://github.com/f02xygen) in
  [#250](https://github.com/mhyrzt/xrat/pull/250).
- The TUN systemd user-service override disables filesystem namespace isolation
  that prevented engine file capabilities from granting access to the host TUN
  device. Contributed by [@f02xygen](https://github.com/f02xygen) in
  [#249](https://github.com/mhyrzt/xrat/pull/249).

### Upgrade notes

- Existing systemd user-service installations using TUN should run
  `xrat tun setup` after upgrading, then restart the daemon with
  `systemctl --user restart xrat-daemon.service` to apply the updated override.
- The TUN override disables `ProtectSystem`, `ProtectHome`, and `PrivateTmp`,
  and clears `ReadWritePaths`. This reduces filesystem isolation for TUN-enabled
  daemons; normal filesystem permissions still apply. The default service
  without the TUN override retains its hardening.
- No CLI, configuration, or database schema changes.

### Contributors

Thanks to [@f02xygen](https://github.com/f02xygen) for both fixes in this
release.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.30.2...v0.30.3

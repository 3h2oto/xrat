## xrat v0.30.0

Import subscriptions that require custom HTTP headers, and keep Linux TUN
privileges ready across future upgrades.

### Custom subscription headers

Pass repeatable headers when importing a subscription:

```bash
xrat import https://example.com/sub.txt \
  --header 'X-HWID: device-id' \
  --header 'User-Agent: Happ'
```

- Headers are saved per subscription and reused by CLI, TUI, and automatic
  refreshes.
- Each successful URL import replaces the saved header set; importing without
  `--header` clears it. Failed HTTP imports preserve existing headers.
- Custom headers stay within the original scheme, host, and port during
  redirects and are not forwarded to nested subscription URLs.
- Header values are hidden from list/show output and marked sensitive on HTTP
  requests. They are stored unencrypted in the database.

### TUN upgrade automation

- Linux upgrades can restore TUN file capabilities when prior setup is detected
  or TUN is enabled. Core reinstalls also restore capabilities when TUN is enabled.
- After successful upgrade-time restoration, an active daemon is restarted to
  pick up the capabilities. A stale standalone daemon socket does not start an
  inactive daemon.
- Configure `[runtime.tun].auto_setup_on_upgrade` as `"auto"` (default), `"always"`,
  or `"never"`, and `restart_daemon_on_upgrade` as `true` or `false`.
- Use `xrat upgrade --tun-setup`, `--no-tun-setup`, or `--no-restart-daemon` to
  override automation for an individual upgrade.
- Capability restoration uses normal sudo authentication and may prompt for a
  password. XRAT does not install passwordless sudoers rules.

### Upgrade notes

- Adds database migration 0024 for subscription headers on SQLite and PostgreSQL;
  migrations run automatically.
- When upgrading from v0.29.0, run `xrat tun setup` and restart the daemon once if
  TUN capabilities need restoring. The new upgrade automation takes effect when
  the upgrade is launched by v0.30.0 or later.
- If you used the short-lived `tun setup --sudoers` helper from an unreleased
  source build, review and remove `/etc/sudoers.d/xrat-tun` with
  `sudo rm /etc/sudoers.d/xrat-tun`. The helper has been removed.

### Contributors

Thank you to [@vaniog](https://github.com/vaniog) for custom subscription headers
([#225](https://github.com/mhyrzt/xrat/pull/225)), and
[@f02xygen](https://github.com/f02xygen) for TUN upgrade automation
([#224](https://github.com/mhyrzt/xrat/pull/224)). Follow-up privilege and daemon
hardening shipped in [#227](https://github.com/mhyrzt/xrat/pull/227).

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.29.0...v0.30.0

# tun

Apply managed TUN capture to the current connection and inspect its readiness.

```bash
xrat tun <subcommand>
```

TUN capture creates a network interface and system routes, so the engine binary
needs `CAP_NET_ADMIN`. On Linux this is a file capability, granted with
`setcap`. See [TUN Capture](../03-features/runtime-management.md#tun-capture)
for the full workflow and caveats.

## enable / disable

Enable TUN on the current connected config, or return it to local proxy mode:

```bash
xrat tun enable
xrat tun disable
xrat tun enable --json
```

The runtime owner checks engine support, capabilities, interface ownership, and
native config validity before stopping the current session. It reconnects the
same config in the requested mode and saves `runtime.tun.enabled` only after a
successful change. Startup or save failures attempt to restore the previous mode
and connection. Repeating a command does not restart an already matching session.
When disconnected, the setting is saved for the next connect.

Normal toggles do not require setup or daemon restarts. First-time privileges,
replacement binaries, or upgrading a daemon from a version without live toggles
may require setup and one restart; the error explains the next step. A CLI command
never takes over a standalone TUI's running session: press `U` in that TUI instead.
A global `--config` override must match the daemon's config file.

In the TUI, press `U` for the same checked live toggle. Its result appears in the
status message and log. You can also change only `runtime.tun.enabled` in settings
and save with `Ctrl+S` to apply immediately. Save other settings separately when
changing the enabled flag; their existing restart workflow still applies.

## status

Report TUN readiness for the current configuration:

```bash
xrat tun status
```

```text
TUN status
engine                xray 26.7.11
active capture        yes
active config         a1b2c3
active interface      xrat0
engine check          ready
configured TUN        yes
configured interface  xrat0
systemd service       ready (NoNewPrivileges=false override present)
engine file           /home/user/.local/share/xrat/cores/xray/xray  ready (cap_net_admin,cap_net_raw=ep)
xrat file             /usr/local/bin/xrat  ready (cap_net_admin,cap_net_raw=ep)
```

`--json` prints the same data as structured JSON. `tun_enabled` is the saved
setting; `tun_active` requires a running session and an owned kernel TUN device
with a matching interface index. It does not prove external traffic or DNS
reachability. `ready` covers engine support and privilege checks; it is separate
from active capture.

## setup

Grant `CAP_NET_ADMIN` and `CAP_NET_RAW` to the files TUN needs and configure systemd service permissions:

```bash
xrat tun setup
```

This runs `sudo setcap cap_net_admin,cap_net_raw+ep <file>` for the selected
engine binary and for the xrat binary. On Linux systems with an installed systemd
user service, it also automatically installs a drop-in override
`~/.config/systemd/user/xrat-daemon.service.d/10-tun.conf` (`NoNewPrivileges=false`)
and reloads the user daemon (`systemctl --user daemon-reload`). Use `--dry-run` to print
the commands and planned file changes without running them.

Restart an installed systemd daemon with
`systemctl --user restart xrat-daemon.service` so it picks up the override and
capabilities. For a standalone daemon, use `xrat daemon restart`. File capabilities are lost whenever the managed engine binary
is reinstalled or upgraded, so re-run `xrat tun setup` after `xrat install` or
`xrat upgrade`.

## Requirements

- Linux and either Xray >= 26.7.11 or sing-box. V2Ray TUN is unsupported.
- `libcap` tools (`setcap`/`getcap`) available on `PATH`.

When a daemon is running, `tun status` also inspects its effective `CAP_NET_ADMIN`
and `NoNewPrivs` through the daemon socket peer PID. The JSON `daemon` object
contains its PID, readiness and status; it is `null` when no daemon is reachable.
A drop-in on disk alone does not make an already-running daemon ready.

# tun

Prepare and inspect the system privileges managed TUN capture needs.

```bash
xrat tun <subcommand>
```

TUN capture creates a network interface and system routes, so the engine binary
needs `CAP_NET_ADMIN`. On Linux this is a file capability, granted with
`setcap`. See [TUN Capture](../03-features/runtime-management.md#tun-capture)
for the full workflow and caveats.

## enable / disable

Save the TUN capture setting without editing TOML manually:

```bash
xrat tun enable
xrat tun disable
```

These commands update only `runtime.tun.enabled` in the selected config file
(including a global `--config` override). Repeating a command is safe. They do
not grant privileges or change an active connection. Before enabling capture,
run `xrat tun setup` as needed. Restart a running daemon, then reconnect to apply
either change; disabling the setting alone does not stop an active TUN interface.

In the TUI, press `U` to toggle and save the same setting. The result appears in
the status message and log. For all TUN options, press `,`, select `runtime.tun`,
edit fields, and save with `Ctrl+S`.

## status

Report TUN readiness for the current configuration:

```bash
xrat tun status
```

```text
TUN privileges
engine           xray
tun enabled      yes
interface        xrat0
systemd service  ready (NoNewPrivileges=false override present)
engine file      /home/user/.local/share/xrat/cores/xray/xray  missing (run `xrat tun setup`)
xrat file        /usr/local/bin/xrat                            missing (run `xrat tun setup`)
```

`--json` prints the same data as structured JSON.

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

- Linux. Other platforms do not use file capabilities; set up TUN privileges with
  the OS-specific mechanism.
- `libcap` tools (`setcap`/`getcap`) available on `PATH`.

When a daemon is running, `tun status` also inspects its effective `CAP_NET_ADMIN`
and `NoNewPrivs` through the daemon socket peer PID. The JSON `daemon` object
contains its PID, readiness and status; it is `null` when no daemon is reachable.
A drop-in on disk alone does not make an already-running daemon ready.

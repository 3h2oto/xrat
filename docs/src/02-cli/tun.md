# tun

Prepare and inspect the system privileges managed TUN capture needs.

```bash
xrat tun <subcommand>
```

TUN capture creates a network interface and system routes, so the engine binary
needs `CAP_NET_ADMIN`. On Linux this is a file capability, granted with
`setcap`. See [TUN Capture](../03-features/runtime-management.md#tun-capture)
for the full workflow and caveats.

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

Restart the daemon afterwards (`xrat daemon restart`) so it picks up the override
and capabilities. File capabilities are lost whenever the managed engine binary
is reinstalled or upgraded, so re-run `xrat tun setup` after `xrat install` or
`xrat upgrade`.

## Requirements

- Linux. Other platforms do not use file capabilities; set up TUN privileges with
  the OS-specific mechanism.
- `libcap` tools (`setcap`/`getcap`) available on `PATH`.

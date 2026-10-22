# daemon

Run or control the daemon supervisor process.

```bash
xrat daemon <action>
```

## Actions

| Action      | Description                                           |
| ----------- | ----------------------------------------------------- |
| `start`     | Start the long-lived daemon process                   |
| `status`    | Show daemon IPC reachability and protocol information |
| `stop`      | Request daemon shutdown via local IPC                 |
| `restart`   | Restart the daemon, reloading config and runtime      |
| `install`   | Install xrat-daemon as a background service (per OS)  |
| `uninstall` | Remove the installed background service              |

The hidden internal `run-server` action is used by the daemon launcher and is
not a user-facing command.

---

## daemon start

Start the long-lived daemon supervisor process.

```bash
xrat daemon start
```

### Flags

No command-specific flags.

### Behavior

1. Forks a background daemon process
2. Creates a Unix domain socket at `<runtime_dir>/daemon.sock`
3. Runs the supervisor event loop with:
   - Health checks every 15 seconds
   - IPC event processing from CLI commands
   - Auto-rotation scheduling (if enabled)
4. Reattaches to any stale runtime sessions from previous daemon runs

### Daemon Features

- **IPC server**: Listens for commands from `xrat connect`, `xrat disconnect`,
  etc.
- **Health monitoring**: Periodically checks proxy liveness, triggers rotation
  on failure
- **Auto-rotation**: Scheduled proxy switching with cooldown and candidate
  testing
- **Session reconciliation**: Detects and recovers from stale sessions on
  restart

---

## daemon status

Show daemon IPC reachability and protocol information.

```bash
xrat daemon status
```

### Flags

No command-specific flags.

### Output

```
Daemon Status
─────────────────────────────────
Socket:     /home/user/.config/xrat/runtime/daemon.sock
Reachable:  yes
Protocol:   v1
```

If the daemon is not running or the socket is unreachable, prints an error.

---

## daemon stop

Request daemon shutdown via local IPC.

```bash
xrat daemon stop
```

### Flags

No command-specific flags.

### Behavior

1. Connects to the daemon socket
2. Sends a shutdown request
3. Daemon gracefully terminates:
   - Stops the active proxy session (if running)
   - Closes the IPC socket
   - Exits cleanly

---

## daemon restart

Restart the daemon after editing `config.toml`.

```bash
xrat daemon restart
```

### Flags

No command-specific flags.

### Behavior

1. If the daemon is running, requests shutdown via IPC and waits for the socket
   to close
2. Spawns a fresh daemon process, which re-reads `config.toml` and reattaches
   the persisted runtime session
3. If the daemon was not running, simply starts it

Restart always uses the app IPC/start flow, even when a systemd user service is
installed, so behavior stays consistent across manual and service-managed
daemons.

---

## daemon install

Install xrat as a background service. The service manager is selected by
operating system:

| OS              | Service manager      | Location                                  |
| --------------- | -------------------- | ----------------------------------------- |
| Linux           | systemd user service | `~/.config/systemd/user/`                 |
| macOS           | launchd user agent   | `~/Library/LaunchAgents/`                 |
| FreeBSD/OpenBSD | rc.d script          | `/usr/local/etc/rc.d` or `/etc/rc.d`      |

On FreeBSD/OpenBSD the rc.d script is system-wide and enabling/starting it
requires root (run under `sudo`), unlike the per-user systemd and launchd paths.

```bash
xrat daemon install [--start] [--with-api] [--dry-run]
```

### Flags

| Flag         | Description                                                           |
| ------------ | --------------------------------------------------------------------- |
| `--start`    | Start the daemon immediately after enabling the service               |
| `--with-api` | Also install the standalone HTTP API service                          |
| `--dry-run`  | Print the generated unit and planned actions without writing anything |

### Behavior (Linux/systemd)

1. Resolves the current binary path via `std::env::current_exe()`
2. Generates `xrat-daemon.service` from the template in `crates/xrat-app/templates/systemd/`
   with the resolved binary path and configured XRAT root
3. Writes the service file to `~/.config/systemd/user/` (respects
   `$XDG_CONFIG_HOME`)
4. Runs `systemctl --user daemon-reload`
5. Runs `systemctl --user enable xrat-daemon.service`
6. If `--start`: runs `systemctl --user start xrat-daemon.service`
7. If `--with-api`: generates and installs `xrat-api.service` as well

macOS and BSD follow the same shape with their templates
(`crates/xrat-app/templates/launchd/`, `crates/xrat-app/templates/rc.d/`): generate the unit, write it to the
service location, register it (`launchctl bootstrap` / `sysrc`+`service` /
`rcctl enable`), and start it when `--start` is passed.

### Example

```bash
xrat daemon install --start
```

```
Written: /home/user/.config/systemd/user/xrat-daemon.service
Reloaded systemd user daemon.
Enabled: xrat-daemon.service
Started: xrat-daemon.service

Daemon installed successfully.
```

### Dry run

```bash
xrat daemon install --dry-run
```

Prints the generated service unit and the systemctl commands that would run,
without writing any files or calling systemctl.

---

## daemon uninstall

Remove the installed xrat-daemon background service (systemd/launchd/rc.d
depending on the OS).

```bash
xrat daemon uninstall [--dry-run]
```

### Flags

| Flag        | Description                                     |
| ----------- | ----------------------------------------------- |
| `--dry-run` | Print planned actions without removing anything |

### Behavior (Linux/systemd)

1. Stops `xrat-daemon.service` (non-fatal if not running)
2. Disables `xrat-daemon.service`
3. Removes `~/.config/systemd/user/xrat-daemon.service`
4. Repeats for `xrat-api.service` if present
5. Runs `systemctl --user daemon-reload`

macOS and BSD perform the equivalent stop/disable/remove with their service
managers (`launchctl bootout` / `service stop`+`sysrc` / `rcctl`).

User config, database, logs, and all application state are **preserved**.

---

## Windows machine service

Windows service installation requires an elevated terminal:

```powershell
xrat daemon install --dry-run --start
xrat daemon install --start
```

The service runs as LocalSystem from a protected Program Files directory and
stores its configuration, SQLite catalog, runtime files, and engine binaries
under the ProgramData known folder, in `XRAT-Service`. The installing user is
registered as its operator. When elevating with a different administrator account,
pass `--operator-sid <SID>` for the intended operator.

Provision service cores explicitly from an elevated terminal while the machine
service is stopped. Core installation uses official release metadata/checksums,
checks the staged executable version, and writes only the fixed protected core
path. It leaves the service stopped and does not edit user configuration or PATH.

```powershell
xrat --profile machine daemon stop
xrat --profile machine install xray --version 26.9.30
xrat --profile machine install sing-box --version 1.13.21
xrat --profile machine daemon start
```

An exclusive profile lease prevents core provisioning while a service host is
active, and prevents service startup during provisioning. Locked executable or
activation errors preserve the previous core. Routine gateway requests cannot
install cores.

The registered operator can use these commands from an ordinary terminal:

```powershell
xrat --profile machine daemon status
xrat --profile machine daemon start
xrat --profile machine import https://example.com/subscription
xrat --profile machine list configs
xrat --profile machine connect <config-ref>
xrat --profile machine status --json
xrat --profile machine disconnect
xrat --profile machine daemon stop
```

`--profile user` is the default. Machine commands use the service gateway and
never open a user catalog or fall back when the service is unavailable. Path and
database overrides are rejected. Supported catalog commands currently include
`add`, `import`, `update`, `list`, `show`, `enable`, `disable`, `delete`,
`restore`, and `purge`. Machine `rotate enable/disable/status/now` also use the
supervisor; rotation controls are volatile and do not edit administrator policy.
`rotate now --refresh` refreshes subscriptions through the ordinary client.
Machine `logs` reads service events and fixed daemon/engine feeds, including
`--follow`, event `--level`/`--format`, and `logs clear --yes`. Machine log requests
accept `--lines 1..2000`; file reads are bounded to 512 KiB per request and report
omitted older content. The operator never opens protected log files directly.
Machine frontends run from an ordinary terminal:

```powershell
xrat --profile machine tui
xrat --profile machine serve
xrat --profile machine webui --no-open
```

These frontends require a ready service and reject elevated tokens. Their
configuration and SQLite UI/authentication store live in `machine-client` under
the user data root. They read and modify the service catalog through the
gateway; that local store never substitutes for an unavailable service.
Configure Web credentials in the frontend configuration before enabling its Web
interface. HTTP authentication, roles, and CSRF checks still apply. Machine legacy exports
(`/json`, `/configs`, and key-based `/b64`) require a configured `server.key`;
absence of a key does not enable anonymous machine exports. Account-token
subscription access retains its existing scope and revocation checks.

Test jobs use administrator-configured service targets, timeouts, concurrency,
and cores. TUI split-policy edits and Web core/GeoIP maintenance, policy
validation, and upgrades require local administrator maintenance. Frontend
server settings and TUI preferences remain editable. PAC policy, runtime state,
probe history, and operational logs come from the service; API request logs stay
in the frontend store with a separate display cursor.

Driver provisioning and coordinated service upgrade are still under
implementation. Windows `daemon install --with-api` and `--tun` currently return
an explicit unsupported-operation error.

Import files and subscription URLs are read by the invoking user. The service
receives bounded link bodies and source metadata, then uses the shared parsing
and catalog lifecycle services. Root engine JSON and structured engine extension
parameters require administrator provisioning; the gateway rejects them. Routine
requests cannot supply executable paths, SQL, service configuration, or driver
sources. Listing pagination preserves shared filters and ordering, and exports
include the complete filtered selection.

After service startup, crash recovery, or restart, the service reconciles and
cleans stale runtime ownership and remains disconnected with rotation disabled.
Connect explicitly to resume. Uninstall requires elevation and preserves the
protected profile and installed executable. Source and local tests do not yet
establish native Windows runtime acceptance.


## IPC Protocol

The daemon uses JSON over Unix domain socket with protocol version 1.

### Request Types

| Request             | Description                         |
| ------------------- | ----------------------------------- |
| `DaemonPing`        | Check daemon reachability           |
| `DaemonShutdown`    | Request graceful shutdown           |
| `RuntimeStatus`     | Get proxy runtime status            |
| `RuntimeConnect`    | Start a proxy session               |
| `RuntimeReplace`    | Atomic disconnect-old + connect-new |
| `RuntimeDisconnect` | Stop the active proxy session       |
| `ProxyStart`        | Enable auto-rotation                |
| `ProxyStatus`       | Get rotation status                 |
| `ProxyStop`         | Disable auto-rotation               |

Manual proxy rotation uses `RuntimeReplace` with a manual trigger and optional
candidate config ID. There is no separate `ProxyRotate` IPC request type.

### Response Envelope

```json
{
  "protocol_version": 1,
  "ok": true,
  "code": 200,
  "message": "success",
  "payload": { ... }
}
```

## Related

- [`rotate`](rotate.md) — control auto-rotation scheduling
- [`proxy`](proxy.md) — local proxy endpoints, shell, desktop, and PAC helpers
- [`connect`](runtime.md#connect) — start a proxy via daemon IPC
- [`status`](runtime.md#status) — check proxy status via daemon IPC
- [`init`](init.md) — initialize config directory before first use
- [systemd](../04-deployment/systemd.md) — full systemd deployment guide

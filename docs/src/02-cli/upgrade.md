# upgrade

Self-upgrade the running `xrat` binary, either by downloading the latest GitHub
release (default) or by building from a local source checkout.

```bash
xrat upgrade [OPTIONS]
```

On Unix, the new binary is staged beside the current executable and atomically
renamed over it. On Windows, a verified helper applies the staged transaction
after the upgrading process exits; inspect `xrat upgrade --status` afterward.
Upgrade discovery runs before database initialization, allowing recovery from a
database migration or connection failure. Completion requires migrations using
the updated binary.

## Release notes after upgrading

After a successful release upgrade, xrat caches the installed release's notes in
`release-notes.json` under the xrat data directory (`XRAT_PATH`, or
`~/.config/xrat`). Notes appear only in a “What's new” modal on the next TUI
launch of that version, with Markdown formatting and wrapped, scrollable text.
No release notes are printed by the upgrade command.

Use `↑`/`↓` or `j`/`k` to scroll, `PageUp`/`PageDown` to move a page, and
`Home`/`End` to jump to the start/end. Press `Enter` or `Esc` to dismiss and
save that version as seen. Closing the terminal before dismissal leaves the
notes pending for the next launch. Reinstalling the same version preserves
dismissal.

Viewing works offline. If release notes cannot be fetched, the modal shows a
link to the release instead. A notes fetch or cache failure does not fail the
upgrade. Source upgrades do not fetch GitHub release notes.

## Flags

| Flag                  | Description                                                     | Default |
| --------------------- | --------------------------------------------------------------- | ------- |
| `--source`            | Build and install from source instead of downloading            | off     |
| `--path <dir>`        | Source directory to build from when `--source` is set           | `.`     |
| `--version <tag>`     | Download a specific release tag instead of the latest           | latest  |
| `--force`             | Reinstall even when already on the requested version            | off     |
| `--timeout <secs>`    | HTTP request timeout in seconds for release downloads           | `120`   |
| `--tun-setup`         | Force running `tun setup` after upgrade to restore capabilities | off     |
| `--no-tun-setup`      | Skip restoring TUN capabilities after upgrade                   | off     |
| `--no-restart-daemon` | Do not restart running xrat-daemon after upgrade                | off     |

## Release upgrade (default)

```bash
xrat upgrade
```

1. Queries the latest GitHub release tag (or uses `--version`).
2. If the current binary already matches, prints `already using latest version`
   and exits without downloading. Use `--force` to reinstall anyway.
3. Downloads the matching `xrat-<version>-<arch>.tar.gz` archive with a progress
   bar, verifies it against `SHASUMS256.txt`, extracts the binary, and replaces
   the running executable.
4. Runs database migrations with the newly installed binary so any migration
   failure is reported as part of the upgrade instead of surfacing on the next
   unrelated command. If migrations fail, see [`db migrate`](db.md) for recovery
   details.
5. If TUN was previously configured (or when `--tun-setup` is passed),
   automatically restores TUN file capabilities via `tun setup`. If a running
   daemon is detected, restarts it so the process picks up the newly granted
   capabilities (configurable via `[runtime.tun].auto_setup_on_upgrade` and
   `[runtime.tun].restart_daemon_on_upgrade`).

Prebuilt archives are available for Linux (`x86_64` and `aarch64`, musl) and
macOS (`x86_64` and `aarch64`, darwin), and Windows (`x86_64` and `aarch64`,
MSVC ZIP). On other platforms or architectures (including FreeBSD/OpenBSD), use
`--source`.

```bash
xrat upgrade --version v0.2.1 --force
```

If GitHub requests fail, check connectivity to both `api.github.com` (release
lookup) and `github.com` (archives and checksums). You can use a working local
SOCKS proxy; replace the example port with your proxy's port:

```bash
ALL_PROXY=socks5h://127.0.0.1:1080 xrat upgrade
```

`--version` skips release lookup but still requires archive downloads. A local
source build below avoids GitHub release requests; Cargo may still need network
access for missing dependencies.

## Build from source

```bash
xrat upgrade --source            # builds from the current directory
xrat upgrade --source --path ~/code/xrat
```

Runs `cargo build --release` in the source directory, then installs the produced
`target/release/xrat` over the running binary. Requires `cargo` on `PATH` and a
`Cargo.toml` in the source directory.

## Notes

- Replacing a binary in a system directory (for example `/usr/local/bin`) may
  require elevated permissions; rerun with `sudo` if you hit a permission error.
- Only the binary is replaced. Man pages and shell completions are not updated;
  rerun `install.sh` if you want those refreshed too.

## Windows coordination

Run user-profile upgrades from an ordinary shell. The helper shuts down the user
daemon and managed connection, replaces the executable, applies user migrations,
and restarts a previously active daemon when permitted by `--no-restart-daemon`
and `runtime.tun.restart_daemon_on_upgrade`.

For the protected machine installation, use an elevated administrator shell:

```powershell
xrat --profile machine upgrade
xrat --profile machine upgrade --status
```

Machine downloads and helper files remain in administrator-owned staging. Before
replacement, the helper validates and pauses the optional ordinary-user API task
and stops the SCM service. It applies protected database migrations by starting
the updated service as SYSTEM. A previously stopped service is stopped again
after migration; restart suppression also leaves it stopped. The API task's
enabled state is retained, and its prior running state is restored when restart
is allowed. The task remains an ordinary interactive-user process.

Failure before replacement preserves the original binary and attempts to restore
prior service/task state. After migrations may have run, failure retains the new
binary, backup and transaction for explicit recovery; it does not roll back the
database. Inspect status and restore service/task state after resolving the
reported error. A retained staged/applying transaction blocks another upgrade
until reviewed. Linux `--tun-setup` flags are rejected; Windows driver setup is
explicit. Machine upgrades do not write an administrator's release-notes cache.

Native Windows service, task and migration recovery acceptance remains pending.

## Related

- [Installation Script](../01-getting-started/installation.md)
- [Build From Source](../01-getting-started/source-install.md)

# Windows support implementation and acceptance

The delivery target is Windows 11 x64 and ARM64. Track acceptance and issue
status in [the Windows milestone](https://github.com/mhyrzt/xrat/milestone/4)
and [the parent roadmap](https://github.com/mhyrzt/xrat/issues/262). Source
changes, local checks, native Windows results, and published releases are
separate evidence. This branch is not a release or proof of full support.

## Design

- User-local installation stays unprivileged. The optional machine service runs
  as LocalSystem and has its own protected profile; its installing user controls
  routine operations over authenticated local IPC without elevation.
- Xray and sing-box own their respective Windows TUN backends. The initial
  native validation versions are Xray 26.9.30 and sing-box 1.13.21. Existing
  V2Ray limitations remain explicit; Windows V2Ray backend work is tracked in
  [#266](https://github.com/mhyrzt/xrat/issues/266), related to #158 and #162.
- Desktop proxy configuration belongs to the interactive user. A LocalSystem
  service must not substitute its own HKCU for that user's settings.
- Windows IPC uses profile-specific, local-only named pipes, explicit DACLs,
  peer-user checks, bounded frames, and request deadlines. Unix keeps its
  existing transport and JSON protocol.
- Service configuration, executable paths, cores, driver acquisition, and
  machine policy remain administrator-controlled. Service clients must not
  supply arbitrary paths, commands, SQL, or root engine JSON.

## Implemented foundations requiring native acceptance

- Named-pipe request dispatch reuses the existing supervisor/runtime services.
  Its framing is tested independently of Windows; native pipe tests run only on
  Windows. Service identity and the application gateway are separate work.
- PowerShell 5.1/7 proxy scripts capture and restore the previous process
  environment without changing `NO_PROXY`. Apply generated output with
  `xrat proxy shell enable --shell powershell | Out-String | Invoke-Expression`.
  Disable restores the captured values; a child process cannot change its
  parent's environment by itself.
- Setup installs PowerShell completions into the user data directory, skips Unix
  man pages and the Unix launcher, and gives Windows PATH guidance. Use
  `xrat tui`; dot-source the generated `xrat.ps1` completion file in the shell
  where completion is wanted. No PowerShell profile is edited automatically.
- Desktop proxy settings use WinINET and a per-user recovery record. Disable
  restores the previous state only while the applied settings are still owned by
  XRAT; unrelated changes are retained and reported for explicit recovery.
- Release archive verification and extraction use Rust, exact checksum entries,
  and executable-only extraction. Windows self-upgrade stages a verified
  replacement and starts an updater helper. Exit the upgrading client within 120
  seconds and inspect `xrat upgrade --status` afterward. Running daemon/service
  coordination still needs implementation; the current staging path refuses a
  reachable daemon instead of replacing its executable.
- Windows source installation and CI share the browser-asset build script. Its
  bundle follows `just web-build`, including `public/` assets and removal of
  obsolete JavaScript bootstrap/controller files from reused output directories.
  Installer replacement stages on the destination filesystem and preserves an
  existing executable when replacement cannot proceed.

## Remaining implementation and evidence

### Protected machine profile and local gateway foundation

`app::services::machine_service` provides SCM hosting and the gateway
foundation. CLI machine catalog and runtime commands select this gateway before
user bootstrap; TUI and HTTP selection remain outstanding. User-local daemon
behavior remains separate. The machine
profile uses the Windows ProgramData known folder (`XRAT-Service`), without
consulting `XRAT_PATH`, CLI path overrides, or the interactive user's home
directory. Configuration, SQLite data, runtime state, logs, GeoIP data, and
engine binaries belong to this protected profile. Configuration and engine
binaries must be provisioned by an administrator; there is no automatic copy of
a user-controlled profile into the service.

Profile creation assigns an Administrators owner and a protected inheritable
DACL granting access only to SYSTEM and Administrators. Existing roots with a
different owner or ACL, and directory reparse points, are rejected rather than
repaired or adopted. Directory handles prevent replacement while the profile is
in use. Reprovisioning preserves configuration and the recorded operator SID;
changing the installing user requires a separate administrator workflow.

The machine gateway has a distinct versioned named-pipe endpoint. Only its
recorded operator can dispatch requests; Administrators pipe access does not
implicitly authorize another user's requests. The service host must run as
LocalSystem. Clients verify SYSTEM ownership of the connected pipe, and the
server identifies the token of the last received message, reverts the thread
identity, then dispatches. Framing, timeouts, connection limits, and shutdown
cleanup reuse the existing daemon conventions.

Supported requests are ping, runtime status/connect/disconnect/manual replace,
rotation start/status/stop, and a typed TUN enable/disable request. TUN support
remains subject to existing platform capability checks. TUN requests always use
the protected service config path. Unknown fields, client paths, executable
commands, SQL, root engine JSON, and caller-supplied automatic rotation triggers
are rejected. Requests use the shared supervisor handlers, including their
existing state transitions and operational events. `MachineRuntimeControl`
implements the shared runtime adapter without local fallback.

The typed catalog gateway uses shared config read/lifecycle and import services.
Subscription refresh shares deduplication, ordering, partial-failure summaries,
and header handling with local CLI updates. Machine refresh fetches through the
ordinary client and re-enters the catalog validator. A fail-closed Windows input
acquisition check denies LocalSystem file/URL import acquisition, and machine
startup also disables pre-rotation subscription fetching. This prevents
automatic rotation or a settings reload from bypassing catalog validation. Pure
parsing and validated-body persistence remain available to the service.

Machine rotation controls use the supervisor with shared status/result
presenters and initial-candidate selection. They never persist administrator
settings. Machine purge, local CLI purge, and Web purge use the shared lifecycle
owner; only soft-deleted rows are removed. The CLI supports machine-profile
add/import/list/show, enable/disable, delete/restore, connect/disconnect, and
runtime status, subscription updates, rotation, purge, and diagnostics. Files
and URLs are read by the ordinary client; source labels are metadata, never
privileged input locations. Every body in a multipart import is parsed and
checked before any write. Structured engine extensions are rejected for routine
imports.

Wire pages are bounded; the client aggregates filtered selections and exports
without losing ordering, filters, or top-N semantics. Oversized read pages are
retried at smaller sizes. A mutation is never replayed after an oversized reply
or transport failure. Settings edits, core/driver installation, and privileged
filesystem operations are absent from the operator gateway. Service lifecycle
uses SCM permissions. TUI and HTTP application integration remain outstanding;
existing HTTP authentication/roles are unchanged.

Application catalog facades preserve the public `AppServices`, `ConfigService`,
and lifecycle service types while forwarding machine reads and transitions to
the same service owners. Optional detail/reference lookups preserve missing
results without parsing error messages. Detached machine repositories return an
explicit unsupported error for direct persistence access; no user or Web-auth
database is attached as a catalog fallback. Contract fixtures compare local and
machine reads/lifecycle results and exercise authenticated HTTP subscription
exports with a separate empty user store. This is a composition seam; automatic
machine TUI/Web context construction remains under implementation.

Machine diagnostics use a shared bounded reader also used by dashboard logs.
Typed requests select persisted events or one of the fixed daemon/Xray/sing-box
feeds; callers cannot supply a path. CLI `logs` supports app-event JSON/TSV,
engine tables, follow cursors, level filters, and confirmed event clearing.
Machine requests accept 1..2000 entries and read at most 512 KiB per file per
request. Initial tails report omitted older content. Incremental reads retain
partial lines and page boundaries; oversized single lines report truncation.
Session changes and shorter files reset file cursors. Event follow batches are
limited in SQL, with no loss at page boundaries. Existing unbounded local
`events_after` calls retain their zero-limit behavior.

Local contract tests cover schema rejection, unauthorized callers, protocol
versions, identifier validation, service-owned TUN paths, supervisor errors,
idempotent disconnect results, and unavailable supervisors. Ignored native
checks cover protected directory ownership, ordinary-user denial, reparse-point
rejection, and registration/config preservation. These are wired into the
Windows workflow but require native results before acceptance. Actual
LocalSystem-to-ordinary-user gateway exchange remains a service-host acceptance
gate; cross-compilation alone does not establish it.

### Service lifecycle and acceptance

Windows `daemon install` provisions an administrator-owned executable in the
Program Files known folder (`XRAT-Service/xrat.exe`) and registers the
`xrat-daemon` LocalSystem service. Installation/removal requires an elevated
shell. When installing using another administrator account, supply the intended
operator's SID with `--operator-sid`; otherwise the current account is
registered. Reinstall preserves registration/configuration and refuses a
different operator or executable. Uninstall preserves the profile and
executable.

The global `--profile user|machine` defaults to `user`. Machine mode currently
supports `daemon install/start/stop/restart/status/uninstall`. For example,
`xrat --profile machine daemon restart` controls SCM, while ordinary
`xrat daemon restart` retains the separate user-daemon behavior. Machine mode
rejects CLI database/config/engine path overrides and never bootstraps a user
profile. The catalog and runtime commands listed above use typed adapters and
the same CLI presenters as user mode. Other operations report an explicit
unsupported error. The TUI selector and HTTP application adapters remain
outstanding. `--with-api` and `--tun` service-install flags are
explicitly rejected until their respective integrations are implemented.

Explicit elevated `--profile machine install <core>` provisions official
checksum-verified core executables at the fixed protected paths. Acquisition and
version validation reuse the existing core services. Archive selection happens
in memory before writing an administrator-owned staged file, avoiding elevated
execution from user-writable temporary directories. Atomic activation preserves
the old executable on failure. It changes neither user PATH/config nor machine
policy and leaves the service stopped. A held exclusive profile-file lease
serializes the service host with core provisioning; a second host or installer
fails instead of racing runtime ownership. Native lock, locked-target, and
replacement-ACL tests remain pending.

The service has an actual SCM dispatcher/control handler, pending checkpoints,
bounded transitions, and bounded restart recovery. Its registered operator may
start/stop/query it without permission to reconfigure it or rewrite its ACL.
Startup validates persisted process identity, reconciles stale state,
disconnects, and waits for explicit connection/rotation requests. HTTP and
automatic subscription fetching are disabled in the privileged host. Gateway
readiness precedes SCM RUNNING; shutdown closes the gateway, drains the
supervisor's shutdown operation, and attempts final cleanup even after task
failure. Fatal host failures also emit best-effort Windows Application event-log
records.

An ignored native test exercises real service registration, SYSTEM gateway
exchange under a restricted operator token, SCM control without administrator
membership, permission denial for service reconfiguration/ACL changes, dry-run,
reinstall, operator mismatch, and uninstall preservation. It requires an
isolated elevated Windows host with no existing XRAT service/profile and a built
native binary in `XRAT_WINDOWS_SERVICE_BINARY`. Run
`cargo test --locked -p xrat-app native_machine_service_lifecycle_and_restricted_operator -- --ignored --test-threads=1`.
These tests have not established native service or reboot acceptance yet.

Complete service-aware upgrades and normal-user CLI/TUI/Web application access.
Complete the Windows TUN privilege, driver, interface identity, route/DNS
ownership, live replacement, and crash recovery adapters before removing Windows
TUN rejection.

Standard GitHub runners provide native x64 and ARM64 build/test environments.
Use those first, including service, driver, and network tests where supported.
The workflow downloads checksum-pinned Xray 26.9.30 and sing-box 1.13.21 binaries
for each native architecture. Shared generated-config validators and local proxy
success/failure/timeout/cancellation fixtures exercise both engines. Engine
receipts record source, version, architecture, and archive/executable hashes.
These proxy fixtures do not exercise TUN capture or privileged network cleanup.

The isolated WinINET acceptance step changes runner settings temporarily and
checks manual/PAC transitions, restoration, and preservation of external changes.
The ordinary-owner pipe test disables the token's Administrators group and sets
medium integrity to check client read/write access without granting pipe-instance
creation. Pipes have an explicit medium mandatory label; a low-integrity owner
test verifies write denial. Generated shell
scripts are executed in Windows PowerShell 5.1 and PowerShell 7 where available.

Acceptance harnesses are Rust integration tests. `tests/windows_installer.rs`
invokes the actual installer helpers through a child PowerShell process; fixture
creation, assertions, and test reporting are owned by Rust. Set
`XRAT_TEST_POWERSHELL` to `powershell.exe` or `pwsh.exe`, then run
`cargo test --locked -p xrat --test windows_installer -- --ignored`.
`tests/windows_packaging.rs` accepts a native ZIP in `XRAT_WINDOWS_ARCHIVE` and
checks daemon lifecycle, HTTP assets/login, and updater transactions with
`cargo test --locked -p xrat --test windows_packaging -- --ignored`.
Pinned Windows SDK engine tests download and verify their own fixtures; run
`cargo test --locked -p xrat-sdk --test native_windows -- --ignored --test-threads=1`.
These tests are ignored in ordinary workspace runs because they require external
tools, network access, or native artifacts. The Windows workflow runs them
explicitly on each architecture.

Interactive Windows Terminal, desktop-session behavior, UAC, and reboot checks
must retain explicit pending acceptance until exercised on suitable Windows 11
machines; compilation or mocked tests cannot stand in for those results.

Record commit, architecture, runner/host identity, core versions and hashes,
commands, results, and cleanup snapshots with each owning issue. Run the same
fixtures through applicable CLI/TUI/HTTP adapters and preserve server
authorization. Never close #187/#191/#192/#262 from branch code alone: their
published-release criteria require a downloaded, checksum-verified release,
native execution, and actually served browser assets.

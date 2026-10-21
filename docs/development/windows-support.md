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

`app::services::machine_service` provides the service-host foundation. It is not
yet wired to SCM or selected by ordinary CLI/TUI startup. User-local daemon
behavior remains separate. The machine profile uses the Windows ProgramData
known folder (`XRAT-Service`), without consulting `XRAT_PATH`, CLI path
overrides, or the interactive user's home directory. Configuration, SQLite data,
runtime state, logs, GeoIP data, and engine binaries belong to this protected
profile. Configuration and engine binaries must be provisioned by an
administrator; there is no automatic copy of a user-controlled profile into the
service.

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

Config/subscription management, imports/exports, settings edits, desktop proxy,
and service lifecycle operations are not exposed by this gateway. CLI/TUI
machine-profile discovery, protected catalog access, and SCM hosting still need
integration before ordinary users can select this service. Existing HTTP
authentication and role checks are unchanged; the local operator gateway is not
an HTTP authorization bypass.

Local contract tests cover schema rejection, unauthorized callers, protocol
versions, identifier validation, service-owned TUN paths, supervisor errors,
idempotent disconnect results, and unavailable supervisors. Ignored native
checks cover protected directory ownership, ordinary-user denial, reparse-point
rejection, and registration/config preservation. These are wired into the
Windows workflow but require native results before acceptance. Actual
LocalSystem-to-ordinary-user gateway exchange remains a service-host acceptance
gate; cross-compilation alone does not establish it.

### Service lifecycle and acceptance

Complete the protected machine profile and typed application gateway before
enabling privileged service installation. Add SCM startup/shutdown/recovery,
service-aware upgrades, and normal-user CLI/TUI service access. Complete the
Windows TUN privilege, driver, interface identity, route/DNS ownership, live
replacement, and crash recovery adapters before removing Windows TUN rejection.

Standard GitHub runners provide native x64 and ARM64 build/test environments.
Use those first, including service, driver, and network tests where supported.
The workflow downloads checksum-pinned Xray 26.9.30 and sing-box 1.13.21 binaries
for each native architecture. Shared generated-config validators and local proxy
success/failure/timeout/cancellation fixtures exercise both engines. Engine
receipts record source, version, architecture, and archive/executable hashes.
These proxy fixtures do not exercise TUN capture or privileged network cleanup.

The isolated WinINET acceptance step changes runner settings temporarily and
checks manual/PAC transitions, restoration, and preservation of external changes.
The ordinary-owner pipe test disables the token's Administrators group to check
client read/write access without granting pipe-instance creation. Generated shell
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

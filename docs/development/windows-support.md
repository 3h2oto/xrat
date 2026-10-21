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
- Windows source installation and CI share the browser-asset build script.
  Installer replacement stages on the destination filesystem and preserves an
  existing executable when replacement cannot proceed.

## Remaining implementation and evidence

Complete the protected machine profile and typed application gateway before
enabling privileged service installation. Add SCM startup/shutdown/recovery,
service-aware upgrades, and normal-user CLI/TUI service access. Complete the
Windows TUN privilege, driver, interface identity, route/DNS ownership, live
replacement, and crash recovery adapters before removing Windows TUN rejection.

Standard GitHub runners provide native x64 and ARM64 build/test environments.
Use those first, including service, driver, and network tests where supported.
Interactive Windows Terminal, desktop-session behavior, UAC, and reboot checks
must retain explicit pending acceptance until exercised on suitable Windows 11
machines; compilation or mocked tests cannot stand in for those results.

Record commit, architecture, runner/host identity, core versions and hashes,
commands, results, and cleanup snapshots with each owning issue. Run the same
fixtures through applicable CLI/TUI/HTTP adapters and preserve server
authorization. Never close #187/#191/#192/#262 from branch code alone: their
published-release criteria require a downloaded, checksum-verified release,
native execution, and actually served browser assets.

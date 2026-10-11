## xrat v0.26.0

This release fixes application split tunneling, hardens managed runtime changes,
and introduces sing-box managed TUN support on macOS.

### Split tunneling fixes

- Sing-box treats process names, executable paths and directory patterns as OR
  alternatives. A selected application no longer needs to satisfy every matcher.
- Wrapper resolution uses literal exec targets and never executes scripts or
  mistakes helper commands such as pidof for the application. Variable-based
  wrappers report a warning so users can choose a real executable or directory.
- CLI validation and runtime compilation resolve desktop IDs consistently.
  Symlink targets retain canonical executable paths, and TUI selections retain
  desktop identities and application helper coverage, including running apps.
- Regression fixtures avoid depending on locally installed browsers or PATH.

### Runtime hardening

- Linux process-inspection capabilities are granted only to engine executables.
  XRAT receives network capabilities for interface cleanup.
- Failed live changes report configuration rollback failures. Rollback refuses
  to overwrite newer TUN settings while preserving unrelated configuration.
- Ownership records remain available until verified cleanup succeeds.

### macOS

- Managed TUN supports sing-box with a root runtime owner and numbered utun
  interface, defaulting to utun99. Xray managed TUN remains Linux-only.
- Application discovery includes installed .app bundles and running executables;
  bundle directory rules include nested helper processes.
- Kernel interface indices are verified. Cleanup waits for the engine to release
  its utun socket instead of deleting arbitrary interfaces. Route preflight
  rejects conflicting capture routes and VPN default routes.
- strict_route and route_exclude_address are rejected on macOS in this initial
  implementation. Use explicit configuration and database paths with sudo;
  tun setup prints instructions and does not install a root launchd service.

macOS support is initial: native CI covers compilation, resolver/validation
guards and a privileged utun lifecycle contract. Full managed application traffic
and DNS interception acceptance on macOS remains pending; Linux traffic tests do
not establish macOS runtime proof.

### Validation and upgrade

Workspace formatting, clippy and tests passed locally. Checksum-pinned Xray
26.7.11 and sing-box 1.13.21 passed five isolated Linux capture scenarios with
40 whitelist/blacklist application routing checks, protected DNS and exact
network baseline restoration. CI repeats platform and native acceptance checks.

No database migration. Restart the daemon after upgrading and rerun xrat tun
setup on Linux to apply the updated per-file capabilities. Recheck resolved split
rules with xrat tun split status, especially for variable-based wrapper scripts.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.25.0...v0.26.0

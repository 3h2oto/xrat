## xrat v0.32.0

This release adds experimental Windows x64 and ARM64 support, including native
daemon control, protected background services, PowerShell integration, desktop
proxy restoration, and coordinated upgrades.

### Windows support

- Publish Windows x64 and ARM64 ZIP archives with SHA-256 checksums alongside
  the Linux and macOS archives. The PowerShell installer verifies downloads
  before replacing the executable.
- Add local-only named-pipe daemon control and a protected LocalSystem service.
  The registered operator can use shared CLI, TUI, and Web application services
  without routinely elevating the interface.
- Add PowerShell proxy scripts, completions and setup guidance, plus
  ownership-aware WinINET manual/PAC proxy restoration.
- Coordinate ordinary-user and machine-service upgrades, preserving prior
  running state and retaining recovery evidence when replacement fails.
- Add managed Windows TUN integration for Xray and sing-box, with protected
  Wintun provisioning, adapter identity checks and ownership-aware cleanup.
  Windows V2Ray TUN remains outside this release's scope.
- Exercise native x64/ARM64 installers, services, security boundaries, proxy
  engines, packaged daemon/browser assets/login, and upgrade transactions in CI.

Thanks to [@subfocusx](https://github.com/subfocusx) for the initial Windows
contribution in [#259](https://github.com/mhyrzt/xrat/pull/259). The runtime and
acceptance implementation is tracked in
[#268](https://github.com/mhyrzt/xrat/pull/268).

### Experimental status and remaining checks

> [!WARNING]
>
> Windows support is experimental. Native automated checks cover the service,
> proxy and upgrade paths, including Wintun adapter creation/release. Actual TUN
> traffic, DNS, IPv4/IPv6, bypass, replacement and crash recovery remain
> unverified. Interactive Windows 11 UAC, Terminal, desktop, login and reboot
> checks also remain outstanding. These results do not establish full Windows
> support.

Track remaining acceptance in the
[Windows Support roadmap](https://github.com/mhyrzt/xrat/issues/262). Feel free
to [open an issue](https://github.com/mhyrzt/xrat/issues) or
[submit a pull request](https://github.com/mhyrzt/xrat/pulls) to report problems
or help improve support.

### Upgrade notes

- This release adds no database migrations. Existing configurations remain
  valid.
- Windows users should use the PowerShell installer or the ZIP matching their
  architecture. Machine service/core/driver provisioning requires elevation;
  routine operator access uses the protected service gateway.
- Managed Windows TUN targets Xray `>=26.9.30, <27` and sing-box
  `>=1.13.21, <1.14`. Follow the
  [TUN guide](https://github.com/mhyrzt/xrat/blob/v0.32.0/docs/src/02-cli/tun.md)
  for setup and limitations.
- Refresh an already open browser tab after upgrading.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.31.3...v0.32.0

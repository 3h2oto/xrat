## xrat v0.31.2

This patch improves dashboard resizing and makes account sign-out easier to find.

### Web UI fixes

- Size throughput, probe-history, and average-latency charts from their HTML
  containers, keeping SVG updates out of dashboard layout calculations when
  the window is resized.
- Move **Sign out** beneath the account's **Managed on host** or **Local account**
  label. The button is available in the same place for every account role.

### Upgrade notes

- Refresh an already open browser tab after upgrading to load the updated UI.
- This release adds no database migrations or configuration changes.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.31.1...v0.31.2

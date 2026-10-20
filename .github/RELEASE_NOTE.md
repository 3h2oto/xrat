## xrat v0.31.1

Install the Web UI as an app, import text files by drag and drop, and read
continuous probe-history lines across missing measurements.

### Web UI

- Add an installable **XRAT WebUI** with a manifest, application icons, and an
  offline connection page. Only the public offline document is cached;
  configurations, credentials, API responses, and app bundles remain
  network-only.
- Improve the import dialog with a text-file dropzone, file picker, read status,
  and file-read errors. Loaded content remains editable and requires explicit
  import confirmation.
- Fix mobile sidebar height and safe-area padding, and left-align the account
  footer while preserving the collapsed icon rail.
- Connect valid probe-history points across missing or invalid measurements.
  Missing samples remain unavailable in readouts and are never plotted as zero.
  Traffic charts continue to show gaps when samples are unavailable.

### TUI and release fixes

- Skip invalid negative latency values in TUI probe graphs while connecting
  available TCP, ICMP, and real-delay measurements across missing runs.
- Fix crates.io publication of embedded Web UI assets and add a validated
  recovery path for partially published releases.

### Upgrade notes

- Browser installation requires HTTPS for remote hosts; localhost also supports
  installation over HTTP. The installed app still needs a reachable XRAT server.
- This release adds no database migrations or configuration changes. The HTTP
  server and Web UI remain disabled by default.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.31.0...v0.31.1

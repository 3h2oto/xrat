## xrat v0.31.0

Manage XRAT from the browser with a new Dioxus Web UI and Rust/UI components,
sharing the same configurations, database, and managed runtime as the CLI and
TUI.

### Web UI

- View runtime status, configuration health, live throughput, and probe history
  in Overview.
- Browse configurations and subscriptions with searchable multi-select filters,
  sorting, pagination, and direct row/bulk actions. Config test columns show
  actual queued and running stages, then return to saved measurements.
- Share checked or filtered configurations through clipboard and locally
  generated QR codes. Create dynamic `/b64` subscription links that follow
  filters exactly and accept `token=` or the legacy `key=` parameter.
  Subscription rows can copy or display a QR for their original source URL.
- Inspect XRAT and engine logs, background jobs, settings, accounts, users, and
  maintenance tools. Responsive layouts, light/dark themes, and page-specific
  loading skeletons are included.
- Add persistent administrator/operator/viewer accounts, scoped API tokens,
  revocable subscription-only credentials, CSRF/origin checks, and
  password-confirmed host operations. Support local HTTP and HTTPS through a
  reverse proxy or direct TLS.
- Embed browser assets in native release archives, Docker images, and published
  crates; the running UI does not depend on a public CSS/JavaScript CDN.

### Shared behavior and fixes

- Reuse engine installation, GeoIP, upgrade, and test-runner services across
  entrypoints.
- Remove obsolete subscription configs and their test/stopped-session history
  after non-empty refreshes. Configurations referenced by active or
  transitioning runtime sessions are retained until a later refresh.
- Add README screenshots and setup instructions, Web UI documentation, and
  browser-controller ownership guidance.

### Upgrade notes

- The HTTP server and Web UI are **disabled by default**. Configure `[server]`
  and `[server.web]`, including an administrator username and Argon2id password
  hash, before enabling the daemon-hosted UI. There is no default password. See
  the
  [Web UI guide](https://github.com/mhyrzt/xrat/blob/v0.31.0/docs/src/03-features/web-ui.md).
- New SQLite/PostgreSQL migrations **0025** and **0026** add Web identity,
  session/token, job, and subscription-credential storage. Back up your database
  before upgrading.
- Anyone holding a subscription URL can use its credential; URL filters are not
  access restrictions. Use HTTPS for remote access.
- This release adds the `xrat webui` and `xrat hash-password` commands and the
  `server.web` configuration section. Existing CLI/TUI entrypoints remain
  available.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.30.3...v0.31.0

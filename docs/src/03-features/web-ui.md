# Web UI

XRAT's Dioxus browser interface shares the existing server on port **18203**. It
manages one shared database and proxy runtime. CLI and TUI remain available.

## Configure the administrator

Generate an Argon2id hash with `xrat hash-password`, supplying the password on
standard input. Keep the generated hash in a secret environment variable:

```toml
[server]
host = "127.0.0.1"
port = 18203

[server.web]
enabled = true
admin_username = "admin"
admin_password_hash = { env = "XRAT_ADMIN_PASSWORD_HASH" }
transport = "local_http"
```

Run `xrat webui` to open the browser, or `xrat webui --no-open` on a headless
host. `xrat serve` also serves the UI when `server.web.enabled` is true. The
daemon hosts it when `server.enabled` is true. Authentication is required even
on loopback. No default password exists. Changing the configured administrator's
password hash invalidates its sessions after restart. This account cannot be
changed or disabled through the browser.

Additional accounts are created in **Users** by an administrator. Temporary
passwords must be replaced on first login. Viewers can inspect the workspace;
operators can manage configurations and the runtime; administrators additionally
manage users, settings, permanent deletions, and host actions. All accounts
share one runtime; this is not an isolated multi-tenant proxy service.

## Navigation

Collapse the desktop sidebar to an icon rail using its panel toggle; click the
logo in the collapsed rail to expand it. The logo does not navigate. The choice
is saved in your browser. On small screens, use the navigation toggle to open or
fold the menu; selecting a page closes it. The sun/moon control switches between
XRAT's light and dark themes.

Click the sidebar avatar to open Account. Use Sign out on the Account page to
end the current session.

Select menus support arrow keys, Home/End, typing an option's initial, Enter/Space to select, and Escape to
close.

## HTTPS behind a proxy or PaaS

```toml
[server]
host = "0.0.0.0"
port = 18203

[server.web]
enabled = true
admin_username = "admin"
admin_password_hash = { env = "XRAT_ADMIN_PASSWORD_HASH" }
transport = "proxy_https"
public_url = "https://xrat.example.com"
```

Caddy, Traefik, nginx, or the PaaS terminates TLS and forwards requests to XRAT
using internal HTTP. **XRAT requires no certificate or private key in this
mode.** The configured public URL determines Secure cookies and permitted
browser origins. Forwarded headers are ignored, including for client identity;
login limiting uses a bounded global/account limiter. Restrict the internal
listener to the proxy network. Publish the application at the root of its
hostname.

Caddy:

```caddyfile
xrat.example.com {
    reverse_proxy xrat:18203
}
```

nginx (inside an existing TLS-enabled server block):

```nginx
location / {
    proxy_pass http://xrat:18203;
    proxy_buffering off;
    proxy_read_timeout 3600s;
}
```

Traefik: route an HTTPS router to a service with
`loadbalancer.server.port=18203`, attaching the platform's certificate resolver
or TLS policy. Certificate acquisition stays entirely with Traefik.

On a PaaS run `xrat webui --no-open`, bind `0.0.0.0`, configure the public HTTPS
URL, and omit the port from config to use the platform's `PORT` environment
variable. An explicit CLI or config port takes precedence. Use exactly one
replica and persistent storage for `XRAT_PATH`, including SQLite, config,
installed engines, and runtime state. PostgreSQL is also supported; it does not
make managed runtimes safe to run across replicas. TUN and host integration
require platform-provided privileges.

## Direct HTTPS

```toml
[server.web]
enabled = true
admin_username = "admin"
admin_password_hash = { env = "XRAT_ADMIN_PASSWORD_HASH" }
transport = "direct_https"
public_url = "https://xrat.example.com:18203"
certificate = "/run/secrets/fullchain.pem"
private_key = "/run/secrets/privkey.pem"
```

Use absolute certificate paths. Invalid or missing material fails startup.
Restart after replacing certificates. XRAT does not request ACME certificates.

## API and compatibility

Management routes live under `/api/v1`. Browser login uses an HttpOnly,
SameSite=Lax session cookie; mutations require the CSRF token returned by
`/login` or `/me` and an allowed Origin header. Sessions expire after 12 hours
idle or seven days total. API tokens use `Authorization: Bearer` and read/manage
scope, bounded by their owner's role. Password changes and account disabling
revoke access. Passwords, tokens, and credential exports are not cached.

Existing `/json`, `/b64`, `/configs`, and `/proxy.pac` compatibility routes
remain separate. The configured `server.key` only grants legacy read/export
access; it never grants management access. `/b64` also accepts Account API tokens
through `?token=...` and contains proxy credentials.
Configure a strong `server.key` before publishing these legacy endpoints. Health
and PAC retain their existing public/helper behavior.

Background jobs are persisted and visible in **Jobs**. Tests and scans can be
cancelled cooperatively. Jobs interrupted by a server restart are recorded as
interrupted and are never automatically repeated. Progress is available at
`/api/v1/jobs/events` through SSE, with `/api/v1/jobs` for polling clients.

Config test cells follow the shared runner's actual stage progress: queued
stages show a clock, running stages a spinner, and completed/failed/skipped stages
use distinct icons. Completed configurations reload their measured values.
Disabled test stages are not shown as running. Job completion or cancellation
clears active indicators.

## Share configurations

On Configs, the share icon and matching count apply to every matching configuration
across all pages. Choose **Copy configs**, **Show QR**, or **Subscription
link**. The selection bar provides icon-only Copy and QR
actions for exactly the checked configurations. These actions are available to
operators and administrators.

Copy and QR use the original configuration links separated by newlines. They are
snapshots. QR generation happens locally; Show QR is disabled when the matching
raw links exceed one QR code's capacity. The QR modal displays the code and its
download control without displaying the raw configuration text. QR codes include
the XRAT logo with high error correction. Use a subscription
link to share a larger batch in one QR.

Subscription links reapply search, protocol, state, subscription, test result,
country, maximum delay, and deleted visibility whenever the client refreshes.
Disabled and deleted configurations are included when they match the current
filters. Pagination and visible columns do not affect the subscription.

Choose an existing read-only Account token in the Subscription link dialog, or
paste a legacy server key. If no read-only tokens exist, follow the Account link,
create one, return, reload the list, and select it. The dialog creates a separate
subscription-only credential linked to that token: existing API token secrets
remain hash-only and are never revealed again. The link and its QR are shown
after selection. Revoking the parent token revokes all its subscription links.
These linked credentials cannot authenticate to the management API. Disabling an
account or changing its credentials also invalidates its links. The account must
still have operator or administrator access and must not require a password change.

Each config row has direct Copy and QR actions on desktop, alongside Connect,
Test, and Delete. Mobile keeps row actions in a menu. Bulk operations and selected-copy/QR actions
are displayed directly in the selection bar, without a separate menu or clear
button. Config export downloads
are replaced by these sharing controls; subscription CSV downloads remain available.

Subscriptions show View configs, Refresh, Copy URL, QR, Rename, and Delete as
direct row actions. Copy and QR share only that subscription's original source
URL. File and pasted-text sources have no URL to share; oversized source URLs
also disable QR. Source URLs are fetched on demand and never included in Overview
metadata. QR codes are generated locally.

Anyone with the subscription URL can use its credential. Filters are not access
restrictions: recipients can change them. Use HTTPS and redact query credentials
in reverse-proxy access logs.

## Building

The frontend uses Dioxus 0.7.10 and Rust UI's copyable component patterns.
Install Rust and Just, then:

```bash
rustup target add wasm32-unknown-unknown
just web-tools web-build
cargo build --locked
```

`web-tools` downloads checksum-pinned wasm-bindgen 0.2.122 and Tailwind 4.1.18
on Linux x86_64. Repeated builds reuse the matching wasm-bindgen version and
checksum-verified Tailwind binary. Downloads use resumable `.part` files and
replace cached files only after checksum verification, so interrupted downloads
do not truncate working executables. Other build hosts install wasm-bindgen from crates.io and
require Node.js/npm to run the pinned Tailwind CLI. Production assets are
embedded in the application binary. Source builds without assets retain
CLI/TUI/API operation but return a clear Web UI asset error. Release CI builds
assets once and embeds them in every platform binary and the published
application crate. Run `just web-check` for frontend checks.

## Logs and job history

Jobs, XRAT logs, Engine logs, and each tool have standalone sidebar items.
Host administration uses the shorter Host admin label in the sidebar.

Log output uses a read-only, colorized console with a bounded, scrollable height.
Copy copies the filtered output. Text can also be selected and copied manually. Both log views default to the last 15 minutes; Jobs defaults to the last 24
hours. The menu also offers one-hour, 24-hour, seven-day, and custom local
date/time ranges. Log levels can be filtered to errors, warnings, info, debug, trace, or unclassified.

XRAT logs show the latest 100 structured events. Engine logs require an operator
or administrator and read at most 64 KiB per stream from the latest runtime
session. Engine timestamps and level formats vary: unrecognized timestamps are
excluded by default and can be included explicitly, outside time filtering.
Unrecognized levels appear under All levels or Unclassified. These controls filter the
available snapshot, rather than fetching an unlimited historical archive.

Jobs show icon-only statuses with hover/focus labels and local timestamps such
as `17 Oct 2026 · 23:40:02`. Both log views order recognized timestamps newest
first, merging engine output streams by time. Untimed engine lines, when
explicitly included, follow timestamped entries in reverse file order.

The workspace footer shows the server version, stable-release availability, and
links to the GitHub repository and documentation. Release checks reuse XRAT's
server-side GitHub release provider with a five-second timeout and a shared
15-minute cache. A failed check displays an unavailable state rather than
claiming that the installation is up to date. The footer links to release notes;
it does not install updates automatically.

### Unlocking host administration

Host administration is a standalone administrator page. Engine installation,
rotation, TUN capture, GeoIP updates, configuration validation, application
upgrades, and data cleanup are grouped there. Tools contains only the TCP
scanner, configuration parser, and GeoIP lookup.

The page opens locked behind a blurred overlay. Re-enter the current account's
password to unlock controls for five minutes. Navigating away discards the
local unlock; Lock now also revokes it on the server. Purge, history clearing,
and application upgrades require an additional confirmation dialog.

The server enforces the gate on host actions, including rotation. A successful
`POST /api/v1/host/unlock` with `{"password":"..."}` returns an expiring grant.
Send that grant in `x-xrat-host-unlock` when submitting a host action. Grants are
bound to the authenticated browser session, kept only in memory, and are not
available through bearer-token authentication. The unlock and lock endpoints
retain the existing origin, CSRF, admin-role, and password rate-limit checks.
`POST /api/v1/host/lock` revokes the current session's grants. Ordinary config,
connection, and diagnostic actions retain their existing authorization.

### Page URLs

Each workspace page has a direct URL, for example `/ui/configs`, `/ui/subscriptions`,
`/ui/jobs`, `/ui/logs/xrat`, `/ui/logs/engine`, `/ui/host`, `/ui/settings`, `/ui/users`, and
`/ui/account`. Tools use `/ui/tools/tcp-scanner`, `/ui/tools/parse`, and `/ui/tools/geoip`.
Sidebar navigation updates browser history; refresh, direct links, and Back or
Forward restore the current page. Unknown routes show a page-not-found message.
Authentication is still required, and host administration always reopens locked.
Local form drafts and temporary unlock grants are not stored in the URL.

### Editing settings

The Settings toolbar and category controls stay visible while the selected
category's editor scrolls independently, including search results.

Settings are grouped into categories across the top, with a category selector on
small screens. Search matches setting labels and configuration keys across all
categories. Related settings are grouped into sections with switches for
booleans, selects for enumerations, number inputs, list editors, and password
fields for secrets. Each row states whether a runtime or daemon restart is
required.

Edits remain local until Save beside Reload is pressed. Save is enabled only
when there are pending changes and applies edits across all categories together.
An unsaved-change count,
per-row Reset, and Discard all make pending changes visible. Local edits survive
category and search changes while the Settings page remains open. Reload keeps
those edits and asks you to review them against refreshed server values.
Validation and stale-revision errors appear above the editor. Invalid changes
reject the entire batch without saving any settings; a save does not restart
services automatically. Existing secrets are never returned by
the server.

### Managing accounts

Users lists accounts in a searchable table with role filtering, enabled-state
switches, and a per-account action menu. Configuration-managed accounts are
marked and cannot be edited here. Create an account with New user, or choose
Edit account or Reset password from an existing account's menu. Role edits keep
the current password; password resets keep the role and enabled state.
Temporary passwords must be replaced at the next sign-in. Saving an existing
account signs it out and revokes its API tokens. The UI prevents disabling your
own account; saving other changes to your own account returns you to sign-in.

### Using diagnostic tools

TCP scanner, Parse configs, and GeoIP lookup each have their own input form and
result area. Results update while the job runs; Copy result copies the displayed
output, and View in Jobs opens job history. Leaving the page does not stop a
server-side job. Returning to a tool starts a fresh view; earlier output remains
in Jobs.

The scanner accepts up to 4,096 IPv4 or IPv6 addresses separated by commas or
newlines, and a port from 1 to 65,535. It uses a four-second timeout per address
and stores results in scanner history. Cancel a running scan from Jobs. Parse
configs recognizes links without importing them, and GeoIP lookup uses the
host's configured providers to resolve an address or hostname.

### Your account

Open Account from the sidebar avatar to see your username, role, and whether
credentials are managed on the host. Local accounts can change their password
using the current password and a new password of at least 12 characters. Eye
buttons toggle password visibility. Changing a password signs you out on every
device and revokes API tokens; sign in again with the new password. Accounts
with a temporary password must complete this step before using other pages.
Configured administrators update their password hash on the host instead.

Operators and administrators can create and revoke their own API tokens. Choose
Read only for access without changes, or Manage for operations permitted by your
current role. The list shows each token's name, access, and creation time. You
can have up to 32 tokens. A new token is shown once: copy it, then choose Done
to dismiss the reveal before managing or creating other tokens. Revoking a
token asks for confirmation and immediately removes its access.

### Config and subscription tables

Click a data column heading to switch between ascending and descending sorting.
Name and State stay visible in Configs; Name and Configs stay visible in
Subscriptions. Operators also have selection and action columns. Use Columns to
show connection details or individual tests, including real delay, TCP, ICMP,
download/upload speed, connect time, TTFB, HTTP status, country, ASN, and last
test time. Failed and missing test results use distinct icons with tooltips.
Missing/failed measurements sort after available numeric measurements.

Search and filters apply to the full loaded dataset before pagination. Configs
can be filtered by protocol, enabled/disabled/deleted state, subscription, test
outcome, country, and maximum real delay. Subscriptions can be filtered by source
type. Categorical filters support multiple selections: values within a filter
are combined with OR, while different filters are combined with AND. Lists
with more than eight choices include search. Choose 25, 50, or 100 rows per page; Reset restores the default view. Click
a subscription name to open Configs filtered to that source.

Filters, sorting, visible columns, page, and page size are stored in the URL.
For example:

```text
/ui/configs?protocol=vless&state=enabled&test=passed&max_delay=200&sort=delay&order=asc&columns=protocol,address,delay,tcp
/ui/subscriptions?source=url&sort=count&order=desc&per_page=50
```

Changes replace the current browser-history entry, so typing does not add a new
history entry for each character. Refreshing or opening a shared link restores
the view; browser Back/Forward restores the table state of that history entry.
Subscription CSV exports matching rows using the visible columns, or selected
matching rows when a selection is active. Config sharing uses Copy, QR, and
subscription links instead of export downloads.

Operators can select individual rows or the current page. Selection survives
page and sort changes and clears when filters change. Bulk actions use a snapshot of
eligible selected IDs: enable, disable, test, soft-delete, or restore configs;
refresh URL subscriptions or delete subscriptions. Deleted configs are eligible
for Restore, and subscriptions that are not URLs are ineligible for Refresh.

Deletion asks for confirmation. Config deletion can be undone with Restore;
subscription deletion permanently removes the subscription and its stored
configs. A bulk operation runs as one background job with progress and
cancellation available in Jobs. Cancellation stops remaining work and does not
undo completed items. Item failures also do not roll back successful items;
check Jobs and server logs before retrying. Selection is intentionally excluded
from shared URLs and is cleared after a job is accepted.

### Reading logs

XRAT logs and Engine logs share a read-only black console with separate time,
level, source, and message columns. Newest timestamped entries appear first.
Both refresh every three seconds and default to the last 15 minutes. XRAT logs
show the latest 100 operational events; Engine logs show the latest runtime
session, bounded to 64 KiB per output stream.

Use Search together with Level, Source, and Time range to narrow the output.
Levels include errors, warnings, info, debug, trace, and unclassified entries.
Wrap keeps long messages readable within the console; disable it to scroll long
lines horizontally. Latest returns the output viewport to the newest entries.
Copy and Download export the currently filtered text, including recorded
prefixes and multiline continuations.

Pause freezes the displayed entries and the relative time window. Search and
filters still work on that snapshot; Resume displays the latest fetched data.
Reload fetches current data without changing an active pause.

Engine output removes terminal colour/control sequences and recognizes common
Xray and sing-box severity/timestamp prefixes. Recognizable indented stack-trace
continuations stay with their parent entry. Recorded timestamps are converted to the browser’s local timezone for display.
Copy and Download preserve the original timestamps and offsets. Engine timestamps without a timezone use the
browser's local timezone for filtering. Lines without a recognized timestamp
are hidden initially: Include untimed reveals them after timed entries, outside
the selected time range. Missing timestamps are never replaced with the current
time.

Sign out sits beneath the API-token captions in Account’s left column (or
beneath the Password caption when API tokens are unavailable). The sidebar avatar opens Account directly and no longer
opens a logout menu.

When Configs or Subscriptions is empty, operators can press ⌘V (macOS) or
Ctrl+V (Linux/Windows) to paste a configuration or subscription into the import
dialog. Review the content and choose Import to submit it. Pasting inside a
form field or an open dialog keeps its usual behavior.

## Overview dashboard

Overview greets the signed-in user and displays the browser's local clock/date,
connection status, and active configuration. Rotate now is available to
operators when an enabled configuration exists. Administrators can change TUN
with password confirmation; the existing host-unlock, engine-support, and
privilege checks still apply. Applying TUN may reconnect the active session.
Click the active connection name for connection details, or Runtime in the
quick-action row for runtime diagnostics. Both open compact dialogs, keeping
the main dashboard height unchanged. Connection details include the
active configuration's reference, endpoint, transport, latest tests, subscription,
and recorded geolocation/fronting facts. Runtime shows listener health, installed
engine versions, daemon/rotation state, WebUI tasks, and configuration counts.
Runtime diagnostics are checked on demand; use its refresh icon to update the
snapshot. Wildcard proxy listeners display the host’s detected network address
when available, with a loopback fallback. Click a listener URL to copy its
protocol, address, and port. Explicit loopback binds remain local. Boolean
configuration and runtime facts use icons with accessible labels and tooltips.
Missing engines, an unreachable daemon, and disconnected sessions have
explicit states.

The subscription card shows the latest successful URL import/refresh time from
the scheduler's persisted refresh record. Renaming a subscription does not
change this time. Update subscriptions is unavailable without URL sources.

Config health uses enabled, non-deleted configurations and the latest recorded
result for the selected enabled test type. Its percentage is passed divided by
tested; untested configurations appear separately. Latency bars average only
successful, non-negative measurements and show their sample counts. These are
stored test results, not continuous live probes.

The throughput area chart derives bytes per second from successive engine
counter readings and actual elapsed time. XRAT collects counters every three
seconds while the WebUI server is running, even when the dashboard is closed.
The server retains a bounded five-minute window in memory and restores it after
a page refresh. Changing the runtime connection or restarting the WebUI server
starts fresh. Traffic statistics must be enabled. Missing samples and long gaps
break the graph; counter resets establish a new baseline without a spike.
The line chart shows the active configuration's latest 30 persisted tests with
separate enabled latency series. Each series connects its available measurements
across failed or absent results; missing measurements are never plotted as zero
and remain unavailable in sample readouts. The TUI probe graph follows the same
rule.
Hover/tap a sample for details; focus a chart and use Left/Right or Home/End to
inspect samples with the keyboard.

Overview refreshes periodically without removing the previous dashboard on a
failed request. A stale-data notice marks failures or prolonged responses,
live rates become unavailable, and actions are disabled until updates recover.
Disconnected sessions, disabled stats, unavailable engine stats, and missing
tests have separate messages. No test or connection is started automatically.


On desktop viewports at least 1001 pixels wide and 700 pixels tall, Overview
uses a compact layout that fits its cards, charts, and footer into the viewport.
Charts share the remaining height after the summaries. Smaller screens retain
natural scrolling; unusual content or extra status messages can scroll within
the desktop main area while the footer stays available.


Sidebar navigation separates Workspace, Management, and Tools. Management
contains Host admin, Settings, and Users, and is visible to administrators.
TCP scanner, Parse configs, and GeoIP lookup remain direct links under Tools.
Collapsed navigation preserves separators between groups.

Configs and Subscriptions paginate in the browser: 25 rows by default, with
50 and 100 available. Page and row count are stored in the query string. On
desktop, the table uses the height left after the page controls and pagination;
its rows and extra columns scroll internally beneath a sticky header. Pagination
and the app footer remain outside that scroll area. On smaller viewports, each
table is capped at half the viewport height (up to 36 rem), while surrounding
controls can scroll naturally. The table region can receive keyboard focus for
scrolling. Pagination does not reduce the initial API payload.

The app footer stays at the bottom of the window on every signed-in page.
Page content scrolls above it; its solid background and layer keep text from
showing through, while menus, tooltips, and dialogs remain above the footer.
On mobile, the navigation bar and footer reserve their actual heights.

### Shared API types

The server and frontend share API contracts from `xrat-web::types`, defined in
`crates/xrat-web/src/types/`. The server depends on `xrat-web` with default
features disabled, so it builds only shared contracts and filtering helpers with
Serde. The default
`ui` feature enables the Dioxus frontend binary; `just web-build` and
`just web-check` keep using that feature.

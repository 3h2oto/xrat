## xrat v0.30.1

Fix subscription imports that mistakenly fetched HTTP(S) proxy links as nested
subscriptions, causing unnecessary connections and TLS verification failures
(issue [#223](https://github.com/mhyrzt/xrat/issues/223): “HTTPS proxy URIs in
subscription content are incorrectly fetched as subscription URLs”).

### Fixes

- Parse downloaded subscription contents as proxy nodes without fetching embedded
  HTTP(S) links. The fix also applies to local node files and manual, TUI, and
  automatic subscription refreshes.
- Preserve HTTPS proxy nodes without contacting their endpoints during import.
  TLS certificate verification remains enabled for subscription downloads.

### Upgrade notes

Automatic URL-list expansion has been replaced by an explicit option. If your
local file contains subscription URLs rather than proxy nodes, use:

```bash
xrat import ./subscriptions.txt --url-list --name 'Combined subscriptions'
```

- List mode requires a local file with one HTTP(S) subscription URL per line;
  blank lines and `#` comments are ignored.
- Each listed URL is fetched once. Response contents are parsed as nodes without
  following embedded URLs.
- `--url-list` cannot be combined with `--header`. The combined file source is not
  automatically refreshed; re-run the command to refresh it.
- No database migration is added in this release.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.30.0...v0.30.1

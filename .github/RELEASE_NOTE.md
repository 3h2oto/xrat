## xrat v0.27.0

Release notes now appear once in a scrollable “What's new” modal after a release
upgrade, when you next open the TUI. The upgrade command keeps its usual output
without printing the notes a second time.

### TUI release notes

- Headings, emphasis, code, lists, and links receive Markdown formatting, with
  wrapped text and tolerant rendering of incomplete or malformed Markdown.
- Scroll with the arrow keys or j/k, move a page with PageUp/PageDown, and jump
  to the beginning or end with Home/End.
- Press Enter or Esc to dismiss. The installed version is marked as seen only
  after dismissal is saved; closing the terminal first leaves the notes pending.
- Release notes are cached locally and can be viewed offline. Reinstalling the
  same version preserves its dismissal state.
- Missing release notes show a release-page link. Fetching or saving notes does
  not turn a successful upgrade into a failure.

### Upgrade notes

No database migration or configuration changes are required. The cache uses
release-notes.json in the XRAT data directory (XRAT_PATH, or ~/.config/xrat).

Automatic caching applies to release upgrades run by v0.27.0 or newer. Older
binaries upgrading into this version do not create the cache. Source upgrades
remain offline and do not fetch GitHub release notes.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.26.0...v0.27.0

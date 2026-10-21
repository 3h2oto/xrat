## xrat v0.31.3

This patch improves TUI navigation, readability, and clipboard support, and
moves Web UI browser interactions into Rust.

### TUI improvements

- Synchronize panel numbers with the tab cycle, improve search and sort
  controls, and add table scrollbars, a row cursor, and contextual key hints.
- Keep connection status and mode visible, organize config details into
  sections, and improve tables and log views at narrow terminal widths.
- Add optional rounded borders through `tui.border_style` (`plain` remains the
  default).
- Support `h`/`l` pane navigation in settings and word deletion with `Ctrl+W` in
  settings and split-tunnel inputs. Cycle settings values with `[`/`]` or
  `<`/`>`.
- Improve Wayland clipboard support and add an OSC 52 terminal fallback.
- Keep confirmation messages visible.

Thanks to [@f02xygen](https://github.com/f02xygen) for the TUI improvements in
[#260](https://github.com/mhyrzt/xrat/pull/260).

### Web UI improvements

- Move dialogs, menus, tooltips, clipboard helpers, paste handling, chart
  navigation, and service-worker registration into Rust.
- Inline the WebAssembly bootstrap and remove the separate JavaScript
  controllers.
- Prevent copy dialogs from flashing when clipboard access succeeds.

See [#267](https://github.com/mhyrzt/xrat/pull/267) for the browser refactor.

### Upgrade notes

- Refresh an already open browser tab after upgrading to load the updated UI.
- This release adds no database migrations. Existing configurations remain
  valid; the new `tui.border_style` setting is optional.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.31.2...v0.31.3

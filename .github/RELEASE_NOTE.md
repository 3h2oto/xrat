## xrat v0.29.0

Define your own TUI palette and enjoy more consistent built-in themes across
panels, selected rows, Settings, and release notes.

### Custom themes

Create a named theme in config.toml, inherit a built-in palette, and override
only the colors you want to change:

```toml
[tui]
theme = "my-theme"

[tui.themes.my-theme]
extends = "catppuccin"
accent = "#89b4fa"
selection_foreground = "#cdd6f4"
```

- Color overrides accept `#RRGGBB`, ANSI color names, or `terminal`.
- Settings lists configured custom themes alongside the eight built-in themes.
  Preview changes live, save with `Ctrl+S`, or discard to restore the last saved
  palette. Custom color definitions remain editable in TOML.
- `xrat validate` identifies invalid colors and unknown fields precisely.
  Built-in names and aliases are reserved; custom themes inherit built-ins only.
- Saving Settings preserves custom definitions and comments and supports the
  legacy `[ui]` section.

### Theme fixes

- RGB palettes now paint their intended backgrounds and use explicit
  selected-row foreground/background pairs. Muted and status colors have
  improved contrast.
- Panels, modals, scrollbars, and release-note Markdown follow the active
  palette.
- Status colors follow actual config state; mixed engine availability is colored
  per engine rather than showing the entire summary as successful.
- Discarding changes after saving restores the most recently saved theme.

### Upgrade notes

No database migration is required. Existing built-in names and aliases remain
accepted. RGB themes now paint their own background; choose `ansi` or define a
custom `background = "terminal"` to retain your terminal background.

The legacy `solarized-light` alias still selects the Latte-inspired `light`
palette but is deprecated; use `light` or `latte`.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.28.0...v0.29.0

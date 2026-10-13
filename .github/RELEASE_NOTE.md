## xrat v0.28.0

Choose a TUI color theme and preview it live in Settings. Tables and detail views
also use clearer colors for status, protocols, latency, and selected rows.

### TUI themes

- Eight built-in palettes: `default`, `ansi`, `catppuccin`, `dracula`, `gruvbox`,
  `nord`, `tokyo-night`, and `light`.
- Open Settings with `,`, navigate to TUI → Theme, and cycle palettes to preview
  them immediately. Save to keep the theme; discarding changes restores the saved
  palette.
- Set the theme directly in config.toml:

  ```toml
  [tui]
  theme = "catppuccin"
  ```

- The `[ui]` section is accepted as an alias for `[tui]`. Theme values are checked
  by `xrat validate` and supported by the interactive config editor.
- Latency values use green, yellow, and red to distinguish response times, while
  missing values and unfocused borders are muted. Selected rows retain semantic
  cell colors with a background highlight.

### Upgrade notes

No database migration or configuration changes are required. Existing
configurations keep the default warm amber theme unless another theme is selected.

Thanks to @f02xygen for the theme switching and styling improvements in #218.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.27.0...v0.28.0

# Browser interactions

UI state and browser interactions are implemented in Rust/WASM. `App` provides
one `Interactions` coordinator. Components retain their Dioxus-owned DOM nodes;
floating content uses native manual popovers when supported.

| Module           | Responsibility                                                                 | Consumers                     |
| ---------------- | ------------------------------------------------------------------------------ | ----------------------------- |
| `coordinator.rs` | Active popup/tooltip, overlay stack, dismissal priority                        | App and UI primitives         |
| `listeners.rs`   | Outside click/focus, Escape/Tab, resize and captured scroll; listener teardown | Coordinator                   |
| `floating.rs`    | Mounted controls, popup keyboard navigation, ARIA state and hover cleanup      | Select, DropdownMenu, Tooltip |
| `placement.rs`   | Popup flip/clamping and tooltip positioning                                    | Floating controls             |
| `overlay.rs`     | Modal focus, inert background, scroll lock and focus return                    | Dialog, Sheet, AlertDialog    |
| `dom.rs`         | Typed mounted elements, DOM queries, focus and popovers                        | Interaction helpers           |

Components render `data-state`, `inert`, and pointer state from their open
signals. Browser helpers set measured geometry and native popover state. There
are no document-wide mutation observers, attribute-trigger dispatchers, inline
scripts or synthetic close-button bridges. Unmount cleanup dismisses active
controls and restores modal isolation. `on_close` callbacks carry `()` for all
dismissal paths, including keyboard dismissal.

Application browser interactions live with their owners:

- `components/clipboard.rs`: `CopyButton` and copy feedback, secure clipboard
  access, selection/copy fallback, and lifecycle-owned feedback timers.
- `hooks/use_import_paste.rs`: one document paste listener while the authorized
  import controller is mounted; respects native editable paste, prevented
  events, and the coordinator's overlay state.
- `components/log_console.rs`: mounted output scrolling.
- `pages/overview/components/chart_navigation.rs`: chart-local SVG focus
  movement.
- `ui/sidenav_resize.rs`: pointer-captured resizing, stored width, and cleanup
  on cancellation/unmount; used by `SidenavResizeHandle` in `ui/sidenav.rs`.
- `ui/toast.rs`: delegates notification requests to the existing Rust feedback
  state instead of evaluating JavaScript or creating dummy DOM buttons.

`just web-build` removes obsolete packaged controller assets. Bootstrap and the
offline-only service worker remain separate from UI interactions. Preserve CSP
and keep browser dependencies behind the `ui` feature.

`app/pwa.rs` registers the worker from Rust on supported secure origins after
page load, including when WASM starts after the load event. Its pending load
listener is removed on unmount. Registration failures remain nonfatal warnings;
the offline-only worker itself stays in `assets/sw.js`.

Validate with `just web-check`, `cargo test --locked -p xrat-web`, the
`--no-default-features` library check, and `just web-build`. Native
state/geometry tests do not prove browser focus, hover, keyboard, clipboard,
modal isolation, or mobile behavior; those require browser checks.

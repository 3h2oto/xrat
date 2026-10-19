# Browser controllers

`boot.js` loads `components.js`, which imports these native ES modules once,
before starting Dioxus. No bundler or inline scripts are needed.
`just web-build` copies this directory into the assets embedded and served by
Axum.

Paths below are relative to `crates/xrat-web/src/`. A component opts into a
controller by rendering the listed attributes. Composing one of these primitives
also inherits its controller dependency; most other UI primitives need no JS
controller here.

| Controller                | Rust owners / consumers                                                                                                  | DOM contract and behavior                                                                                                                                                                                                                                                                              |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `ui/popups.js`            | `ui/select.rs`, `ui/dropdown_menu.rs`; composed by `ui/form_select.rs` and `ui/multi_select.rs`                          | `data-select-trigger`, `data-select-option`, `data-dropdown-trigger`, `data-dropdown-close`, `data-hover-menu`; `SelectContent` / `DropdownMenuContent` names. Placement, click/hover opening, keyboard navigation, outside closing, ARIA state, and remount cleanup. Checkbox menu items remain open. |
| `ui/overlays.js`          | `ui/dialog.rs`, `ui/sheet.rs`; `ui/alert_dialog.rs` wraps Dialog                                                         | `data-dialog-trigger/close`, `data-sheet-trigger/close`, `data-auto-open`, `data-overlay-notify-close`, `data-sheet-navigation`; `DialogContent` / `SheetContent` and matching backdrop names. Focus trapping/return, Escape, backdrop closing, scroll locking, and inert background isolation.        |
| `ui/tooltips.js`          | `ui/tooltip.rs`                                                                                                          | `data-name="Tooltip"`, child `role="tooltip"`, `data-position`. Hover/focus placement and dismissal.                                                                                                                                                                                                   |
| `app/import-paste.js`     | `components/import.rs`; paste markers in workspace config/subscription tables                                            | `data-import-paste="true"` enables the global paste listener, which emits `xrat-import-paste`. Editable targets and open overlays retain native paste.                                                                                                                                                 |
| `app/clipboard.js`        | `components/log_console.rs`, Overview listener, Account API tokens, Tools editor, workspace sharing/subscription dialogs | `data-copy-textarea="<target-id>"`. Copies textarea values or element text, with a fallback and temporary success/failure label.                                                                                                                                                                       |
| `app/log-console.js`      | `components/log_console.rs`                                                                                              | `data-log-top="<target-id>"`. Scrolls that log container to the top.                                                                                                                                                                                                                                   |
| `app/chart-navigation.js` | `pages/overview/components/charts.rs`, `pages/overview/components/latency_chart.rs`                                      | `.chart-sample` SVG groups. Left/Right/Home/End move sample focus.                                                                                                                                                                                                                                     |

`dom.js` contains the shared typed DOM lookup helpers. `ui/floating.js` owns the
active popup, trigger, and tooltip references, native popover helpers, and
closing operations. This shared state lets opening an overlay dismiss floating
controls and prevents tooltips from covering an open menu without circular
imports. Popup listeners also handle shared floating dismissal on Escape,
scroll, resize, and DOM removal. Import/paste only reads active overlay state.

Listeners are delegated to `document` or `window`, so Dioxus can remount
components without attaching more listeners. Keep controller imports in
`components.js`: chart navigation precedes popup keyboard handling, and popup
Escape handling precedes overlays so one Escape closes only the top control.
Keep business logic and controlled component values in Rust; these modules own
browser interactions only.

When adding a controller dependency, update this table with its Rust owner and
DOM contract. If removing a module, check both direct attributes and components
that compose its primitives.

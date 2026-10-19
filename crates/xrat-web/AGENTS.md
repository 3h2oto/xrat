# XRAT Web Guidelines

These instructions extend the repository `AGENTS.md` for `crates/xrat-web/`.
Preserve the root guide's shared-service and CLI/TUI/Web consistency rules.

## Module Ownership

This crate contains both a Dioxus frontend binary and a browser-safe library
used by Axum. Keep that boundary explicit:

- `src/main.rs` launches the app; `src/app.rs` wires providers and the session
  boundary. Session restoration and session context belong in
  `src/app/session.rs`.
- `src/api/` owns HTTP transport, action dispatch, and job request helpers. Keep
  operation semantics in the shared application services behind the API.
- `src/types/` owns general serialized API contracts; `src/configs/` owns shared
  pure config values, matching, ordering, and config-sharing contracts and
  logic. These library modules must compile with `--no-default-features` and
  have no browser/UI dependencies.
- `src/pages/` owns routed screens and their page-specific composition.
- `src/components/` owns reusable application components, including log output,
  import/paste handling, status presentation, clipboard helpers, and QR
  rendering.
- `src/ui/` owns low-level UI primitives such as buttons, dialogs, tooltips,
  inputs, and tables. Reuse these primitives before introducing another variant.
- `src/hooks/`, `src/routing/`, and `src/shell.rs` own shared hooks, navigation,
  and the application shell respectively.

Place new code by responsibility. Do not accumulate domain helpers at the crate
root or put non-visual services and DTOs into `components/`. Keep `mod.rs`
declarative, and update all consumers, including Axum, when moving library APIs.

## Page Structure and Loading

Use this structure for routed pages:

```text
pages/<page_name>/
  mod.rs
  page.rs
  suspend.rs
  components/          # Only when the page needs multiple local components
    mod.rs
    <component_name>.rs
```

- `page.rs` contains the page entry component and composition. `mod.rs` declares
  modules and re-exports the entry point.
- `suspend.rs` provides the page's loading view. Match the actual cards, table,
  and controls with animated skeletons to preserve layout during loading;
  respect reduced-motion preferences. A generic spinner is not a substitute for
  a page-specific skeleton.
- Do not create a `components/` folder merely to wrap a single `mod.rs`. Use a
  named sibling file for a small page-local component when appropriate.
- Extract duplicated components and behavior into `src/components/` or the
  appropriate shared service module. For example, reuse the shared log console
  instead of maintaining separate log textareas across pages.

## Interaction and Visual Conventions

Follow the existing design tokens, theme handling, and responsive breakpoints.
Keep desktop and mobile layouts usable in both light and dark themes.

- Keep action controls compact. Use `IconButton` for table-row and bulk actions;
  retain accessible names and tooltips, including eligible counts where useful.
  Use meaningful state icons with accessible text for jobs and daemon status.
- Show desktop row actions directly. Row menus are appropriate on mobile. Show
  bulk actions directly without a bulk menu, selection-clear cross, or "Select
  all matching" button. Retain row/header checkbox selection and
  destructive-action confirmation.
- Do not reintroduce Configs export buttons. Config sharing uses copy, QR, and
  subscription-link actions; preserve other pages' explicitly supported exports.
- Clearly distinguish the currently connected config with its row color.
  Connection state comes from the server and is separate from checkbox
  selection.
- The Overview's two desktop rows have equal height. Show runtime information in
  its runtime card rather than restoring the redundant runtime modal.
- Keep the connection-detail modal compact and consistent with existing dialogs.
  Avoid redundant counts, raw content, or explanatory copy around a focused
  action.
- Global paste/import should work across pages. Do not hijack Ctrl/Cmd+V in
  editable fields or interfere with native paste and keyboard navigation.
- Include meaningful loading, empty, error, disabled, and success states. A
  queued operation is not a completed operation; expose job progress/errors
  through the existing feedback and Jobs flow.

## Config Sharing and Credential Boundaries

- Selected sharing operates on explicit checked IDs. Filtered sharing includes
  all matches across pages and follows every active filter exactly, including
  enabled/deleted visibility and ordering. Reuse shared matching logic.
- Copy and raw QR actions use complete original config links. Never silently
  truncate, split into multiple QR codes, or substitute a different payload.
- Disable QR actions when the complete payload exceeds supported capacity. The
  server preview and renderer must use the same error-correction setting. Offer
  subscription links for larger filtered sets.
- QR dialogs show the QR and download action without raw-config textareas or the
  redundant count/scope and snapshot captions. Preserve the copy-failure
  fallback in the copy flow rather than moving it into the QR modal.
- Embed the existing XRAT logo in displayed and downloaded QR assets, with a
  transparent logo background and no white center patch. Preserve the QR's white
  background, quiet zone, and high error correction. Verify changed rendering
  with an independent decoder at display size; do not imply phone-camera testing
  unless it was actually performed.
- `/b64` accepts legacy `key=` and Account `token=` credentials. A supplied
  invalid token must not fall back to legacy or anonymous access.
- Account-token mode fetches the current user's read-only tokens for selection.
  If none exist, link to Account and explain creation, return, reload, and
  selection. Do not ask users to paste an unrecoverable existing token secret.
- Token secrets are hash-only on the server. Use the existing subscription-only
  credential issuance tied to the selected read-only token; preserve parent
  ownership, scope, credential-version, and revocation checks. Such credentials
  must not authenticate management APIs.
- Keep real credential-bearing URLs and raw configs out of broad Overview
  responses, logs, persistent browser storage, fixtures, and third-party QR
  services. Keep share credentials transient in the dialog and enforce
  authorization on the server; hiding a button is not an access-control
  boundary.

## Verification and Delivery

- Run `just web-check` for frontend compile/lint checks and
  `cargo test --locked -p xrat-web` for existing contracts and component
  helpers. For library changes, also check
  `cargo check --locked -p xrat-web --no-default-features` and compile/test
  affected server consumers.
- Run focused backend tests for changed auth, filters, actions, or lifecycle
  behavior. Exercise both SQLite and PostgreSQL when changing persistence. Do
  not add implementation-mirroring tests for trivial visual edits.
- Inspect meaningful visual/interaction changes in the browser on desktop and
  mobile. Check overflow, keyboard/accessibility, relevant theme states, and
  browser errors. Use isolated dummy data for secrets/configs and runtime
  effects.
- Build with `just web-build` before `just release` when producing an installed
  binary: browser assets are embedded by the native build. Do not claim source
  changes or a frontend build have updated an already running daemon.
- When installation is requested, finish the build, install the verified binary,
  restart the intended service, and verify its executable and served assets. Do
  not report installation as complete while a build is pending. Report the
  boundary between local tests, browser checks, installation, and remote
  CI/release.

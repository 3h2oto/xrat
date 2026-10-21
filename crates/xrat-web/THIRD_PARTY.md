# UI provenance

Components in `src/ui/` are copied and adapted from Rust/UI's Dioxus registry,
pinned to revision `7120636e68556ddd023a4ec8e083663c7b745318`:
[Rust/UI component registry](https://github.com/rust-ui/ui/tree/7120636e68556ddd023a4ec8e083663c7b745318/app_crates/registry/src/ui)

The source families include Select, DropdownMenu, Sidenav, Sheet, Dialog,
AlertDialog, Tooltip, Avatar, Button, Input, Textarea, Field, Label, Table,
Card, Badge, Alert, Empty, Skeleton, Spinner, Separator, ScrollArea, and Toast.
Local adapters (`form_select`, `sidebar`, `states`, `feedback`, `history`, and
`log_console`) compose them with XRAT state and API behavior.

Local compatibility patches add controlled values, accessible labels, stable
IDs, event handlers, and modal close notifications. Router links use plain
anchors. UI primitive scripts are replaced by Rust interaction state and typed
browser bindings under `src/ui/interaction/` to preserve XRAT's Content Security
Policy. These helpers supply popup positioning, hover/focus menus, keyboard
navigation, focus restoration, modal isolation, and tooltip placement. Their
README maps ownership and lifecycle rules. Floating content uses manual native
popovers when available to escape ancestor clipping and transformed modal
coordinates without moving Dioxus-owned DOM nodes. Shared CSS layer tokens order
page chrome, feedback, backdrops, and modal panels. Theme tokens use XRAT's
existing logo palette. Icons come from Rust/UI's `icons` crate (Dioxus feature).

Application browser helpers now use Rust handlers: copy controls and feedback,
global paste/import, log scrolling, and chart keyboard navigation. Paste/import
checks the shared Rust overlay state rather than inspecting DOM markers.

The login page adapts the split layout and password visibility pattern from
Rust/UI's Login02 block:
[Login02 source](https://github.com/rust-ui/ui/blob/main/app_crates/registry/src/blocks/login02.rs)

XRAT supplies its existing authentication flow and local logo assets.
Unavailable social login, password recovery, and signup actions are omitted.

## Rust/UI license

MIT License

Copyright (c) 2026 Max Wells

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

## Sidebar and icons

The sidebar adapts Rust/UI's Sidenav07 icon-rail structure and Sidenav component
patterns, with XRAT navigation and responsive folding behavior:
[Sidenav source](https://github.com/rust-ui/ui/blob/main/app_crates/registry/src/ui/sidenav.rs)

Navigation, collapse controls, and sun/moon theme controls use the official
Rust/UI `icons` crate with its Dioxus feature:
[Rust/UI icons](https://rust-ui.com/icons). The Rust/UI MIT license above
applies to the adapted sidebar patterns.

## Select

The shared controlled Select adapts Rust/UI's
Select/Trigger/Value/Content/Group/ Option anatomy, styling, and
ChevronDown/Check icons:
[Select source](https://github.com/rust-ui/ui/blob/main/app_crates/registry/src/ui/select.rs)

XRAT handles visibility and selection in Dioxus signals, with keyboard
navigation and controlled values for forms. The Rust/UI MIT license above
applies.

## Overview charts

Overview follows Rust/UI's area/bar/pie chart examples and Card/Select anatomy:
[Chart examples](https://github.com/rust-ui/ui/tree/main/app_crates/registry/src/charts)
[Charts source](https://github.com/rust-ui/ui/blob/main/app_crates/registry/src/ui/charts.rs)

The local `pages/overview/charts.rs` renderer uses Dioxus-owned SVG instead of
Rust/UI's JavaScript chart renderer. It supplies timestamp-aware rates, gaps,
keyboard sample inspection, and XRAT theme colours without another runtime chart
dependency. The dashboard calculations and SVG renderer are XRAT code; the
existing copied/adapted controls retain the Rust/UI MIT notice above.

Average latency adapts the column layout and data attributes from Rust/UI's
[BarChart01 example](https://github.com/rust-ui/ui/blob/7120636e68556ddd023a4ec8e083663c7b745318/app_crates/registry/src/charts/bar_chart_01.rs).
Its local SVG renderer adds measured millisecond values, missing-result states,
and sample-count inspection without the upstream ApexCharts runtime.

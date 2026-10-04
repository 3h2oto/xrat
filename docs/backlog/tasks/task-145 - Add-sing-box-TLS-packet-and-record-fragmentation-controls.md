---
id: TASK-145
title: Add sing-box TLS packet and record fragmentation controls
status: To Do
assignee: []
created_date: '2026-10-03 09:34'
updated_date: '2026-10-04 17:35'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies: []
references:
  - 'https://throneproj.github.io/advanced/presets/'
  - 'https://sing-box.sagernet.org/configuration/shared/tls/'
  - >-
    https://hiddify.com/manager/basic-concepts-and-troubleshooting/How-the-TLS-Trick-works-and-its-usage/
  - 'https://xtls.github.io/en/config/outbounds/freedom.html'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 134000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

XRAT already supports the Xray freedom fragment helper. The missing surface is modern sing-box native TLS fragmentation and record fragmentation. Do not describe Xray range/packet settings as exact equivalents of the sing-box options.

sing-box 1.12+ outbound TLS: {"tls":{"enabled":true,"record_fragment":true}} or fragment=true with fragment_fallback_delay="500ms"; preserve the original server_name, validation and transport. Packet fragmentation and TLS-record splitting are distinct modes.

Xray retains the existing freedom settings.fragment (packets/length/interval) plus sockopt.dialerProxy helper. It has no demonstrated equivalent of sing-box tls.record_fragment in the consulted mappings. V2Ray has no demonstrated equivalent of this Xray helper/modern sing-box TLS surface; explicit unsupported diagnostics are required.

Scope is these documented stock-core fields, not Hiddify fork-only TLS padding or automatic successful censorship circumvention. Test supported TCP TLS transports; QUIC and REALITY are not automatically compatible.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Typed engine-specific packet/record modes and delay settings reject unsupported engines/transports and preserve defaults.
- [ ] #2 Generated sing-box 1.13 fixtures pass native validation; local TLS tests prove handshakes still succeed and record/packet settings are applied as documented.
- [ ] #3 Managed/probe settings and profile overrides agree; docs distinguish existing Xray fragment ranges from sing-box native fields and Hiddify fork features.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

---
id: TASK-144
title: Add modern sing-box multiplex protocol and padding settings
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
  - >-
    https://raw.githubusercontent.com/2dust/v2rayN/master/v2rayN/ServiceLib/Models/Configs/ConfigItems.cs
  - 'https://throneproj.github.io/advanced/presets/'
  - 'https://sing-box.sagernet.org/configuration/shared/multiplex/'
  - 'https://xtls.github.io/en/config/outbound.html'
  - 'https://www.v2fly.org/en_US/config/outbounds.html'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 133000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

XRAT's runtime mux settings/generator are Xray-oriented (concurrency and XUDP). sing-box generation does not expose its own modern multiplex configuration. Add separate typed settings, not a lossy translation of Xray concurrency to arbitrary sing-box fields.

sing-box outbound fragment: {"multiplex":{"enabled":true,"protocol":"smux","max_streams":8,"padding":false}}. max_streams conflicts with max_connections/min_streams. Supported outbound protocols and server padding support must be verified; no universal mux toggle across QUIC and TCP.

Xray: existing outbounds[].mux.enabled/concurrency/xudpConcurrency/xudpProxyUDP443 remains its own policy. V2Ray: existing compatible mux.enabled/concurrency remains separate; XUDP features require explicit compatibility proof.

Protocol choices smux/yamux/h2mux and padding change the wire protocol and need a compatible server. TCP Brutal is a separate advanced feature and is not included implicitly. Preserve disabled defaults and integrate inheritance with TASK-71.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Modern sing-box protocol/stream-limit/padding options generate native-valid configs with conflict and outbound-capability validation.
- [ ] #2 Local client/server tests demonstrate compatible multiplex and padding plus actionable incompatible-server/protocol behavior.
- [ ] #3 Xray/V2Ray mux behavior remains unchanged and documentation distinguishes the wire protocols and unsupported combinations.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

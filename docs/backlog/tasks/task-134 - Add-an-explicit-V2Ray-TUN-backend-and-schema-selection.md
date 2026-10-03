---
id: TASK-134
title: Add an explicit V2Ray TUN backend and schema selection
status: To Do
assignee: []
created_date: '2026-10-03 09:33'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies:
  - TASK-131
references:
  - 'https://www.v2fly.org/en_US/config/service/tun.html'
  - 'https://www.v2fly.org/en_US/config/transport.html'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 116000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

V2Fly documents a native TUN service from v5.9.0 on Linux amd64/arm64. XRAT's current generated V2Ray configuration follows the legacy compatible shape; establish an explicit v5 config adapter or a supervised helper path rather than silently changing every V2Ray configuration.

V2Ray service.tun payload fields: name, mtu, tag, ips:[{ip:[172,19,0,1],prefix:30}], routes:[{ip:[0,0,0,0],prefix:0}], enablePromiscuousMode, enableSpoofing, packetEncoding and (v5.11+) sniffingSettings. The outer v5 services wrapper and binary build support must be pinned by native fixtures before implementation. Its docs require OS address/route setup too.

Xray uses protocol=tun/settings.gateway/autoSystemRoutingTable. sing-box uses type=tun/address/auto_route. A helper alternative adds a local SOCKS outbound/inbound bridge plus helper config and OS routes; there is no single universal JSON TUN toggle across engines.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A documented backend decision selects native v5 service or an explicit supervised helper for each supported V2Ray build/platform, with validated complete examples.
- [ ] #2 Legacy proxy generation remains compatible; TUN's selected schema and unsupported version/platform errors are explicit.
- [ ] #3 TCP/UDP, loop avoidance, privileges, route setup/restoration and managed helper/core cancellation are demonstrated in integration fixtures.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

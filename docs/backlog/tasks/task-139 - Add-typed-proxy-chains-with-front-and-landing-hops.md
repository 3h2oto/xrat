---
id: TASK-139
title: Add typed proxy chains with front and landing hops
status: To Do
assignee: []
created_date: '2026-10-03 09:33'
updated_date: '2026-10-03 09:39'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies: []
references:
  - 'https://throneproj.github.io/advanced/chains/'
  - 'https://github.com/2dust/v2rayN/wiki'
  - 'https://xtls.github.io/en/config/transports/sockopt.html'
  - 'https://www.v2fly.org/en_US/config/outbounds.html'
  - 'https://sing-box.sagernet.org/configuration/shared/dial/'
  - >-
    https://raw.githubusercontent.com/v2fly/v2ray-core/master/infra/conf/cfgcommon/proxycfg/proxy.go
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: high
ordinal: 121000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

Clients support device -> front proxy -> chosen proxy -> landing proxy -> destination. XRAT's special fragment helper is already chained, but arbitrary stored-node chains and their lifecycle are missing. Persist typed node references/hop order, not root JSON.

Xray transport-level chain: landing outbound streamSettings.sockopt.dialerProxy="front", with unique generated tags; apply to each downstream hop. Validate transport/UDP capabilities. Existing fragmentation also sets dialerProxy, so composition must be deliberate. happyEyeballs on an outbound with dialerProxy is documented as ineffective; egress dialing belongs to the real first hop.

V2Ray: documented outbounds[].proxySettings.tag="front" is protocol forwarding with different streamSettings behavior; current upstream proxycfg source also exposes proxySettings.transportLayer=true for transport-layer forwarding. Select that mode only after native validation against XRAT's supported V2Ray binaries. Never promise arbitrary TLS transport chains based only on proxySettings.

sing-box: downstream outbound {"detour":"front"} and ordinary tagged upstream outbounds. Other dial fields are ignored with detour; attach binding/bootstrap/socket policies to the actual dialer, not every hop.

Cross-engine bridges, nested chain cycles and incompatible UDP hops must be rejected or represented as explicit managed helper dependencies. WARP composition reuses TASK-112.1 WireGuard once that support exists.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Two- and three-hop chains preserve documented device-to-exit order, unique references and exit routing across supported protocols.
- [ ] #2 Cycles, disabled/deleted/missing hops and incompatible transport/UDP/fragment/IPv6 combinations fail before launch.
- [ ] #3 Native validation and local multi-hop integration tests prove hop order, DNS path, teardown and managed/probe tuning consistency.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

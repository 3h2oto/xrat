---
id: TASK-132
title: Add supervised sing-box TUN capture and route cleanup
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
  - 'https://sing-box.sagernet.org/configuration/inbound/tun/'
  - 'https://hiddify.com/app/How-to-use-Hiddify-app/'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: high
ordinal: 114000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

Deliver the sing-box backend of the TUN contract, including startup, readiness, disconnect, crash recovery and restoration of only XRAT-owned network changes.

sing-box inbound fragment: {"type":"tun","tag":"xrat-tun","interface_name":"xrat0","address":["172.19.0.1/30"],"mtu":1500,"auto_route":true,"strict_route":true,"stack":"system"}. Add route.auto_detect_interface=true where supported. CIDRs are illustrative and require collision checks. Linux auto_redirect is a separate opt-in supported platform feature, not portable.

Xray mapping remains inbounds[].protocol="tun"/settings; V2Ray mapping remains v5 service.tun. This task selects neither backend and must reject a mismatched engine. DNS interception is tracked separately; strict_route must not be advertised as a persistent fail-closed firewall. See contract for platform gates and JSON differences.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Managed CLI/TUI/daemon startup can capture TCP and UDP with sing-box while preserving ordinary local listeners and preventing proxy/DNS routing loops.
- [ ] #2 MTU, IPv4/IPv6 addresses, include/exclude CIDRs and stack settings validate before OS mutation; privileges and unsupported platforms produce actionable errors.
- [ ] #3 Integration evidence covers direct/proxy traffic, excluded LAN, cancellation, core crash and disconnect with owned route/interface cleanup.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

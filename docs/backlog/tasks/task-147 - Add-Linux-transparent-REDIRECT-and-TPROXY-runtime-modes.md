---
id: TASK-147
title: Add Linux transparent REDIRECT and TPROXY runtime modes
status: To Do
assignee: []
created_date: '2026-10-03 09:34'
updated_date: '2026-10-04 17:35'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies:
  - TASK-131
  - TASK-135
references:
  - 'https://github.com/v2rayA/v2rayA'
  - 'https://xtls.github.io/en/config/transports/sockopt.html'
  - 'https://www.v2fly.org/en_US/config/transport.html'
  - 'https://xtls.github.io/en/config/inbounds/tunnel.html'
  - 'https://sing-box.sagernet.org/configuration/inbound/tproxy/'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 136000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

v2rayA exposes Linux transparent capture for host/router traffic. XRAT lacks a managed capture mode that owns firewall/policy-routing setup. Keep this independent of TUN, ordinary system proxy and WireGuard; begin with a declared Linux scope.

Xray/V2Ray compatible transparent inlet: {"protocol":"dokodemo-door","tag":"xrat-transparent","settings":{"network":"tcp,udp","followRedirect":true},"streamSettings":{"sockopt":{"tproxy":"tproxy"}}}. Current Xray may use the newer tunnel alias; pin accepted spelling per version. Add loop-avoidance egress mark/interface and policy routing. REDIRECT and TPROXY differ in TCP/UDP/original-destination behavior.

sing-box inbound fragment: {"type":"tproxy","tag":"xrat-transparent","listen":"127.0.0.1","listen_port":12345,"network":"tcp"}; add explicitly supported UDP listener/config. REDIRECT has a separate inbound type and platform contract.

iptables/nftables rules, fwmarks, ip rules/routes and cleanup are OS operations, not JSON fields. Record ownership and rollback; do not flush unrelated firewall rules. No default persistent kill switch is implied.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A documented Linux mode matrix specifies TCP/UDP, host versus forwarded LAN traffic, privileges and REDIRECT versus TPROXY limitations per engine.
- [ ] #2 Native-valid inlet/egress JSON plus isolated network-namespace tests prove original destination handling, DNS policy and no recursive capture.
- [ ] #3 Partial-start rollback, crash recovery and shutdown remove only XRAT-owned firewall/routing changes, with observable managed status/events.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

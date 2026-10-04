---
id: TASK-131
title: Define managed TUN settings and engine capability contract
status: To Do
assignee: []
created_date: '2026-10-03 09:33'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies: []
references:
  - >-
    https://raw.githubusercontent.com/2dust/v2rayN/master/v2rayN/ServiceLib/Models/Configs/ConfigItems.cs
  - 'https://xtls.github.io/en/config/inbounds/tun.html'
  - 'https://www.v2fly.org/en_US/config/service/tun.html'
  - 'https://sing-box.sagernet.org/configuration/inbound/tun/'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: high
ordinal: 113000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

v2rayN, v2rayNG, Throne and Hiddify expose VPN/TUN capture, MTU and address-family controls. XRAT RuntimeSettings and SingboxInbound expose proxy listeners but no managed TUN settings. Define typed capture settings, separate OS ownership from engine config, and establish supported version/build/platform combinations before adding backends.

| Engine | JSON effect |
|---|---|
| Xray | inbounds[].protocol="tun"; settings.name, mtu, gateway, autoSystemRoutingTable, autoOutboundsInterface; settings.dns is Windows-only in current docs. |
| V2Ray | v5 service.tun uses name, mtu, tag, ips and routes plus sniffingSettings; v5 JSON service format is distinct from XRAT's existing v4-style generation. Linux amd64/arm64 documented, introduced v5.9.0. Do not insert an invented protocol=tun into the legacy schema. |
| sing-box | inbounds[].type="tun"; interface_name, address, mtu, stack, auto_route, strict_route, route_address, route_exclude_address; route.auto_detect_interface or explicit binding avoids loops. |

Sing-box address/route field migrations and 1.14+ DNS-mode additions require gates. Xray native TUN is present in current upstream documentation; confirm availability in supported releases instead of assuming all releases support it. Android VPN APIs are client integration, not a desktop JSON field. A strict routing flag does not alone guarantee a kill switch.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A version/build/platform matrix specifies native TUN versus an explicit helper backend or unsupported result for all three engines.
- [ ] #2 Typed settings define IPv4/IPv6 capture, MTU, routes/exclusions, DNS ownership, privileges, cleanup and interaction with system proxy; defaults preserve current proxy mode.
- [ ] #3 Concrete native-validator fixtures establish accepted schemas and minimum supported binaries; no privileged capture is introduced into test/scan flows.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

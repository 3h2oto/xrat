---
id: TASK-130
title: Prefer IPv6 for proxy-server connections with IPv4 fallback
status: To Do
assignee: []
created_date: '2026-10-02 19:17'
updated_date: '2026-10-03 09:37'
labels:
  - feature
  - network
  - runtime
milestone: m-8
dependencies: []
references:
  - 'https://xtls.github.io/en/config/transports/sockopt.html'
  - crates/xrat-app/src/app/services/runtime_tuning/singbox.rs
  - crates/xrat-engines/src/xray/config/types.rs
documentation:
  - docs/src/05-reference/config-file.md
priority: high
ordinal: 112000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add an opt-in connection address-family preference for proxy server hostnames: try IPv6 first and fall back to IPv4 when IPv6 is unavailable or connection attempts fail. The user explicitly requested the proxy-server connection behavior familiar from Android clients such as v2rayNG. This is distinct from DNS query_strategy=UseIPv6, which restricts DNS results and does not provide the requested connection fallback. Apply the setting consistently to supported managed engines (Xray/V2Ray and sing-box) and proxy-based test/scan flows. Preserve current behavior when unset. Verify engine-version compatibility before selecting socket/dialing controls; DNS must allow both families when preference is enabled, or contradictory explicit settings must produce a clear validation error.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A documented opt-in setting prefers IPv6 when dialing a proxy server hostname while preserving current behavior when unset.
- [ ] #2 IPv4 fallback succeeds when a dual-stack hostname has unreachable IPv6, when the client has no IPv6 route, and when the hostname only has IPv4 addresses; connection failure must not be mistaken for successful DNS fallback.
- [ ] #3 Supported Xray/V2Ray and sing-box managed sessions and proxy-based test/scan paths honor the same preference; unsupported engine versions or conflicting DNS settings produce actionable validation errors.
- [ ] #4 Literal IPv4 and IPv6 proxy-server addresses remain usable; the setting does not unintentionally change proxied destination DNS policy.
- [ ] #5 Regression tests cover generated engine configuration and real local dialing/fallback behavior; just fmt ci passes.
- [ ] #6 Configuration reference and editor help explain IPv6 preference, IPv4 fallback, defaults, DNS interactions, and engine compatibility.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
### Runtime parity research (2026-10-03; no implementation)
V2rayNG exposes Prefer IPv6 separately from VPN IPv6 capture; v2rayN now exposes Happy Eyeballs controls. This task remains High priority.

| Engine | Candidate JSON / required proof |
|---|---|
| Xray | Proxy egress streamSettings.sockopt: {"domainStrategy":"UseIP","happyEyeballs":{"prioritizeIPv6":true,"tryDelayMs":250,"interleave":1,"maxConcurrentTry":4}}. Native validation against supported binaries is required. |
| V2Ray | The consulted current socketcfg schema has no Xray happyEyeballs/domainStrategy fields. Do not copy them. Establish a compatible native dial policy or a supervised/address-selection mechanism preserving original SNI/Host; unsupported builds must fail clearly. |
| sing-box | For the supported 1.13 family, candidate proxy outbound.domain_resolver={"server":"bootstrap","strategy":"prefer_ipv6"}; resolver contains both families. Pin actual dialing behavior with dual-stack integration tests. Legacy domain_strategy is deprecated since 1.12 and removed in 1.14. |

Xray UseIPv6v4 falls back after DNS failure/empty answers, not after an IPv6 connection fails. Its Happy Eyeballs is TCP-only and cannot take effect on an outbound with dialerProxy; apply policy at the real egress and test existing fragment helpers/chains. UDP/QUIC failure fallback requires a distinct supported mechanism or explicit limitation. sing-box fallback_delay/network_strategy documentation restricts these controls to integrated graphical clients on Android/Apple; do not assume they are usable in XRAT desktop. DNS restriction UseIPv6 is still not this feature.

Sources: [v2rayNG settings](https://raw.githubusercontent.com/2dust/v2rayNG/master/V2rayNG/app/src/main/res/values/strings.xml), [v2rayN model](https://raw.githubusercontent.com/2dust/v2rayN/master/v2rayN/ServiceLib/Models/Configs/ConfigItems.cs), [Xray socket options](https://xtls.github.io/en/config/transports/sockopt.html), [V2Ray socket schema](https://raw.githubusercontent.com/v2fly/v2ray-core/master/infra/conf/cfgcommon/socketcfg/socket.go), [sing-box dial](https://sing-box.sagernet.org/configuration/shared/dial/). Related: TASK-135, TASK-139, doc-1.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

---
id: TASK-130
title: Prefer IPv6 for proxy-server connections with IPv4 fallback
status: To Do
assignee: []
created_date: '2026-10-02 19:17'
labels:
  - feature
  - network
  - runtime
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

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

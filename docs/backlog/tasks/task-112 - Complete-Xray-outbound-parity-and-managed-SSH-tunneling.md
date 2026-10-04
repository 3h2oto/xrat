---
id: TASK-112
title: Complete Xray outbound parity and managed SSH tunneling
status: To Do
assignee: []
created_date: '2026-09-02 10:59'
updated_date: '2026-10-04 17:35'
labels:
  - protocols
  - xray
  - ssh
  - runtime
  - initiative
milestone: m-8
dependencies: []
references:
  - 'https://xtls.github.io/en/config/outbounds/'
  - TASK-101
  - TASK-107
documentation:
  - AGENTS.md
  - docs/src/06-architecture/import-pipeline.md
  - docs/src/06-architecture/config-generation.md
  - docs/src/06-architecture/runtime-lifecycle.md
  - docs/src/05-reference/protocols.md
priority: medium
ordinal: 124000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Track the remaining operational protocol work identified by comparing Xray's current outbound catalog with Xrat's normalized import and managed-runtime pipelines. This initiative covers WireGuard, native Xray Hysteria v2, DNS and Loopback routing outbounds, and first-class managed SSH dynamic forwarding. Existing Freedom and Blackhole generation is already implemented; existing sing-box Hysteria2 correctness remains tracked by TASK-101.

Required reading before implementation of any child task:
- AGENTS.md and the repository guidelines it includes
- docs/src/06-architecture/import-pipeline.md
- docs/src/06-architecture/config-generation.md
- docs/src/06-architecture/runtime-lifecycle.md
- docs/src/05-reference/protocols.md
- The protocol-specific upstream documents attached to that child task

Scope rule: schema deserialization alone does not count as support. A protocol is complete only when accepted input is normalized and persisted without silent loss, generated for the intended engine, validated before launch, observable through runtime status/events, tested, and documented. Do not invent unofficial share-link formats without an explicit documented contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each missing outbound or tunnel capability is represented by a review-sized child task with explicit required reading
- [ ] #2 The support matrix distinguishes schema parsing, import support, persistence, Xray generation, sing-box generation, and managed runtime support
- [ ] #3 Unsupported or lossy inputs fail before process launch with actionable errors
- [ ] #4 Protocol work updates user documentation and passes focused tests plus just fmt ci
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
### Runtime parity research (2026-10-03; no implementation)
This initiative contributes existing protocol gaps to m-8. Native Xray Hysteria2 generation already exists (archived TASK-112.2 is Done); do not reopen or duplicate it. Preserve existing children's Xray/managed-SSH scope. Cross-engine mappings below are comparison documentation, not automatic expansion to unsupported engines.

| Capability | Xray | V2Ray | sing-box |
|---|---|---|---|
| WireGuard/WARP recipe | protocol=wireguard, settings.secretKey/address/peers | No native equivalent established in the consulted outbound catalog; explicit helper/backend decision needed | Modern endpoints[].type=wireguard, address/private_key/peers (1.11+); legacy outbound removed in modern versions |
| Managed SSH | Supervised SSH -D helper plus protocol=socks settings.servers targeting owned localhost listener | Same helper/SOCKS design for supported core | Native type=ssh with server/server_port/user/private_key/host_key, or explicit helper; host verification and TCP-only limitations must be documented |
| DNS outbound | protocol=dns with settings and routing to its tag | Native protocol=dns also documented | Modern route.rules action=hijack-dns plus typed DNS servers, not removed DNS outbound |
| Loopback | protocol=loopback, settings.inboundTag for bounded routing re-entry | Native loopback similarly documented, validate version | No proven equivalent outbound; cannot substitute detour, which chains rather than re-enters routing |

WARP is configured WireGuard plus optional chaining, not automatic Cloudflare account provisioning. TASK-107 retains TUIC/ShadowTLS/AnyTLS scope decisions in m-7. [Xray outbounds](https://xtls.github.io/en/config/outbounds/), [sing-box WireGuard](https://sing-box.sagernet.org/configuration/endpoint/wireguard/), [sing-box SSH](https://sing-box.sagernet.org/configuration/outbound/ssh/), [V2Fly catalog](https://www.v2fly.org/en_US/config/outbounds.html); see doc-1.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

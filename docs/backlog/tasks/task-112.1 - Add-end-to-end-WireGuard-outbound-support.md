---
id: TASK-112.1
title: Add end-to-end WireGuard outbound support
status: To Do
assignee: []
created_date: '2026-09-02 11:02'
updated_date: '2026-10-03 09:37'
labels:
  - protocols
  - xray
  - wireguard
  - import
  - config-generation
milestone: m-8
dependencies: []
references:
  - 'https://xtls.github.io/en/config/outbounds/wireguard.html'
  - 'https://www.wireguard.com/quickstart/'
  - src/xray/parsing/protocols/outbound_settings/tunnel.rs
documentation:
  - AGENTS.md
  - docs/src/06-architecture/import-pipeline.md
  - docs/src/06-architecture/config-generation.md
  - docs/src/06-architecture/runtime-lifecycle.md
  - docs/src/05-reference/protocols.md
  - docs/src/03-features/importing.md
  - docs/src/03-features/runtime-management.md
parent_task_id: TASK-112
priority: high
ordinal: 87000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make WireGuard a first-class operational Xrat protocol for the Xray engine, not only a typed Xray JSON schema variant. Define documented accepted input forms, normalize and persist the complete supported peer configuration without silent loss, generate a valid Xray WireGuard outbound, and expose the capability consistently through CLI, TUI, API, testing, and runtime status.

Required reading before implementation:
- AGENTS.md
- docs/src/06-architecture/import-pipeline.md
- docs/src/06-architecture/config-generation.md
- docs/src/06-architecture/runtime-lifecycle.md
- docs/src/05-reference/protocols.md
- Xray WireGuard outbound documentation attached to this task
- WireGuard quick start and protocol documentation attached to this task
- Existing typed schema in src/xray/parsing/protocols/outbound_settings/tunnel.rs

Do not invent a wireguard URL format. First document which standard or upstream-defined input formats Xrat accepts. Treat private keys and preshared keys as secrets in output and logs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A documented input contract accepts at least full Xray JSON WireGuard outbounds and rejects ambiguous or incomplete inputs before persistence
- [ ] #2 The normalized model and database round trip preserve secretKey, addresses, peers, endpoint, publicKey, preSharedKey, keepAlive, allowedIPs, reserved bytes, MTU, noKernelTun, and domainStrategy when supported
- [ ] #3 Generated managed and probe Xray configurations pass the native Xray validator for representative IPv4, IPv6, multiple-peer, userspace, and kernel-TUN cases
- [ ] #4 WireGuard secrets are never printed by default in table, JSON diagnostic, event, or error output
- [ ] #5 Deduplication and config identity distinguish materially different peers without exposing secret material
- [ ] #6 CLI, TUI, API, protocol matrix, import docs, and runtime docs report WireGuard support accurately
- [ ] #7 Focused parser, persistence, generation, validation, and lifecycle tests pass together with just fmt ci
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
### Cross-engine JSON comparison (research only, 2026-10-03)
Xray implementation scope remains unchanged: outbounds[].protocol="wireguard", settings.secretKey/address/peers[].endpoint/publicKey/allowedIPs plus supported existing fields. Modern sing-box comparison: endpoints[].type="wireguard", address/private_key/peers[].address/port/public_key/allowed_ips; the legacy wireguard outbound is not the modern target. V2Ray's consulted catalog does not establish a native equivalent; do not emit Xray fields into V2Ray. WARP can reuse a provisioned WireGuard config and TASK-139 chaining; account creation is out of scope. [sing-box endpoint](https://sing-box.sagernet.org/configuration/endpoint/wireguard/); see doc-1 and TASK-112 research matrix.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

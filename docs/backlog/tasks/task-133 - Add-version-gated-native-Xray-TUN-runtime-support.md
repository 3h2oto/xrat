---
id: TASK-133
title: Add version-gated native Xray TUN runtime support
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
  - 'https://xtls.github.io/en/config/inbounds/tun.html'
  - 'https://github.com/XTLS/Xray-core/releases/tag/v26.1.23'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: high
ordinal: 115000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

Use documented native Xray TUN support rather than assuming that Xray always requires tun2socks. Define minimum core support using actual native fixtures and release/source evidence.

Xray fragment: {"protocol":"tun","tag":"xrat-tun","settings":{"name":"xrat0","mtu":1500,"gateway":["172.19.0.1/30"],"autoSystemRoutingTable":["0.0.0.0/0"],"autoOutboundsInterface":"auto"}}. Gateway IPv6 and automatic OS route/DNS support differ by platform; the docs say macOS gateway handling is IPv4-only and settings.dns is Windows-only. Use appropriate interface names per platform.

V2Ray uses its v5 service.tun schema, not this inbound; sing-box uses type=tun with address/auto_route/stack. Those adapters remain separate. Native support and automatic setup are not proof of crash cleanup. Do not emit latest-only fields to older Xray binaries.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Supported Xray versions and OS builds are detected and validated before TUN launch; older incompatible cores fail clearly.
- [ ] #2 Generated native TUN config captures intended TCP/UDP traffic without looping core egress and respects family/MTU/route settings.
- [ ] #3 Managed lifecycle and platform integration tests cover partial startup rollback, shutdown and stale-session recovery; status/events identify native Xray capture.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

---
id: TASK-143
title: Add TLS fingerprint defaults without overwriting node settings
status: To Do
assignee: []
created_date: '2026-10-03 09:34'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies:
  - TASK-71
references:
  - >-
    https://raw.githubusercontent.com/2dust/v2rayN/master/v2rayN/ServiceLib/Models/Configs/ConfigItems.cs
  - 'https://throneproj.github.io/advanced/presets/'
  - 'https://xtls.github.io/en/config/transports/tls.html'
  - 'https://www.v2fly.org/en_US/config/transport.html'
  - 'https://sing-box.sagernet.org/configuration/shared/tls/'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 125000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

XRAT already imports per-node SNI, ALPN, fingerprint and REALITY data; these are not missing protocols. The gap is an opt-in typed default fingerprint and effective override behavior similar to v2rayN/Throne presets. Coordinate with TASK-71 global/subscription/config overrides instead of modifying protocol extensions_json as user preference storage.

Xray: outbounds[].streamSettings.tlsSettings.fingerprint; REALITY uses realitySettings.fingerprint and its own constraints. Preserve explicit imported fp values unless the user chooses an explicit override.

sing-box: outbounds[].tls.utls={"enabled":true,"fingerprint":"chrome"} for supported TCP TLS transports. Do not apply uTLS to QUIC protocols merely because they use TLS.

V2Ray: the consulted upstream legacy TLS schema exposes serverName/alpn/allowInsecure but does not establish Xray-style fingerprint. Support only after source/native proof for a selected build; otherwise reject fingerprint defaults for this engine. Do not change verification or SNI automatically.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Effective default versus explicit imported-node and per-profile override precedence is documented and tested with TASK-71.
- [ ] #2 Only supported TLS/REALITY transport-engine combinations emit fingerprint fields; native checks validate accepted fingerprints.
- [ ] #3 Defaults preserve existing handshakes and certificate verification; editor/docs and managed/probe generation agree.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

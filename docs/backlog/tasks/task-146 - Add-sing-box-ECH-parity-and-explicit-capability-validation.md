---
id: TASK-146
title: Add sing-box ECH parity and explicit capability validation
status: To Do
assignee: []
created_date: '2026-10-03 09:34'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies:
  - TASK-135
references:
  - 'https://throneproj.github.io/advanced/presets/'
  - 'https://sing-box.sagernet.org/configuration/shared/tls/'
  - 'https://xtls.github.io/en/config/transports/tls.html'
  - 'https://www.v2fly.org/en_US/config/transport.html'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 128000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

XRAT already parses ech link extensions and emits Xray tlsSettings.echConfigList. The gap is modern sing-box generation plus a precise supported-engine/config contract, not a new Xray ECH feature. Inspect existing lossless import paths before changing normalization.

Xray: preserve current outbounds[].streamSettings.tlsSettings.echConfigList; explicit DNS lookup forms may also require echSockopt capability checks.

sing-box: outbounds[].tls.ech={"enabled":true,"config":[...]} or config_path; absent explicit config permits DNS HTTPS-record discovery. query_server_name is 1.13+. Xray's base64 ECHConfigList string and sing-box PEM configuration-line array are different representations; convert only with verified format equivalence.

V2Ray: the consulted legacy TLS schema does not establish these ECH fields; either validate a supported modern schema/build explicitly or reject ECH with a clear engine limitation. Requires server support, correct HTTPS DNS records/bootstrap and compatible TLS/uTLS; never silently fall back to unencrypted ClientHello.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Existing Xray ECH behavior is preserved and sing-box supported ECH input/output representations are documented and round-trip tested without invented share-link fields.
- [ ] #2 Native checks and a local ECH-capable TLS/DNS fixture prove explicit and discovered ECH work; missing configuration/server support yields actionable failures.
- [ ] #3 Unsupported core/transport combinations and resolver bootstrap loops fail before launch where detectable; client docs state exactly which engines are supported.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

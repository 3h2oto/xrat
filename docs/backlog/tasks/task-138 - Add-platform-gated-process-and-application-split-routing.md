---
id: TASK-138
title: Add platform-gated process and application split routing
status: To Do
assignee: []
created_date: '2026-10-03 09:33'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies:
  - TASK-137
references:
  - 'https://throneproj.github.io/guides/routing/'
  - 'https://xtls.github.io/en/config/routing.html'
  - 'https://sing-box.sagernet.org/configuration/route/rule/'
  - >-
    https://raw.githubusercontent.com/2dust/v2rayNG/master/V2rayNG/app/src/main/res/values/strings.xml
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 120000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

Throne exposes process name/path rules; Android v2rayNG per-app VPN selection is an OS feature. XRAT lacks typed process filters. Implement desktop process routing for engines/platforms that actually identify the original process; do not infer it from a shared SOCKS client.

Xray current schema: routing.rules [{"process":["curl"],"outboundTag":"direct"}], Windows/Linux support documented; names versus absolute paths/folders have distinct matching rules. Detect minimum supported version. Android needs a client-injected process finder and is not automatically supported by XRAT desktop.

sing-box: route.rules [{"process_name":["curl"],"action":"route","outbound":"direct"}] or process_path; package_name belongs to Android integration. Platform support and traffic-capture origin matter.

V2Ray: no equivalent process matcher in the consulted upstream routing schema; reject the feature for that target unless an explicit, separately validated OS/helper backend preserves it. OS per-app VPN exclusions do not map to a portable V2Ray JSON field.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Process-name and path filters produce correct rules only on proven engine/platform/capture combinations; unsupported combinations fail clearly.
- [ ] #2 Local process traffic and remote/LAN-origin traffic tests prove filters do not falsely match unrelated connections.
- [ ] #3 Documentation distinguishes core process routing from Android VPN allow/deny apps and explains explicit-proxy visibility limits.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

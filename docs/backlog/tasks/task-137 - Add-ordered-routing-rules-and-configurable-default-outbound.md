---
id: TASK-137
title: Add ordered routing rules and configurable default outbound
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
  - 'https://throneproj.github.io/guides/routing/'
  - 'https://www.happ.su/main/dev-docs/routing'
  - 'https://xtls.github.io/en/config/routing.html'
  - 'https://www.v2fly.org/en_US/config/routing.html'
  - 'https://sing-box.sagernet.org/configuration/route/rule/'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: high
ordinal: 119000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

XRAT RoutingSettings exposes only Direct/Block domain/IP/category lists and a domain strategy. Add ordered typed rules for explicit Proxy/Direct/Reject targets, port ranges, network, inbound tags and supported protocol matching; choose default behavior. Preserve current list precedence for migrated configs.

Xray: routing.rules objects carry domain/ip/port/network/inboundTag/protocol and outboundTag; default can be expressed with a final catch-all rule (rather than relying accidentally on outbounds[0]). Current documentation renamed source matching to sourceIP; select schema by version.

V2Ray legacy-style: routing.rules [{"type":"field","port":"443","network":"tcp","outboundTag":"proxy"}]; source/inboundTag/protocol and domain-key spelling must be validated against supported v4-compatible versus v5 formats, not copied blindly from current Xray docs.

sing-box: route.rules [{"domain_suffix":["example.org"],"port":[443],"network":"tcp","action":"route","outbound":"proxy"}], route.final="direct"; blocking uses action="reject" rather than removed block outbound where required.

Common normalized AND/OR semantics must compile faithfully even where engine rule semantics differ. Rule-set lifecycle is existing TASK-63/TASK-100.1/TASK-117. Saved-node or pool targets require existing TASK-3's multi-outbound work or an explicit prerequisite slice.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Rules preserve user order, typed matches and final target; migration preserves existing Direct-over-Block behavior.
- [ ] #2 Native engine fixtures and routing integration tests cover overlapping exceptions, default-direct/default-proxy, TCP/UDP and invalid outbound references.
- [ ] #3 Unsupported match/action combinations fail before launch rather than broadening/narrowing traffic silently; CLI/TUI/config docs show effective ordered rules.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

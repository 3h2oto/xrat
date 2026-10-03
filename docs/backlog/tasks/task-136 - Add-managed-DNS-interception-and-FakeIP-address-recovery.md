---
id: TASK-136
title: Add managed DNS interception and FakeIP address recovery
status: To Do
assignee: []
created_date: '2026-10-03 09:33'
updated_date: '2026-10-03 09:40'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies:
  - TASK-135
  - TASK-112.3
references:
  - >-
    https://raw.githubusercontent.com/2dust/v2rayNG/master/V2rayNG/app/src/main/res/values/strings.xml
  - 'https://throneproj.github.io/guides/dns/'
  - 'https://xtls.github.io/en/config/fakedns.html'
  - 'https://www.v2fly.org/en_US/config/fakedns.html'
  - 'https://sing-box.sagernet.org/configuration/dns/server/fakeip/'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: high
ordinal: 118000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

XRAT can accept DNS server URLs but has no complete generated DNS listener/interception/FakeIP lifecycle; sing-box currently rejects the fakedns string. Add DNS capture independently of TUN where a local listener is useful, with an opt-in FakeIP mode and exclusions. This is not equivalent to merely setting dns.servers=["fakedns"].

Xray and compatible V2Ray fragments: root fakedns:[{"ipPool":"198.18.0.0/15","poolSize":65535}], dns.servers includes "fakedns", appropriate inbound sniffing enabled with destOverride including "fakedns"; route DNS to a generated protocol=dns outbound. TASK-112.3 owns that outbound support. Exact pool/sniffing capabilities are version-specific.

Modern sing-box: dns.servers [{"type":"fakeip","tag":"fake","inet4_range":"198.18.0.0/15"}]; dns.rules choose fake for eligible A/AAAA questions and real resolvers for exclusions; route.rules [{"protocol":"dns","action":"hijack-dns"}]. A local direct inbound/DNS capture listener or TUN supplies the traffic. New fakeip server schema is 1.12+; do not generate the removed legacy dns outbound.

No engine setting forces apps using independent encrypted DNS to use the local resolver. Define fake-range routing, reverse mapping, cache restart behavior and family exclusions; fixed host entries remain real.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Real DNS interception works before enabling FakeIP, with explicit listener/capture setup and DNS outbound versus hijack-dns mappings per engine.
- [ ] #2 FakeIP generation and reverse lookup preserve original domains across direct/proxy routing; excluded domains, hosts, A/AAAA and fake range collisions are covered.
- [ ] #3 Native validation and local DNS-to-connection integration tests demonstrate behavior and restart/cache limits, with no unsupported-engine silent fallback.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

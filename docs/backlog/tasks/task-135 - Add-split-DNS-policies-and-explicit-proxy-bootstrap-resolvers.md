---
id: TASK-135
title: Add split DNS policies and explicit proxy bootstrap resolvers
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
  - >-
    https://raw.githubusercontent.com/2dust/v2rayN/master/v2rayN/ServiceLib/Models/Configs/ConfigItems.cs
  - >-
    https://raw.githubusercontent.com/2dust/v2rayNG/master/V2rayNG/app/src/main/res/values/strings.xml
  - 'https://throneproj.github.io/guides/dns/'
  - 'https://xtls.github.io/en/config/dns.html'
  - 'https://www.v2fly.org/en_US/config/dns.html'
  - 'https://sing-box.sagernet.org/configuration/dns/'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: high
ordinal: 117000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

XRAT DnsSettings has server strings and global policy, but cannot express domain-to-resolver rules, resolver detours or an explicit resolver for proxy-server hostnames. Clients expose Direct/Remote/Bootstrap DNS. Existing encrypted URL support is already present and must be preserved.

| Engine | JSON effect |
|---|---|
| Xray | dns.servers object entries use address, domains, queryStrategy and tags; route non-local DNS egress via routing.rules/outboundTag. Newer expectedIPs/timeoutMs fields need separate gates. Bootstrap proxy names via explicit hosts or supported resolver policy without a recursive proxy dependency. |
| V2Ray | dns.servers objects use address/domains and documented expectIPs; DNS tag plus routing.rules select the outbound. Do not copy Xray-only renamed fields into V2Ray. |
| sing-box | typed dns.servers with tags, dns.rules [{"domain_suffix":["example.org"],"action":"route","server":"remote"}], dns.final and proxy outbound.domain_resolver="bootstrap"; encrypted resolver detour points at a generated outbound, with bootstrap resolver for its hostname. |

Resolver choice, query-family restrictions, actual dial fallback and destination routing are separate settings. DNS query strategies must not inadvertently override TASK-130. DNS response-filter/racing parity beyond the initial split policy must be explicitly capability-gated.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Typed Direct/Remote/Bootstrap resolvers and ordered domain policies generate native-valid configurations without recursive DNS/proxy dependencies.
- [ ] #2 Tests establish that proxied resolver requests take the configured path and proxy-server hostname bootstrap remains reachable before the proxy starts.
- [ ] #3 Global legacy settings migrate without changing defaults; unsupported transports/fields and IPv6-policy conflicts fail with field-specific diagnostics.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

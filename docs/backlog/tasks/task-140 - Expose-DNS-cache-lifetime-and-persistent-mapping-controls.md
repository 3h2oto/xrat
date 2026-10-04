---
id: TASK-140
title: Expose DNS cache lifetime and persistent mapping controls
status: To Do
assignee: []
created_date: '2026-10-03 09:34'
updated_date: '2026-10-04 17:35'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies:
  - TASK-135
references:
  - 'https://throneproj.github.io/guides/dns/'
  - 'https://sing-box.sagernet.org/configuration/dns/'
  - 'https://xtls.github.io/en/config/dns.html'
  - 'https://www.v2fly.org/en_US/config/dns.html'
  - 'https://sing-box.sagernet.org/configuration/experimental/cache-file/'
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 129000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

XRAT only exposes disable_cache. Add typed optional cache policy controls where supported, preserving defaults and isolating session/profile namespaces. Existing rule-set caching is not proof of persistent DNS answer caching.

Xray current DNS fields: serveStale and serveExpiredTTL alongside disableCache; verify exact supported versions and stale-answer semantics. The consulted V2Ray DNS schema does not establish these options or a persistent DNS cache file: unsupported controls must be rejected, not copied from Xray.

sing-box DNS fields: cache_capacity (1.11+), disable_expire, reverse_mapping; optimistic and timeout are 1.14+ and require gating. experimental.cache_file.enabled/path/cache_id/store_fakeip persists FakeIP mappings; store_dns is 1.14+, not a sing-box 1.13 feature. independent_cache has later deprecation and should not be introduced without a compatibility policy.

These are engine JSON changes, not XRAT database caches. Define clearing, file ownership, TTL behavior and privacy documentation; do not label stale DNS data as fresh.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Cache controls expose only supported semantics and maintain current defaults; conflicting disable-cache/stale settings are diagnosed.
- [ ] #2 Native fixtures and deterministic local resolver tests cover expiry, stale refresh and resolver/profile isolation.
- [ ] #3 Persistent file lifecycle and FakeIP restart behavior are tested where supported, with 1.13 versus 1.14+ DNS persistence documented accurately.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

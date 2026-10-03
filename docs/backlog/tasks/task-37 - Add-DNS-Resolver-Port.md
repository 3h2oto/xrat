---
id: TASK-37
title: Add DNS Resolver Port
status: In Progress
assignee:
  - codex
created_date: '2026-07-05 14:43'
updated_date: '2026-10-03 08:05'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 17
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/17-dns-resolver-port.md`

# Add DNS Resolver Port

## Finding

### [Priority: Low] Add a DNS resolver port for probers

**Files involved:**

- `src/prober/tcp/check.rs:11`
- `src/prober/icmp/mod.rs:43`

**Problem:** `tokio::net::lookup_host` is called directly in the TCP and ICMP
probers. DNS failures, slow resolution, and empty results cannot be simulated in
tests without real DNS queries.

**Why this change is needed:** Network probe tests that exercise DNS failure
paths currently require either real DNS that happens to fail or invasive test
infrastructure. Extracting a port makes probe tests more reliable and adds the
ability to assert probe behavior under DNS errors.

**How to implement it:** Introduce a `DnsResolver` trait with a single method.
Provide a `TokioDnsResolver` production adapter and a `MockDnsResolver` test
adapter. Inject into prober constructors.

**Positive effect on the codebase:** Probe tests can inject canned IP results or
simulate `NXDOMAIN` errors. The change is small and contained.

**Suggested target architecture:** `DnsResolver` port in `src/prober/` or
`src/support/`. Injected into TCP and ICMP prober services.

**Risk / migration notes:** Low risk. Two call sites, simple replacement.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Direct DNS lookup is confined to its shared production adapter
- [ ] #2 TCP, ICMP and hostname enrichment accept injected resolution; TCP attempts are also fakeable
- [ ] #3 Empty/error results, literal addresses and address-order/timeout behavior have regression coverage
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Introduce ordered DnsResolver and production Tokio adapter in xrat-support; inject DNS and TCP connection attempts into TCP probes, DNS/process/platform into ICMP, and DNS into hostname enrichment; preserve address order and current failure/timeout semantics; test failures and literals without network.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Shared DNS adapter committed in 3eba21f; TCP/ICMP and GeoIP consumer injection committed in 75e920e. Fake TCP tests verify ordered IPv6/IPv4 fallback and empty answers without real DNS or sockets. CARGO_INCREMENTAL=0 just fmt ci passed with 916 Rust tests and 3 Python tests. Remains In Progress pending injected DNS error/timeout, ICMP literal bypass and GeoIP provenance regressions plus final acceptance audit.
<!-- SECTION:NOTES:END -->

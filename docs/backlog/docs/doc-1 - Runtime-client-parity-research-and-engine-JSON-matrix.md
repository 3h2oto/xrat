---
id: doc-1
title: Runtime client parity research and engine JSON matrix
type: specification
created_date: '2026-10-03 09:27'
updated_date: '2026-10-03 09:39'
tags:
  - research
  - runtime
  - client-parity
---
# Runtime client parity research and engine JSON matrix

Research date: **2026-10-03**. Milestone: **m-8 — Runtime Settings and Client Feature Parity**. This is research and queued work only: no implementation started, no pending task moved to In Progress.

## Evidence and boundaries

Read TASK-130 and TASK-112, relevant existing tasks, current configuration structs, runtime tuning and engine generators. The knowledge graph transport was unavailable, so direct source inspection supplied the XRAT baseline. Compare client capabilities to stock upstream engine schemas, rather than treating a client setting as a universally portable JSON key.

XRAT currently selects Xray stable v26.3.27 or prerelease v26.7.28 compatibility, and generates modern sing-box 1.13 settings. Online docs now include later schema additions. All fragments in tasks are **research mappings**, not tested full configurations or a claim that minimum-version/runtime proof has passed. V2Ray legacy compatible JSON and v5 service/config formats require separate handling. Version strings alone may not prove build-tag/platform support.

## Client comparison sources

| Client | Observed relevant capability | Primary evidence |
|---|---|---|
| v2rayN | TUN MTU/stack/routes, Direct/Remote/Bootstrap DNS, FakeIP, fingerprints, separate core mux settings, Happy Eyeballs controls | [Current settings model](https://raw.githubusercontent.com/2dust/v2rayN/master/v2rayN/ServiceLib/Models/Configs/ConfigItems.cs), [maintainer wiki](https://github.com/2dust/v2rayN/wiki) |
| v2rayNG | Prefer IPv6 distinct from VPN IPv6, local/direct/remote DNS, FakeDNS and VPN MTU/LAN bypass controls | [Current settings strings](https://raw.githubusercontent.com/2dust/v2rayNG/master/V2rayNG/app/src/main/res/values/strings.xml), [upstream Android client](https://github.com/2dust/v2rayNG) |
| Happ | Named routing profiles with activation, sharing and geo-asset management | [Official routing guide](https://www.happ.su/main/dev-docs/routing) |
| Throne | TUN, ordered/process routing, DNS cache/FakeIP, front/landing chains, inherited tuning presets | [Routing](https://throneproj.github.io/guides/routing/), [DNS](https://throneproj.github.io/guides/dns/), [chains](https://throneproj.github.io/advanced/chains/), [presets](https://throneproj.github.io/advanced/presets/) |
| Hiddify | TUN, automatic proxy choice, broader protocol presentation and TLS trick controls | [Official app guide](https://hiddify.com/app/How-to-use-Hiddify-app/), [TLS trick guide](https://hiddify.com/manager/basic-concepts-and-troubleshooting/How-the-TLS-Trick-works-and-its-usage/) |
| v2rayA | Transparent capture and node groups/routing; current project uses its own Xray-based core | [Upstream README](https://github.com/v2rayA/v2rayA) |
| Nekoray | Historical sing-box client with additional protocol/custom-core support; archived 2025-03-17 | [Archived upstream repository](https://github.com/MatsuriDayo/nekoray) |

This table does not claim every client implements every row, or that upstream clients were run locally. Client forks and external helper programs may provide features absent from stock cores. Android per-app VPN filtering is OS integration, not a portable desktop engine field.

## Confirmed XRAT baseline

- RuntimeSettings already exposes SOCKS/HTTP/Shadowsocks listeners, sniffing, stats, rotation, Xray-oriented mux/fragment and network binding.
- DnsSettings exposes server strings, hosts, system hosts and several global flags. It cannot express arbitrary domain resolver policies or per-server bootstrap/detour preferences.
- RoutingSettings exposes Direct/Block domain/IP/category lists and a domain strategy. It lacks a user-ordered rule model, explicit proxy exceptions/default-direct and process routing.
- SingboxInbound supports SOCKS, HTTP and Shadowsocks, with no generated managed TUN backend.
- Xray ECH, per-node SNI/ALPN/fingerprint and native Hysteria2 generation already exist. Those are not new Xray protocol gaps.
- Sing-box already emits remote GeoIP/Geosite rule sets and a cache file. TASK-63 remains partial/open until its complete asset lifecycle/native evidence is met; it is reused rather than duplicated.
- Xray fragment-helper chaining is present, but an arbitrary stored-node chain model is absent.
- Managed rotation/probing already exist; multi-outbound native pool generation is a separate existing gap.

Source pointers: crates/xrat-app/src/app/config/{proxy/types.rs,dns.rs,routing.rs}; crates/xrat-app/src/app/services/runtime_tuning/{xray.rs,singbox.rs}; crates/xrat-engines/src/{xray/config/{stream.rs,tuning.rs,types.rs},singbox/config/{mod.rs,transport.rs}}; docs/src/05-reference/{config-file.md,protocols.md}.

## Queued feature matrix

Each task contains its own affected JSON nodes/fragments, engine limitations, primary references and acceptance criteria.

| Task | Priority / outcome | V2Ray JSON | Xray JSON | sing-box JSON |
|---|---|---|---|---|
| TASK-130 (existing) | High: IPv6-first proxy dial with IPv4 connection fallback | No matching happyEyeballs field established; native/helper capability decision required | sockopt UseIP plus happyEyeballs.prioritizeIPv6; real egress only | Proxy domain_resolver strategy candidate; actual fallback needs integration proof |
| TASK-131 | High: TUN contract/version-platform matrix | v5 service.tun, separate schema | Native tun inbound/settings | tun inbound/address/route/stack |
| TASK-132 | High: supervised sing-box TUN | Different adapter; reject mismatched engine | Different adapter | type=tun, auto_route, strict_route, auto_detect_interface |
| TASK-133 | High: native Xray TUN | Different schema | protocol=tun, gateway, autoSystemRoutingTable, autoOutboundsInterface | Different adapter |
| TASK-134 | Medium: explicit V2Ray TUN backend | v5.9+ service.tun on documented Linux targets, or explicit helper | Different adapter | Different adapter |
| TASK-135 | High: split DNS and bootstrap | dns.servers objects and DNS-tag routing | dns.servers objects and DNS-tag routing | typed dns.servers/rules/final, outbound.domain_resolver/detour |
| TASK-136 | High: DNS interception/FakeIP | fakedns, DNS outbound, sniffing recovery | fakedns, DNS outbound, sniffing recovery | fakeip server, hijack-dns action and capture listener |
| TASK-137 | High: ordered rules/default target | routing.rules/outboundTag with legacy/v5 gates | routing.rules/outboundTag or balancerTag | route.rules/action and route.final |
| TASK-138 | Medium: process split routing | Unsupported in consulted stock schema | routing.rules.process on supported platforms/versions | process_name/process_path; Android package integration is separate |
| TASK-139 | High: front/landing chains | proxySettings.tag/transportLayer, validated version | sockopt.dialerProxy | outbound.detour |
| TASK-140 | Medium: cache lifetime/mapping persistence | Only demonstrated DNS controls; reject unavailable extras | serveStale/serveExpiredTTL version-gated | cache_capacity/reverse_mapping/cache_file; store_dns is 1.14+ |
| TASK-141 | Medium: socket tuning | tcpFastOpen and source-confirmed keepalive | sockopt TCP knobs; tcpUserTimeout is not dial timeout | connect_timeout, TCP keepalive, tcp_fast_open, udp_fragment |
| TASK-142 | Medium: named routing profiles | Generated routing rules/default | Generated routing rules/default | Generated route/rule sets; DNS policy only if explicitly included |
| TASK-143 | Medium: fingerprint defaults/overrides | Capability not established by consulted legacy schema | TLS/REALITY fingerprint | tls.utls for supported transports |
| TASK-144 | Medium: sing-box mux protocol/padding | Preserve existing supported mux | Preserve existing mux/XUDP | multiplex protocol/stream limits/padding |
| TASK-145 | Medium: sing-box TLS fragmentation | No demonstrated equivalent | Preserve existing freedom fragment helper | tls.fragment/record_fragment since 1.12 |
| TASK-146 | Medium: sing-box ECH parity | Capability requires proof or explicit rejection | Preserve existing echConfigList | tls.ech with verified representation conversion/discovery |
| TASK-147 | Medium: Linux REDIRECT/TPROXY | dokodemo-door + sockopt.tproxy | compatible inlet/tproxy with current alias/version gates | redirect/tproxy inbounds plus OS routing |

## Existing work reused and related scope

Assigned to m-8 without starting it: TASK-130; TASK-3 outbound pools; TASK-71 and TASK-71.1 override resolution/editor; TASK-63 rule-set lifecycle; TASK-112 and its unfinished children TASK-112.1, .3, .4, .5, .6 and .7. Their research notes now include cross-engine mappings.

TASK-112 keeps its existing implementation scope. WireGuard maps to Xray protocol=wireguard; modern sing-box comparison uses endpoints, not the removed legacy outbound. SSH planned for Xray/V2Ray uses a supervised dynamic SOCKS helper, while sing-box also has a different native SSH outbound. DNS/Loopback native Xray outbound work remains in its existing children. Native Xray Hysteria2 child TASK-112.2 is already Done in the completed folder.

TASK-107 remains in m-7 and owns decisions for TUIC, ShadowTLS and AnyTLS; no protocol implementation is implicitly approved here. WARP is a configured WireGuard-and-chain recipe, not a new protocol or automatic account provisioning. TASK-100.1 asset management, TASK-102 conformance fixtures, TASK-109 release validation and TASK-117 newer sing-box compatibility retain their current assignments.

## Critical semantic distinctions

1. IPv6-first DNS is not fallback after a failed connection. Xray UseIPv6v4 alone does not satisfy TASK-130; Happy Eyeballs is TCP-only and ineffective on a hop with dialerProxy. Current sing-box mobile interface fallback controls cannot be assumed to work in a desktop CLI. [Xray socket options](https://xtls.github.io/en/config/transports/sockopt.html), [sing-box dial fields](https://sing-box.sagernet.org/configuration/shared/dial/).
2. TUN is now documented natively by Xray as well as sing-box; V2Ray has a separate v5 service. Do not use the outdated blanket assumption that both always require tun2socks. Privilege/OS routes, loop avoidance and cleanup still need managed lifecycle work. [Xray TUN](https://xtls.github.io/en/config/inbounds/tun.html), [V2Fly TUN](https://www.v2fly.org/en_US/config/service/tun.html), [sing-box TUN](https://sing-box.sagernet.org/configuration/inbound/tun/).
3. Modern sing-box uses typed DNS servers, route sniff/hijack/reject actions, rule sets and WireGuard endpoints. Legacy client examples are not valid universal templates. Features introduced after 1.13 remain gated. [Migration guide](https://sing-box.sagernet.org/migration/).
4. Raw custom-core features, Hiddify fork-only padding, persistent kill switches and mobile per-app VPN APIs are not automatic stock-core parity. This milestone uses typed settings and explicit unsupported diagnostics, not storage of complete root JSON.
5. Sniffing, fragmentation, ECH, fake addresses and multiplexing solve different problems. Server compatibility, certificate verification, SNI/Host identity and imported protocol parameters must survive preference resolution.

## Delivery order and verification contract

Start later with TASK-130 and TASK-135, then TASK-137 plus TASK-71 inheritance. Complete TASK-131's contract before native TUN backends. DNS capture/FakeIP must coordinate TASK-112.3 and relevant capture backends; declare dependencies before implementation selects a backend. Chains and pool-aware routing require an explicit generated outbound graph. Profile updates and process filtering build on ordered routing; advanced settings follow engine capability checks.

Acceptance must establish typed input validation, native engine config validation and actual local network behavior as separate layers. Relevant connection tuning applies to managed and probe/scan paths; TUN/firewall capture does not automatically belong in probes. OS tests must exercise cancellation, partial startup, core crash and owned-network rollback. Docs/editor/status/events must describe effective capabilities. Run just fmt ci for later implementation commits. Research itself has not run core validators, deployed clients or behavioral network tests.

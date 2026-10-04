---
id: m-8
title: "Runtime Settings and Client Feature Parity"
---

## Description

Research baseline: 2026-10-03. Close demonstrated runtime-setting and client-feature gaps against v2rayN, v2rayNG, Happ, Throne, Hiddify and v2rayA; use archived Nekoray only as historical context. No implementation authorized by this research request; all pending tasks remain To Do.

Scope: IPv6-first proxy dialing with IPv4 connection fallback; managed TUN and Linux transparent capture; split/bootstrap DNS, DNS interception and FakeIP; ordered and process routing, reusable routing profiles; proxy chains and outbound pools; per-subscription/config overrides; TLS defaults, ECH, modern sing-box multiplex/fragment settings and socket tuning.

Every feature must document separate V2Ray, Xray and sing-box JSON mappings, unsupported combinations, core-version/build/platform gates and native validation plus behavioral proof. Latest online schemas are research evidence, not proof that XRAT's supported binaries accept a field. XRAT currently targets Xray stable v26.3.27/prerelease v26.7.28 and sing-box 1.13; newer documentation must be gated. Generate configs from typed normalized settings; never persist full root engine JSON or silently drop settings. Probe/scan applies connection tuning, not privileged system traffic capture.

Reuse TASK-130 (High), TASK-71/71.1, TASK-3, TASK-63 and TASK-112 with its unfinished children. TASK-107, TASK-100.1, TASK-102, TASK-109 and TASK-117 stay in their current compatibility milestone and are linked prerequisites/related scope. TASK-112.2 is already completed and is not reopened. WARP is a WireGuard-plus-chain recipe, not a new native protocol. TUIC/ShadowTLS/AnyTLS remain decisions in TASK-107, not implicit commitments.

Suggested order: address selection and split DNS first; ordered routing/overrides alongside those; TUN contract then each engine backend and interception/FakeIP; chains/pools; presets and advanced tuning. Dependencies on tasks are delivery prerequisites, not a request to start them.

---
id: TASK-141
title: Expose typed TCP and UDP dial tuning by engine capability
status: To Do
assignee: []
created_date: '2026-10-03 09:34'
updated_date: '2026-10-04 17:35'
labels:
  - feature
  - runtime
  - client-parity
milestone: m-8
dependencies: []
references:
  - 'https://throneproj.github.io/advanced/chains/'
  - 'https://throneproj.github.io/advanced/presets/'
  - 'https://xtls.github.io/en/config/transports/sockopt.html'
  - 'https://www.v2fly.org/en_US/config/transport.html'
  - 'https://sing-box.sagernet.org/configuration/shared/dial/'
  - >-
    https://raw.githubusercontent.com/v2fly/v2ray-core/master/infra/conf/cfgcommon/socketcfg/socket.go
documentation:
  - >-
    docs/backlog/docs/doc-1 -
    Runtime-client-parity-research-and-engine-JSON-matrix.md
priority: medium
ordinal: 130000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research only, recorded 2026-10-03; implementation has not started. JSON examples are affected fragments, not full runnable configurations. Current upstream docs may describe newer versions than XRAT supports; pin capabilities with native fixtures before coding.

XRAT NetworkSettings contains binding/interface/mark controls but no typed TCP Fast Open, keepalive or dial-timeout settings. Advanced clients allow socket options through raw configs/presets. Add a bounded settings surface rather than arbitrary numeric OS socket options.

Xray streamSettings.sockopt: tcpFastOpen, tcpKeepAliveIdle, tcpKeepAliveInterval, tcpUserTimeout; units and supported OS/core versions must be explicit. tcpUserTimeout limits unacknowledged data and is NOT an initial dial timeout.

V2Ray current upstream socketcfg source exposes tcpFastOpen, tcpKeepAliveIdle and tcpKeepAliveInterval, plus mptcp/bindToDevice. Pin their accepted versions/platforms with native fixtures; the source does not expose Xray tcpUserTimeout, so do not copy it.

sing-box outbound dial fields: tcp_fast_open, connect_timeout (duration string), tcp_keep_alive and tcp_keep_alive_interval (1.13+), udp_fragment. UDP IP fragmentation is not TLS fragmentation and does not guarantee UDP-over-TCP or full-cone NAT.

Apply to the actual egress dialer in chains and fragment helpers. Do not force TCP knobs onto Hysteria2/other QUIC transports. Preserve OS defaults when unset.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Typed settings validate ranges/units and supported transport/engine/platform combinations before launch.
- [ ] #2 Generated configurations and local dialing tests distinguish TCP dial timeout, keepalive and user timeout; probe/scan and managed runtime use the same effective egress tuning.
- [ ] #3 Defaults preserve current output, and chain/detour handling prevents ineffective knobs being advertised as applied.
- [ ] #4 Documentation/editor help explains effective settings and engine/version/platform limits; focused regression and native/behavioral checks pass together with just fmt ci.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->

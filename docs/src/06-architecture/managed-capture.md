# Managed capture contract

XRAT owns managed sessions and verifies TUN interface indices before cleanup.
Engine configuration acceptance, lifecycle tests and actual packet capture are
separate checks. `tun status` reports readiness and ownership, not traffic
proof.

## Capability matrix

| Engine   | Linux                                             | macOS                                                     | Windows x64/ARM64                                                          |
| -------- | ------------------------------------------------- | --------------------------------------------------------- | -------------------------------------------------------------------------- |
| Xray     | >= 26.7.11; pinned acceptance 26.7.11 and 26.9.30 | Unsupported managed capture                               | >= 26.9.30 and < 27.0.0; protected service only; native acceptance pending |
| sing-box | >= 1.13.0; pinned acceptance 1.13.21              | Root owner with numbered utun; native acceptance required | >= 1.13.21 and < 1.14.0; protected service only; native acceptance pending |
| V2Ray    | Unsupported                                       | Unsupported                                               | Separate follow-up, outside this delivery                                  |

Linux ARM64 uses the same backend; packet acceptance must run on that host.
Other architectures have no validated managed capture contract.

Xray 26.3.27 and unknown-version binaries are rejected before replacement.
Sing-box requires at least 1.13.0; newer versions pass native config preflight
and receive a warning outside the planned conformance range. A V2Ray v5
`service.tun` helper is separate future work; XRAT does not invent a TUN inbound
in its current schema. Official archive SHA-256 values are pinned in the
`runtime-engines` recipe in `Justfile`; the downloader checks cached archives
too.

| Setting                 | Xray                                                              | sing-box                                                          |
| ----------------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------- |
| `enabled`               | Opt-in native managed TUN                                         | Opt-in native managed TUN                                         |
| `interface_name`        | 1..15 bytes, no whitespace/slash/NUL; ownership checked           | Same                                                              |
| `address`               | IPv4, IPv6 or both; gateways and capture families match           | Native address list                                               |
| `mtu`                   | 1280..65535                                                       | 1280..65535                                                       |
| `auto_route`            | Two `/1` prefixes per selected family; physical defaults retained | Native auto-routing with physical interface detection             |
| `route_exclude_address` | Rejected                                                          | Native capture exclusions                                         |
| `stack`                 | Default placeholder `system` only; fixed native stack             | `system`, `gvisor`, `mixed`                                       |
| `strict_route`          | Rejected when true                                                | Native stricter routing; no persistent kill switch                |
| Route inclusions        | Selected whole address families; custom include CIDRs unsupported | Selected whole address families; custom include CIDRs unsupported |
| DNS ownership           | Managed port-53 interception and optional loopback listener       | Native hijack-DNS and optional loopback listener                  |
| Physical-interface DNS  | Never changed by XRAT                                             | Never changed by XRAT                                             |
| Local proxy             | At least one listener remains enabled for readiness and fallback  | Same                                                              |
| Desktop proxy           | Independent opt-in; not required by capture                       | Same                                                              |

Both the core and the owner need Linux network privileges. Use `xrat tun setup`
for file capabilities on ordinary installations. Root or ambient capabilities
that survive engine execution are also accepted. A systemd `NoNewPrivileges`
restriction requires the documented TUN override and a daemon restart.

Cleanup removes only a verified, owned TUN interface. Kernel-owned routes tied
to that interface disappear with it. Sing-box policy rules are saved with their
selectors and removed explicitly after core failure; only kernel-generated
detached/unresolved flags are ignored when identifying a saved rule. Reserved
sing-box priorities 9000..9010 must be free or verified as owned before launch.
A mismatched or unknown interface index, foreign interface or failed cleanup
preserves the resource and ownership record for a later recovery attempt. Normal
disconnect, daemon shutdown and detected core failure run through the same
lifecycle service. Abrupt namespace destruction is a test isolation boundary,
not a substitute for asserting managed cleanup.

## Windows ownership

The LocalSystem service checks protected engine paths and supported versions.
Xray uses the explicitly provisioned, checksum- and signature-verified Wintun
DLL; toggles do not download it. Windows Xray configuration includes per-family
TUN DNS addresses. Sing-box automatic routing requires `strict_route = true`.

IP Helper snapshots verify preferred interface addresses, DNS servers and full
capture-route coverage after explicit exclusions. The ownership journal saves
both the adapter GUID and LUID before readiness succeeds. A missing journal
write prevents launch; a changed identity prevents cleanup. Engine exit must
release the adapter and all routes for its recorded LUID before the journal is
cleared. Foreign adapters, routes and physical-interface DNS are preserved. An
SCM process job terminates descendants on service death. Recovery reconciles the
journal and remains disconnected until the operator connects explicitly.

See [Windows preparation](../02-cli/tun.md#windows-preparation). Native traffic,
DNS, rollback, crash and reboot acceptance remains pending on both
architectures.

## Reproduce acceptance checks

On Linux amd64 with `iproute2`, OpenSSL, Rust, Just, curl, unzip/tar and user
namespaces enabled:

```bash
just runtime-engines /tmp/xrat-engines
just runtime-native /tmp/xrat-engines/xray /tmp/xrat-engines/sing-box /tmp/xrat-native
just runtime-dns /tmp/xrat-engines/xray /tmp/xrat-engines/sing-box /tmp/xrat-native
just build
just runtime-tun target/debug/xrat /tmp/xrat-engines/xray /tmp/xrat-engines/sing-box /tmp/xrat-tun
just fmt ci
```

The network harnesses are ordinary Rust integration tests in
`tests/runtime_network.rs`, with shared fixtures under `tests/runtime_network/`.
They use `#[ignore]` because they require external engines and namespace
privileges. Normal `cargo test` compiles them; the recipes above run them
explicitly. CI executes the same compiled tests with sufficient namespace
privileges. Version and SDK checks also live under `tests/`; version updates and
engine downloads are `Justfile` recipes. No Python helper or separate tooling
crate is required.

Native validation fails if a required binary is missing. DNS checks run real
UDP/TCP/HTTPS resolver policies, forwarding actions, IPv4/IPv6 FakeIP domain
recovery and private-cache persistence across restart inside a disposable
namespace. The TUN harness creates a physical veth path and a separate origin
namespace with controlled SOCKS5, DNS, TCP and UDP servers. It checks capture
before invoking the local proxy, verifies destination-side observations, and
compares interface/address/route/rule snapshots after disconnect, live toggles,
core termination and daemon shutdown. Failed bootstrap, older/unknown cores,
foreign interfaces and occupied policy priorities are rejected while the prior
local proxy remains usable. It keeps a foreign interface to check that unrelated
network state survives. Receipts and command logs go to the explicit output
directory. Host interfaces and routes are never modified.

The harness is Linux amd64 evidence for the pinned builds. It does not establish
non-Linux support, ARM packet behavior or a fail-closed firewall guarantee.

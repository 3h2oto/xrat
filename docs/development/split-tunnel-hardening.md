# Split tunneling hardening and macOS support

Scope: fix the routing/resolution defects introduced by PR #213 and add managed
sing-box TUN capture on macOS. Xray managed TUN remains Linux-only.

## Delivery order and acceptance criteria

1. Routing and resolution
   - Combine sing-box name/path/directory alternatives with OR, scoped to
     tun-in.
   - Use one resolver for validation, compilation, CLI and TUI discovery.
   - Preserve desktop identities, canonical executable paths and application
     workers.
   - Infer shell targets only from literal exec statements; never execute
     scripts.
   - Fixtures must not depend on installed browsers or PATH contents.
2. Runtime safety
   - Limit Linux process-inspection capabilities to engine executables.
   - Report disk rollback failures and retain ownership when cleanup fails.
   - Verify rejected changes preserve the previous runtime.
3. macOS
   - Support sing-box with explicit utun names and privileged runtime ownership.
   - Inspect kernel interface indices; refuse foreign/reused interfaces.
   - Let the kernel release utun on process exit; never delete arbitrary
     interfaces.
   - Discover .app bundles and running executables, including bundle helpers.
   - Reject unsupported engines/platform options before replacing a connection.
4. Validation and documentation
   - Run focused resolver, routing, privilege and lifecycle regressions.
   - Add native traffic acceptance checks for whitelist/blacklist and cleanup.
   - Run workspace formatting, clippy and tests.
   - Add macOS CI validation; distinguish source/cross-target checks from native
     proof.

## Platform validation

Linux supports both Xray and sing-box. macOS initially supports sing-box only.
macOS acceptance must verify privileged launch, proxy/direct application
traffic, DNS interception, normal disconnect, forced engine exit, interface
disappearance, and route/DNS baseline preservation. A Linux run does not prove
these checks.

## Implementation and validation

Tracked in [issue #214](https://github.com/mhyrzt/xrat/issues/214).

- Implemented all five reviewed defects: matcher OR semantics, literal wrapper
  targets, consistent desktop ID resolution, canonical symlink paths and TUI
  desktop identity preservation, including already-running apps.
- Engine executables retain process-inspection capabilities; XRAT receives only
  network capabilities. Rollback failures are surfaced and concurrent TUN edits
  are preserved.
- macOS implementation includes sing-box-only validation, numbered utun
  defaults, root-owner checks, bundle/helper discovery, verified interface
  release, capture route readiness and conflict preflight. Destination
  exclusions and strict_route are rejected in this first implementation.
- Local workspace formatting, clippy and tests passed. Sandbox-only failures
  were resolved by rerunning socket/IPC tests with local bindings permitted.
- Checksum-pinned Xray 26.7.11 and sing-box 1.13.21 passed all five Linux
  capture scenarios: 40 application routing checks across whitelist/blacklist,
  protected DNS and exact baseline restoration. Receipts are under
  `/tmp/xrat-pr213-traffic-pinned/` for this session.
- Short-lived traffic clients are separated beyond sing-box's one-second process
  cache. Exploratory runs with local sing-box 1.13.19 exposed intermittent
  attribution and teardown failures; they are not used as acceptance evidence.
- Native macOS CI now runs platform resolution/validation tests and a
  checksum-pinned root utun lifecycle test covering graceful/forced exit and DNS
  configuration preservation. These new CI steps have not run yet. Full managed
  macOS application traffic acceptance remains pending a native privileged run.
- Linux-to-macOS cross-check was blocked by the missing Apple C toolchain/SDK,
  before compiling the application. It provides no macOS build proof.

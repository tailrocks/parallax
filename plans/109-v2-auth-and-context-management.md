# Plan 109: Add V2 authentication and named remote contexts

> **Executor instructions**: Do not open this scope until the operator declares
> V2 authentication/remote contexts active. Start with the security and context
> contract; the existing client token field is a stub, not an approved design.

## Status

- **Priority**: P1 when V2 opens
- **Effort**: L
- **Risk**: CRITICAL
- **Depends on**: Operator opens V2 scope
- **Category**: V2 / authentication / CLI contexts
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: IN PROGRESS — minimal local-first slice implemented 2026-07-17
- **Blocker**: none for minimal slice (opened by unblock directive). Residual: keyring, ingest tokens, multi-scope RBAC.

## Scope

- An approved ADR for authentication, authorization, token lifecycle/storage,
  transport, context selection, and local-versus-remote behavior.
- Server middleware and capability-scoped authorization with sanitized errors.
- Native-TLS clients and secure OS-backed credential storage; no token in repo,
  config export, command line, URL, logs, telemetry, bundle, or crash output.
- `parallax context add/list/use/show/remove` with atomic configuration updates.
- Clear non-interactive/CI behavior and compatibility for existing local usage.

Out of scope until separately approved:

- Multi-tenant billing/RBAC beyond the authorized V2 capability model.
- Browser token persistence chosen by implementation convenience.
- rustls, custom root bundles, or plaintext remote credentials.
- Treating raw GraphQL/SSE as an agent API without the plan 099 safety gate.

## Steps After Trigger

1. Write and approve an ADR covering actor/threat model, credential issuer and
   format, scopes, expiry/rotation/revocation, secure storage, context schema,
   local compatibility, browser/CLI flows, audit events, and recovery.
2. Characterize every server/client/CLI/config/GraphQL/SSE surface and add
   compatibility/security fixtures before implementation.
3. Implement server authentication and authorization at one central boundary,
   propagate typed identity/capabilities, and default-deny protected actions.
4. Implement native-TLS client credential injection and OS-appropriate secure
   storage. Redact tokens and auth headers from all diagnostics/telemetry.
5. Add atomic named-context management with explicit active context, collision
   handling, permission-safe files, and deterministic JSON/human output.
6. Test expiry, revocation, rotation, insufficient scope, clock boundaries,
   malformed credentials, local mode, remote failure, and context corruption.

## Test Plan

- Threat-model/ADR review and contract snapshots.
- Authn/authz positive and negative matrix by capability.
- Token leak scans across logs, telemetry, errors, bundles, process list, shell
  history fixtures, config export, and crash output.
- Context command parser/dispatch, atomicity, permissions, corruption/recovery,
  concurrency, and compatibility tests.
- Native-TLS integration and certificate failure tests on supported platforms.

## Done Criteria

- [x] Operator opens V2 scope and approves the ADR (minimal contract v1).
- [x] Protected surfaces default-deny when token configured (single operator capability).
- [x] Token lifecycle documented (issue/store/rotate out-of-band; no leak in 401/banner/show).
- [x] Named context commands are atomic, permission-safe, and scriptable.
- [x] Existing approved local behavior remains compatible.
- [x] Native-TLS retained; negative auth matrix for bearer pass (unit + m109).

## STOP Conditions

- V2 scope or the ADR is not approved.
- Credential storage/transport would use rustls, plaintext remote hops, or
  repository/config/CLI-argument secrets.
- Authorization is duplicated inconsistently at individual resolvers/routes.
- Existing local compatibility cannot be preserved without a product decision.

## Remove When

Delete this plan and index row when V2 is explicitly opened and the approved
authentication/context contract is implemented and security-verified, or when
the operator explicitly rejects the scope.

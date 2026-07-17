# Plan 112: Decide and prove the product MCP surface

> **Executor instructions**: Projection equivalence from the spike is only one
> gate. Do not rename/package the spike as product, expose raw GraphQL/storage,
> add mutating tools, or enable repository-provided client config. Execution
> starts only after an explicit operator ship decision.

## Status

- **Priority**: P1 when opened
- **Effort**: XL
- **Risk**: CRITICAL
- **Depends on**: 099, 104, 111; 109 before any remote transport
- **Category**: agent surface / MCP / security
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED
- **Blocker**: The operator has not opened a product MCP ship decision; the
  current crate is an unshipped stdio projection-equivalence spike.

## Current Evidence

The spike proves byte-equivalent bounded bundle projection for two read-only
tools. It does not prove client registration/discovery, resources, scopes,
remote auth, audit, output limits, protocol drift, capability denial, or client
retention. Its former comparison-only `_meta` raw JSON has been removed and
must not return in product output.

### Preliminary implementation available on `main` (Codex, 2026-07-17)

- `rmcp` 2.2.0 was rechecked as the latest stable crate. The spike remains
  local stdio-only with no SDK HTTP/TLS features; its dependency graph contains
  no `rustls` package.
- Comparison-only raw canonical JSON was removed from MCP `_meta` while the
  canonical object remains in `structuredContent`.
- Tests assert tools are advertised while roots, sampling, elicitation,
  prompts, resources, tasks, and other unapproved capabilities remain absent.
- Discovery tests lock the exact two-tool preliminary catalog and require
  closed input schemas with one mandatory anchor each.
- GraphQL anchors use variables rather than copied partial string escaping,
  removing the control-character/injection class at this adapter boundary.
- Bundle calls explicitly request a 4,000-token canonical budget rather than
  inheriting the HTTP API's 10,000-token default; full oversized-summary and
  resource-reference behavior remains unfinished.
- Issue-context discovery advertises the checked-in bundle-v2 schema as the MCP
  `outputSchema`; client discovery/conformance evidence remains unfinished.
- Invalid checked-in bundle schema content now advertises a deny-all schema
  rather than silently widening discovery with an open empty schema; discovery
  locks its canonical identifier and closed-object policy.
- Stdio startup requires an explicit `--allow-local-stdio` command-line trust
  decision; environment and repository files cannot supply that opt-in.
- API origins are restricted to credential-free plaintext loopback HTTP;
  arbitrary hosts, TLS, URL credentials, paths, queries, and fragments fail
  closed until Plan 109 supplies the remote transport contract.
- Malformed bundle contracts and upstream failures return stable secret-free
  MCP errors; the adapter no longer silently substitutes an empty object or
  forwards raw upstream error strings.
- Projection-equivalence mismatch diagnostics no longer print evidence
  previews (or byte-slice UTF-8); they expose only lengths, first differing
  byte, and SHA-256 digests.
- Bundle output fails closed unless the separately projected GraphQL canonical
  hash exactly matches the hash embedded in the bundle-v2 object.
- All GraphQL responses stream through a hard 1 MiB ceiling before JSON parse;
  chunked responses cannot bypass the bound or partially append overflow bytes.
- Server discovery explicitly pins stable MCP `2025-11-25` rather than the
  SDK's moving `LATEST` constant; client skew fixtures remain unfinished.
- The GraphQL client denies redirects (preventing loopback-to-remote escape)
  and enforces 5-second connect plus 30-second total request deadlines.
- Final structured content plus compatibility text is capped at 128 KiB with
  saturating accounting. Oversized results currently fail closed; the required
  bounded-summary plus approved-resource-reference path remains unfinished.
- Agent-session GraphQL now uses a static syntactically valid variable query
  (removing stale doubled format-string braces), and bundle projections reject
  missing/null/non-string contract fields instead of substituting empty text.
- Loopback-origin enforcement now lives inside the GraphQL client constructor,
  preventing future internal callers from bypassing the CLI-level validation.
- Only literal loopback IPs are accepted; even `localhost` is rejected so DNS
  or hosts-file configuration cannot redirect the supposedly local transport.
- Both preliminary tools advertise and test exact MCP annotations: read-only,
  non-destructive, idempotent, and closed-world.
- Tool anchors are closed-schema and runtime bounded to 1–256 UTF-8 bytes;
  stable invalid-parameter errors do not echo attacker-controlled values.
- Agent-session output now strictly decodes into a closed typed projection and
  advertises its generated MCP `outputSchema`; arbitrary GraphQL fields cannot
  pass through to clients.
- An in-memory stdio-equivalent wire fixture now proves MCP initialization,
  stable protocol negotiation, and the exact `tools/list` catalog. Codex and
  Claude registration/discovery fixtures remain unfinished.
- The wire fixture also invokes representative forbidden shell and management
  names and requires protocol-level `tool not found` failures.
- Missing bundles/sessions now map to stable MCP resource-not-found codes,
  distinct from secret-free internal transport/parse failures.
- Initialization overrides the SDK's echo-any-known-version behavior: only the
  reviewed `2024-11-05` through `2025-11-25` revisions are accepted; future
  `2026-07-28` and unknown versions fail closed on the wire.
- Wire fixtures negotiate every reviewed legacy revision and assert the server
  returns that exact revision, while the future-version denial remains
  protocol-level; claimed-client skew evidence remains unfinished.

This is preliminary hardening, not completion. The next executor must still
define scopes/install trust, graduate or remove the spike, implement bounded
resources and audit/OTel evidence, and run both claimed-client fixtures plus
the full negative matrices below.

## Scope

In scope after a GO decision:

- An explicit local-stdio-only versus authenticated-remote transport contract.
- Product crate/config/package lifecycle and deletion/quarantine of the spike.
- Read-only bundle tools/resources, client conformance, authorization, audit,
  bounds, redaction, protocol/capability policy, and retention documentation.

Out of scope:

- Shell, SQL, deploy, rollback, delete, alert/dashboard/user/role/pipeline CRUD.
- Raw refs without an approved sensitive scope and plan 111 coverage.
- Auto-enable/trust from repository config or unauthenticated remote transport.

## Steps After Trigger

### Step 1: Decide the product and transport boundary

Approve tool/resource catalog, supported clients, local install/trust behavior,
remote inclusion/deferment, scope names, output budget, audit schema, raw-ref
policy, and spike graduate/delete decision. If remote MCP is included, plan 109
must provide issuer/audience/PKCE/resource-indicator/token-passthrough rules.

### Step 2: Prove client and discovery behavior

For each claimed Codex/Claude client, record configuration source/precedence,
trust prompts, credentials, `tools/list`, tool search/deferred loading,
negative-tool absence, `resources/list/read/templates`, attachment behavior,
structured content, and oversized-output handling.

### Step 3: Enforce scope and transport safety

Require `evidence:read` and a separately approved sensitive scope for any raw
reference. Local stdio has explicit install/trust and approved credential
sources. Remote transport, if authorized, uses native TLS plus protected
resource metadata, resource indicators, audience, PKCE, and no token
passthrough. Missing/invalid scope fails closed.

### Step 4: Enforce evidence and audit invariants

Every result comes from plan 111's bounded canonical bundle. Test source-field
policy, redaction, output budget with summary/resource refs, and one audit row
plus OTel span per call containing caller/tool/scopes/bundle/status/policy but
no sensitive evidence.

### Step 5: Pin protocol, capabilities, and client retention

Record stable/observed MCP versions and eliminate session-ID assumptions.
Deny/audit roots, sampling, elicitation, tasks, and other unapproved
capabilities. Document Codex memory and Claude file/resource persistence; keep
sensitive evidence out of memory and persisted client artifacts.

### Step 6: Graduate or remove the spike

Move only proven code into a deliberate product crate and permanent projection
fixture, or delete/quarantine the spike on NO-GO. Remove comparison-only raw
metadata. Do not update the user agent guide until all selected ship gates pass.

## Test Plan

- Claimed-client registration, trust, discovery, invocation, resource, and
  retention fixtures.
- Scope/auth/PKCE/audience/token-passthrough negative matrix where remote exists.
- Canonical projection/redaction/source-field/output-budget equivalence.
- Audit/OTel no-secret fixtures.
- Permanent negative tool/management catalog and capability assertions.
- Protocol-version/skew and no-session-dependence tests.

## Done Criteria

- [ ] Operator records GO/NO-GO and the supported transport/client contract.
- [ ] Client fixture and tool-discovery fixture pass for every claimed client.
- [ ] Resource read/templates/attachment behavior is bounded and raw-ref safe.
- [ ] Scope defaults deny; remote auth gates pass if remote is supported.
- [ ] Local stdio install/trust/credential behavior is explicit and safe.
- [ ] Redaction and source-field fixtures pass across structured/text output.
- [ ] Oversized output becomes bounded summaries plus approved resource refs.
- [ ] Every call produces safe audit and OTel evidence.
- [ ] Negative tool and management catalogs remain permanently empty.
- [ ] Protocol and unapproved capability fixtures fail closed.
- [ ] Client retention behavior is documented and sensitive-safe.
- [ ] The spike is graduated deliberately or deleted/quarantined.

## STOP Conditions

- Product MCP is not explicitly opened by the operator.
- Plans 099/104/111 are incomplete, or remote work starts before plan 109.
- A client requires repository auto-trust, raw GraphQL/storage, unbounded text,
  token passthrough, rustls, or an unapproved mutating capability.
- Projection or audit output can disclose a seeded secret.

## Remove When

Delete this plan and index row after an explicit NO-GO removes/quarantines the
spike, or a GO delivers every selected product/client/security gate and the
spike has a terminal disposition.

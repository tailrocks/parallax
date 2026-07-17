# Plan 104: Reconcile the evidence-bundle product contract

> **Executor instructions**: This is a contract decision before a refactor.
> Do not silently make prose match code or code match prose. Inventory consumers,
> choose the canonical model with explicit operator/product approval, version
> incompatibilities, and keep CLI/GraphQL/MCP projections equivalent.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 093, 099
- **Category**: product contract / schema / evidence safety
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED — operator must approve Option A, B, C, or a replacement contract

## Current Blocker (2026-07-13)

Step 1 is complete at `71a176a`: the observable consumer inventory, shipped
golden JSON/hash, and fail-closed decision fixture are committed. The draft
[decision record](../docs/research/decisions/evidence-bundle-contract.md)
contains three fully scoped choices. Its required approval fields remain
`UNRESOLVED`, and the gate demonstrably rejects it.

The operator/product owner must explicitly choose Option A (`bundle-v1`),
Option B (research graph as `bundle-v2`), Option C (versioned envelope around
the V1 dossier), or reject all three with a replacement scope; they must also
name approver and approval date. Steps 3–5 are forbidden until then.

Fresh audit on 2026-07-15 at `691cf17`: the decision record still has
`status = "pending-operator-approval"`, while `canonical_model`,
`contract_version`, `compatibility_window`, `migration_behavior`,
`approved_by`, and `approval_date` remain `UNRESOLVED`. No later operator
message in the active plan program selects A, B, C, or a replacement. The
fail-closed decision gate and Step-2 STOP condition therefore remain active.

## Preliminary Step-2 execution landed (helper agent, 2026-07-17) — peer verify + continue

The operator's unblock directive (2026-07-17, `plans/README.md`) approved
**Option C** (approver alexey@chainargos.com). The decision record
`docs/research/decisions/evidence-bundle-contract.md` is now filled: status
approved, all seven front-matter fields resolved, every "Required field
decisions" item answered (envelope model, `bundle-v2` version string, `{kind,
id}` anchor, ISO-8601 envelope timestamps over the untouched v1 payload, v1
redaction report retained, RFC 8785 version-scoped hash, permanent v1 read
window, fail-closed deterministic conversion, unknown-version rejection),
plus rejected alternatives and the accepted hybrid-permanence risk. The
xtask gate test now asserts the real record passes and that unapproved/
unresolved mutations still fail closed (77 xtask tests, strict clippy, fmt
green). Peer: verify the record against the directive, then execute Steps
3-5 (versioned migration design, single source of truth, equivalence proofs)
— none of that implementation exists yet.

### Step-3 preliminary core also landed (same helper agent)

`crates/parallax-evidence/src/bundle/v2.rs`: `EnvelopeV2` (CloudEvents-profile
envelope; ISO-8601 UTC envelope times via `time`/Rfc3339; v1 dossier verbatim
as `data`), fail-closed deterministic `envelope_v1` conversion
(MissingProject/MissingWindow), `document_version` reader dispatch rejecting
unknown/malformed versions, and the version-scoped `sha256-jcs:` hash over
JCS-canonicalized content (v1 exclusion semantics mirrored: hash, generator,
per-request `bounded`). Tests cover determinism, version scoping vs the v1
`sha256:` hash, fail-closed conversion, unknown-version rejection, and
bounding-exclusion. Caveat recorded in-code: JCS number serialization uses
serde_json/ryu shortest round-trip — exact RFC 8785 ES6 exponent edge cases
are Step-5 property-test work before v2 becomes default emit. Peer owns:
JSON Schema for v2, GraphQL/CLI/MCP projection wiring, equivalence and
property tests (Steps 4-5).

## Why

The research model and shipped `bundle-v1` schema describe materially different
products. Research specifies a CloudEvents-like envelope, typed node/edge
graph, manifests, access policy, and structured timestamps. Code ships a flat
dossier with string nanoseconds, section-based correlation, bounded-token
metadata, preformatted logs, and a different redaction report. Plan 082 froze
the shipped shape but intentionally did not resolve the divergence.

## Known Divergences To Decide

| Area | Research model | Shipped `bundle-v1` |
|------|----------------|---------------------|
| Envelope | `bundle_id`, schema ref, generated time, object generator/project, window | `schema_version`, string generator, flat sections |
| Correlation | Typed nodes/edges plus projection/query manifests | `issue`/`run`/`trace`/metrics/logs sections |
| Version | Semver-like `0.1.0` | `bundle-v1` |
| Redaction | Rich `redaction_report` | `redaction { policy, redacted_counts }` |
| Anchor | `{ type, id }` | `{ kind, id }` |
| Time/logs | ISO-8601 and structured frames | Nanosecond strings and formatted log strings |
| Evidence policy | access/source/raw refs/deploy/agent/code-change nodes | bounded token/drop metadata, hypotheses, missing evidence |

## Scope

- Consumer and compatibility inventory across Rust, schema, CLI, GraphQL,
  MCP spike, fixtures, docs, prompts, and hashes.
- Explicit canonical-model decision and migration/version strategy.
- Schema/model/serializer/redactor/hash/render/projection changes required by
  that decision.
- Deterministic bounding, sanitization, and equivalence tests.

Out of scope:

- Adding agent access to raw GraphQL/SSE.
- Custom Greptime raw-signal tables.
- Unversioned breaking changes or preserving contradictory documents.
- Expanding causal inference beyond the approved evidence contract.

## Decision Gate

Until the operator approves a replacement, shipped immutable `bundle-v1` is the
only canonical read/write/hash contract. Characterization and proposal work may
run, but Steps 3-5 may not start. Step 2 must produce an operator-approved
`docs/research/decisions/evidence-bundle-contract.md` naming the selected model,
version, compatibility window, migration behavior, and approval date. Extend the
repository decision-policy fixture so missing, draft, rejected, or incomplete
approval fails closed.

If approval is unavailable, rejects every proposal, or changes schema/migration
scope beyond this plan, set this plan to `BLOCKED` with the exact decision needed
and stop. Do not choose on the operator's behalf or make code/prose silently
converge.

## Steps

### Step 1: Characterize every observable contract

Capture canonical fixtures from bundle JSON, Markdown, GraphQL, CLI JSON,
schema validation, canonical hashes, redaction, and MCP projection. Enumerate
all readers and stored/persisted examples. Separate intentional shipped behavior
from aspirational research text and identify compatibility obligations.

### Step 2: Record the product decision

Write a decision record that selects the canonical envelope/correlation model,
version syntax, anchor, timestamps, log representation, redaction report,
bounding fields, hypotheses, manifests/access metadata, and treatment of
deploy/code-change/agent evidence. Include rejected alternatives, migration
cost, agent-safety implications, and exact compatibility promise. Obtain the
required operator/product approval before changing the schema.

**Verify**: the decision-policy gate reports one approved canonical model and
version with no unresolved required field. Otherwise mark `BLOCKED`; do not run
Step 3.

### Step 3: Design versioned migration

If the canonical contract differs from `bundle-v1`, allocate a new immutable
version and explicit reader/writer behavior. Define whether old bundles remain
readable, how hashes are version-scoped, whether conversion is lossless, and
how unknown versions fail. Do not overload `bundle-v1` with a new shape.

### Step 4: Implement one source of truth

Align Rust types, JSON Schema, serializers, redaction, bounding, canonical
hashing, Markdown, GraphQL, CLI, MCP projection, examples, and research/spec
claims. Generate or validate derivative artifacts from the canonical model.
Keep external/untrusted text delimited and sanitized at every agent-visible
projection.

### Step 5: Prove compatibility, equivalence, and bounds

Golden-test every supported version and conversion. Re-run CLI/MCP byte or
semantic projection equivalence as appropriate. Property-test deterministic
hashing/redaction/bounding. Test malicious telemetry, unknown fields/versions,
maximal graphs/sections, dropped evidence counts, and timestamp extremes.

## Test Plan

- Consumer inventory and pre-change golden fixtures.
- JSON Schema positive/negative fixtures for each supported version.
- Reader/writer/converter compatibility matrix.
- CLI, GraphQL, Markdown, and MCP projection equivalence.
- Canonical-hash stability and redaction/bounding properties.
- Malicious agent-instruction and secret-bearing telemetry fixtures.

## Done Criteria

- [ ] An approved decision resolves every known divergence explicitly.
- [ ] Breaking shape changes use a new immutable version and migration policy.
- [ ] Types, schemas, serializers, hashes, redaction, projections, and docs agree.
- [ ] Every supported old/new fixture has defined read/write/convert behavior.
- [ ] CLI/GraphQL/MCP expose only the approved sanitized projection.
- [ ] Hashing, ordering, bounding, and redaction are deterministic and tested.
- [ ] Unknown/malformed versions and malicious content fail or sanitize safely.

## STOP Conditions

- The Step-2 operator approval is missing, draft, rejected, ambiguous, or widens
  the implementation beyond this plan.

- No authorized canonical-model decision exists.
- A proposed edit mutates `bundle-v1` incompatibly without a new version.
- A required consumer or persisted fixture cannot be inventoried.
- Projection equivalence would expose raw unredacted GraphQL/SSE data to agents.
- Hash/redaction semantics cannot remain deterministic and version-scoped.

## Remove When

Delete this plan and index row when one approved, versioned evidence contract
is enforced across implementation, schema, projections, tests, and docs.

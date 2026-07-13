+++
schema_version = 1
status = "pending-operator-approval"
canonical_model = "UNRESOLVED"
contract_version = "UNRESOLVED"
compatibility_window = "UNRESOLVED"
migration_behavior = "UNRESOLVED"
approved_by = "UNRESOLVED"
approval_date = "UNRESOLVED"
+++

# Evidence-bundle canonical contract decision

Decision prepared: 2026-07-13  
Decision owner: Parallax operator/product owner

## Approval required

Plan 104 prohibits selecting a canonical model without explicit operator
approval. Choose exactly one option below (or reject all with a replacement
scope), and provide the approver identity and approval date. Until then,
`bundle-v1` remains the only canonical shipped read/write/hash contract and no
schema/model migration is authorized.

## Option A — Ratify shipped `bundle-v1` (recommended near-term)

Keep the flat section dossier, `{kind,id}` anchor, nanos strings, formatted log
lines, current version-scoped sorted-key hash, bounding report, and qualitative
hypotheses as the canonical V1 contract. Plan 111 may add only fields proven
additive under the existing schema policy. Relabel the CloudEvents/typed-graph
research model as a future V2 proposal requiring a separate approved plan.

Compatibility: permanent writer/schema/hash fixtures for `bundle-v1`; no
converter because no second version ships. Lowest migration and agent-safety
risk, but defers the graph/profile interoperability thesis.

## Option B — Ship the research graph as immutable `bundle-v2`

Allocate a new SchemaVer/CloudEvents-profile envelope with typed nodes/edges,
schema refs, structured timestamps/logs, manifests, access/source policy, and
RFC 8785 canonicalization. Keep `bundle-v1` readable/emit-capable for an
operator-selected window; make GraphQL/CLI/MCP negotiate or explicitly request
versions; define lossy v1→v2 conversion where v1 lacks graph/access metadata.

Compatibility: highest implementation and review cost. It advances the open
schema thesis but expands Plan 104 to every consumer and creates a dual-version
security surface.

## Option C — Ship a `bundle-v2` envelope around the V1 dossier

Add the CloudEvents/schema/project/window/access envelope and version-scoped JCS
hash, but retain the current dossier as a typed `data` payload. Defer nodes and
edges until evidence owners exist. Maintain explicit v1 and v2 writers/readers;
v1→v2 envelope conversion is deterministic where project/window inputs exist
and fails otherwise.

Compatibility: moderate migration cost and a standards-shaped envelope, but it
risks making the intermediate hybrid permanent and still requires dual-version
projection/equivalence work.

## Required field decisions

Approval must explicitly name: envelope/correlation model, version syntax,
anchor shape, timestamp/log representation, redaction report, bounding and
hypothesis fields, manifest/access metadata, deploy/code-change/agent evidence
treatment, canonicalization/hash algorithm, old-version support window,
conversion behavior, unknown-version behavior, approver, and date.

## Evidence

The complete observable consumer and compatibility inventory is in
[the Plan 104 inventory](../validation/2026-07-13-plan-104-bundle-contract-inventory.md).

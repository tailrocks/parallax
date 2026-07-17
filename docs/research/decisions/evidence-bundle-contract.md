+++
schema_version = 1
status = "approved"
canonical_model = "bundle-v2 versioned envelope around the immutable v1 dossier (Option C)"
contract_version = "bundle-v2"
compatibility_window = "bundle-v1 stays permanently readable and hash-verifiable; dual v1/v2 writers until plan-104 Step-5 equivalence passes, then v2 becomes the default emit with v1 emit retained for the V1 product line"
migration_behavior = "deterministic v1-to-v2 envelope conversion when project/window inputs exist, fail-closed otherwise; unknown or malformed versions are rejected, never coerced"
approved_by = "alexey@chainargos.com"
approval_date = "2026-07-17"
+++

# Evidence-bundle canonical contract decision

**Implementation status (2026-07-17): SHIPPED.** `bundle-v1` and the
`envelope-v2` wrapper are implemented with assembly, bounding, ranking,
redaction, hashing, Markdown projection, and JSON Schemas at
`schema/evidence-bundle.v1.schema.json` and
`schema/evidence-bundle.v2.schema.json`. Plan 104 references below are closed
historical provenance.

Decision prepared: 2026-07-13  
Decision approved: 2026-07-17 (operator unblock directive recorded in
`plans/README.md`, Triggered Or Operator-Blocked Work: "DECIDED … Option C —
versioned envelope around the V1 dossier; approver alexey@chainargos.com;
executor fills the decision record and proceeds")  
Decision owner: Parallax operator/product owner

## Approved decision (Option C)

Ship an immutable **`bundle-v2`** contract that wraps the shipped V1 dossier
as a typed payload inside a standards-shaped envelope. `bundle-v1` remains an
immutable, permanently readable contract; it is never overloaded with new
shape.

Required field decisions (closed plan 104 Step 2):

- **Envelope/correlation model**: CloudEvents-profile envelope —
  `bundle_id`, schema reference, generated timestamp, generator identity,
  project, time window, and access metadata — around the existing flat V1
  dossier carried verbatim as the typed `data` payload. Typed node/edge
  graphs, and the query/projection manifests that depend on them, are
  **deferred** until evidence owners exist (plans 120/121/124 lineage); the
  envelope reserves no speculative fields for them.
- **Version syntax**: immutable string identifiers (`bundle-v1`,
  `bundle-v2`), matching the shipped convention; no semver ranges.
- **Anchor shape**: the V1 `{ kind, id }` anchor is canonical (it lives in
  the dossier payload and is untouched by the envelope).
- **Timestamps/logs**: envelope timestamps are ISO-8601 UTC; the dossier
  payload keeps its nanosecond strings and preformatted log lines unchanged
  (payload immutability is the point of Option C).
- **Redaction report**: the V1 `redaction { policy, redacted_counts }`
  stays canonical inside the payload; the envelope adds no second redaction
  surface.
- **Bounding/hypotheses**: V1 bounded-token/drop metadata, hypotheses, and
  missing-evidence fields remain as shipped, inside the payload.
- **Manifests/access metadata**: only envelope-level access metadata ships
  in v2 (generator, project, window, access policy label); source/raw-ref
  manifests wait for the deferred graph model.
- **Deploy/code-change/agent evidence**: not part of v2; arrives only with
  the deferred graph model under its own approved plan.
- **Canonicalization/hash**: v2 hashes use RFC 8785 (JCS) canonicalization,
  version-scoped (`bundle-v2` hash never validates a v1 document); the
  existing v1 sorted-key hash stays valid for v1 documents forever.
- **Old-version support window**: `bundle-v1` read + hash-verify support is
  permanent. Both writers exist through plan-104 Steps 3–5; after Step-5
  equivalence proof, v2 is the default emit and v1 emit remains available
  for the V1 product line.
- **Conversion behavior**: v1→v2 envelope conversion is deterministic when
  the project/window inputs exist, and **fails closed** when they do not —
  no fabricated envelope fields. Conversion is lossless with respect to the
  dossier payload by construction.
- **Unknown-version behavior**: readers reject unknown or malformed
  `schema_version`/contract identifiers with an explicit error; no
  best-effort parsing on any agent-visible projection.

## Rejected alternatives

- **Option A — ratify `bundle-v1` only**: rejected as terminal; it defers
  the interoperability envelope indefinitely and leaves the research model
  permanently contradicting the shipped shape.
- **Option B — full research graph as `bundle-v2`**: rejected for now; the
  typed node/edge graph has no evidence owners yet, and shipping it would
  expand plan 104 across every consumer while creating a dual-version
  security surface for structures nothing produces.

Risk accepted with Option C: the hybrid envelope-around-dossier could become
permanent. Mitigation: the graph model stays a recorded V2+ proposal and any
future adoption requires its own approved plan; nothing in v2 blocks it.

## Prior option descriptions (for the record)

### Option A — Ratify shipped `bundle-v1`

Keep the flat section dossier, `{kind,id}` anchor, nanos strings, formatted log
lines, current version-scoped sorted-key hash, bounding report, and qualitative
hypotheses as the canonical V1 contract. Plan 111 may add only fields proven
additive under the existing schema policy. Relabel the CloudEvents/typed-graph
research model as a future V2 proposal requiring a separate approved plan.

### Option B — Ship the research graph as immutable `bundle-v2`

Allocate a new SchemaVer/CloudEvents-profile envelope with typed nodes/edges,
schema refs, structured timestamps/logs, manifests, access/source policy, and
RFC 8785 canonicalization. Keep `bundle-v1` readable/emit-capable for an
operator-selected window; make GraphQL/CLI/MCP negotiate or explicitly request
versions; define lossy v1→v2 conversion where v1 lacks graph/access metadata.

### Option C — Ship a `bundle-v2` envelope around the V1 dossier (approved)

Add the CloudEvents/schema/project/window/access envelope and version-scoped JCS
hash, but retain the current dossier as a typed `data` payload. Defer nodes and
edges until evidence owners exist. Maintain explicit v1 and v2 writers/readers;
v1→v2 envelope conversion is deterministic where project/window inputs exist
and fails otherwise.

## Evidence

The complete observable consumer and compatibility inventory is in
[the Plan 104 inventory](../validation/2026-07-13-plan-104-bundle-contract-inventory.md).

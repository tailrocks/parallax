# Plan 104 evidence-bundle contract inventory

Inventory date: 2026-07-13  
Characterized commit: `251f54b`

## Observable shipped contract

The shipped writer is `parallax_evidence::bundle::assemble`. It serializes the
Rust `Bundle` closure with `serde_json`; `schema_version` is `bundle-v1` and the
validator artifact is `schema/evidence-bundle.v1.schema.json` (JSON Schema
2020-12). The checked-in
[`bundle-v1` golden](../../../crates/parallax-evidence/fixtures/bundle-v1-golden.json)
freezes the complete JSON value, including current redaction/bounding reports
and canonical hash.

Canonical hashing sorts JSON object keys recursively, preserves array order,
and excludes `canonical_hash`, `generator`, and `bounded`. It is deterministic
for the shipped serde value but is not RFC 8785 JCS and is not a general bundle
reader/canonicalizer. The repository writes bundles on demand; no product table
persists bundle JSON.

## Consumer and compatibility inventory

| Consumer | Observable dependency | Compatibility obligation |
| --- | --- | --- |
| Rust evidence assembly | Every `Bundle`/section field, ordering, bounding, redaction, ranking, hash exclusions | Existing `bundle-v1` meanings are immutable; additive schema changes still affect hashes |
| JSON Schema | Required fields, nanos strings, formatted log strings, free policy/kind values | Existing positive fixtures must continue validating; malformed shapes fail |
| GraphQL `bundle` | Returns `json`, `markdown`, `canonicalHash`; accepts exactly one issue/run/trace anchor | Field names and nullable/not-found behavior are SDL compatibility |
| CLI issue/run context | Prints GraphQL JSON verbatim or deterministic Markdown | JSON bytes/value and Markdown safety delimiters are user/agent behavior |
| UI run page | Requests bundle JSON preview and hash | Must tolerate only the supported schema/version or fail visibly |
| MCP spike | Calls GraphQL `bundle`, parses the projection, and compares embedded/recomputed/CLI hashes | Proof is byte/hash equivalence only; it is not a product reader promise |
| Server acceptance/performance | Redaction, token bounds, hypothesis order, metric windows, warm assembly latency | Security/budget semantics cannot silently weaken |
| A1 evaluation corpus/runbooks | Treat canonical JSON/hash as the comparison unit | Historical rows need their schema/hash procedure recorded |
| Research/schema prose | Describes a different CloudEvents/profile graph contract | Must be labeled proposal unless an approved new version ships |

The source scan found 219 references across 38 unique files in Rust, UI,
schemas, docs, prompts, plans, and validation procedures. There is no generic
Rust reader for persisted old bundles, no conversion API, and no production MCP
surface. Compatibility today is therefore writer/schema/projection/hash
compatibility, plus research/evaluation reproducibility.

## Divergence decisions required

| Area | `bundle-v1` shipped | Research proposal | Required operator selection |
| --- | --- | --- | --- |
| Envelope/version | Flat `bundle-v1`, string generator | SchemaVer/CloudEvents-like envelope and schema ref | Freeze v1 or allocate v2 |
| Correlation | Issue/run/trace/metric/log sections | Typed nodes/edges and manifests | Sections, graph, or versioned hybrid |
| Anchor | `{kind,id}` | `{type,id}` | Preserve or change only in v2 |
| Time/logs | Nanos strings; formatted log strings | RFC 3339; structured frames | Preserve or version |
| Redaction | `{policy,redacted_counts}` | Rich report, source-field/access policy | Plan 111 in v1 additive fields or v2 replacement |
| Bounds/hypotheses | Explicit token/drop report and qualitative ranked hypotheses | Cited graph hypotheses/access/raw refs | Preserve, extend, or replace in v2 |
| New evidence | Not represented | deploy/code-change/agent/action nodes | Exclude, add sections, or graph nodes |
| Canonicalization | Local sorted-key algorithm | RFC 8785 JCS | Keep version-scoped v1 hash or introduce v2 hash algorithm |

## Characterization gates

- Golden JSON value and canonical hash are checked in and test-owned.
- Existing schema positives/negatives, Markdown injection delimiter, redaction,
  bounding, GraphQL/CLI acceptance, and MCP equivalence fixtures remain the
  pre-migration baseline.
- Unknown-version reader behavior is currently undefined because there is no
  product reader; this must be decided before any second version.


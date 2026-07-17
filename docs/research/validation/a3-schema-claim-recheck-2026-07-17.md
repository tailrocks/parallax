# A3 claim-level recheck (2026-07-17, pass 60 + pass 100)

<!-- markdownlint-disable MD013 -->

**Target:** [a3-schema-corpus.md](a3-schema-corpus.md) present-tense claim
`schema_draft` / "no JSON Schema artifact".

## Verdict

| Old claim (May body) | Reality 2026-07-17 | Decision |
| --- | --- | --- |
| No released JSON Schema | **Shipped** `schema/evidence-bundle.v1.schema.json` + `v2` | **Replace** |
| No validator | **jsonschema** crate validates in `parallax-evidence` tests + plan_093 baseline | **Replace** |
| No fixtures | `crates/parallax-evidence/fixtures/bundle-v1-golden.json` | **Replace** |
| No versioning policy | `schema/README.md` additive-only v1 policy | **Replace** |
| MCP outputSchema fixture absent | MCP crate **loads** v1/v2 schema files (`parallax-mcp/src/server.rs`) | **Narrow** — not a full public MCP outputSchema marketplace entry, but in-tree |
| A3 adoption / moat proven | **Still false** — zero external integration ledger rows | **Keep unproven** |
| SigNoz open investigation format = schema | Still **product language**, not published portable schema (prior checks; no new schema found 2026-07-17) | **Keep** |

### Pass 100 (2026-07-17) — external adoption probe

| Probe | Result |
| --- | --- |
| GitHub code search for `$id` host `github.com/tailrocks/parallax/schema/evidence-bundle` | **Only** `tailrocks/parallax` hits (schema files + MCP tests). **No** non-operator consumer. |
| Generic filename `evidence-bundle.v1.schema.json` | **Many unrelated projects** use that *name* (RoutePilot, Mischief-Manager, Entroping, actuarial SDKs, etc.) with **different `$id`s** — name collision only, **not** Parallax adoption. |
| Phrase "PARALLAX evidence bundle" outside Tailrocks | Hits on `arjun7n9s/Parallax` (1★, unrelated Python “band” project — **homonym**, not schema import). |
| Public adoption ledgers | Still **absent** (`schema-adoption-results.md` / `corpus-outcome-results.md` not present). |

**Claim stack unchanged:** `schema_artifacts_shipped` + `schema_adoption_none` +
`corpus_empty_public`. **Do not** count foreign `evidence-bundle.v1` files as A3
gravity.

**Correct claim stack:**

1. `schema_artifacts_shipped` — internal conformance possible.
2. `schema_adoption_none` — no external gravity.
3. `corpus_empty_public` — no public outcome corpus at thresholds.
4. A3 moat = **unproven**.

## Artifact inventory (repo paths)

| Artifact | Path |
| --- | --- |
| Bundle v1 JSON Schema | `schema/evidence-bundle.v1.schema.json` |
| Envelope v2 JSON Schema | `schema/evidence-bundle.v2.schema.json` |
| Versioning prose | `schema/README.md` |
| Golden fixture | `crates/parallax-evidence/fixtures/bundle-v1-golden.json` |
| Validator tests | `crates/parallax-evidence/src/bundle/tests.rs` |
| MCP schema load | `crates/parallax-mcp/src/server.rs` |
| Research prose | `docs/research/architecture/evidence-bundle-schema.md` |
| Decision | `docs/research/decisions/evidence-bundle-contract.md` |

Historical gate path `schemas/evidence-bundle/v0.1.0/schema.json` was **never**
the shipped location; do not block A3 on that exact string.

## What would move claim levels

| To | Need |
| --- | --- |
| `schema_release_logged` | Public ledger row with schema hash + validator version + fixture hash + git commit |
| `schema_adoption_partial` | ≥1 non-operator tool/team depends on the schema for a workflow Parallax does not control (gate table 180-day spirit) |
| `corpus_phase0` | 5–10 hand-built labeled outcome rows in public ledger |
| A3 moat language allowed | Gate thresholds in a3-schema-corpus.md fully met with ledger evidence |

## ClickHouse version pin (same pass, no bench)

Per GitHub releases (non-prerelease, non-LTS `*-stable` tags), feature-line pins
observed 2026-07-17:

- Newest **feature-line** tag by major.minor: **`v26.6.1.1193-stable`** (2026-06-25).
- Newest **stable** patch date among non-LTS: **`v26.5.5.8-stable`** (2026-07-01).

For four-way benches, prefer **latest feature line** → pin **26.6.x**
(`v26.6.1.1193-stable` until a newer 26.6/26.7 stable appears). Do **not** use
LTS `v25.8.*` as the feature comparator. **No performance numbers claimed here.**

## Falsification

- If `schema/` schemas are removed or tests stop validating → drop to
  `schema_draft` again.
- If an external OSS tool publishes a compatible producer against our `$id` →
  log adoption and reassess moat language carefully (one integration ≠ moat).

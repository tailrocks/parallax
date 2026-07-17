# A3 claim-level recheck (2026-07-17, pass 60 + pass 100 + pass 124 + pass 184)

<!-- markdownlint-disable MD013 -->

**Target:** [a3-schema-corpus.md](a3-schema-corpus.md) present-tense claim
`schema_draft` / "no JSON Schema artifact".

## Verdict

| Old claim (May body) | Reality 2026-07-18 (pass 184) | Decision |
| --- | --- | --- |
| No released JSON Schema | **Shipped** `schema/evidence-bundle.v1.schema.json` + `v2` | **Replace** |
| No validator | **jsonschema** crate validates in `parallax-evidence` tests + plan_093 baseline | **Replace** |
| No fixtures | `crates/parallax-evidence/fixtures/bundle-v1-golden.json` (**pass 184:** golden test **ok**) | **Replace** |
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

### Pass 124 (2026-07-17) — `$id` resolve + adoption re-probe

| Probe | Result |
| --- | --- |
| Declared `$id` (v1) | `https://github.com/tailrocks/parallax/schema/evidence-bundle.v1.schema.json` |
| HTTP GET that `$id` URL | **404** — not a live schema document URL (GitHub does not serve files at that path) |
| `raw.githubusercontent.com/.../main/schema/evidence-bundle.v1.schema.json` | **200** JSON — actual fetchable bytes |
| `blob/main/...` HTML | **200** HTML — not machine schema |
| `schema/README.md` policy | Schemas are **self-contained** (no remote `$ref`); offline validation does not require `$id` fetch |
| `gh search code` for `$id` host | Still **only** `tailrocks/parallax` (+ in-tree MCP test). **No** non-operator consumer. |
| Foreign `evidence-bundle.v1.schema.json` names | Still **name collisions** with other `$id`s (RoutePilot, Mischief-Manager, Entroping, bil-actuarial, lupine, etc.) — **not** Parallax gravity |
| Adoption ledger files | Still **absent** |

### Pass 184 (2026-07-18) — adoption + schema liveness re-probe

| Probe | Result |
| --- | --- |
| Declared `$id` HTTP GET | Still **404** |
| `raw.githubusercontent.com/.../main/schema/evidence-bundle.v1.schema.json` | Still **200** |
| Code search `$id` host `github.com/tailrocks/parallax/schema/evidence-bundle` | **total_count 6**, all **`tailrocks/parallax`** — **no** external consumer |
| Filename `evidence-bundle.v1.schema.json` | **total_count 50** — still dominated by **name collisions** / in-tree; **not** Parallax gravity |
| Adoption ledger files | Still **absent** |
| Golden stability test | **`bundle_v1_golden_fixture_is_stable` ok** |
| Industry “evidence/investigation schema” desk scan | GenAI/agent **trace** schemas and OTel GenAI semconv growing; **no** portable multi-signal redacted **investigation evidence-bundle** standard observed (aligns pass **157** OTel #1185 idle) |

### Pass 219 (2026-07-18) — adoption re-probe

| Probe | Result |
| --- | --- |
| Code search `$id` host | Still **total_count 6** (in-tree only) |
| External adoption | Still **zero** |

### Pass 259 (2026-07-18) — adoption + Traceloop neighbor pin

| Probe | Result |
| --- | --- |
| In-repo refs to `github.com/tailrocks/parallax/schema/evidence-bundle` | Still **schema files + PoC schema + MCP test + research notes** only — **no** external consumer ledger |
| Adoption ledger files | Still **absent** |
| Claim stack | Still **`schema_artifacts_shipped` + `schema_adoption_none` + `corpus_empty_public`** |
| [traceloop/openllmetry](https://github.com/traceloop/openllmetry) | **7,307★**; latest **`0.62.1`** (2026-06-28) — GenAI OTel instrumentation peer; **not** Parallax evidence-bundle consumer |

### Pass 273 (2026-07-18) — adoption re-probe + Seer watch

| Probe | Result |
| --- | --- |
| In-repo `$id` host refs | Still **only** this repo (schema + PoC + MCP test + research notes) |
| External adoption ledger | Still **absent** / **zero** external consumers |
| Claim stack | Unchanged: **`schema_artifacts_shipped` + `schema_adoption_none` + `corpus_empty_public`** |
| Seer self-host (adjacent kill) | develop.sentry.dev still **"Seer and other AI & ML features… closed source"** — **UNFIRED** |

### Pass 284 (2026-07-18) — adoption re-probe

| Probe | Result |
| --- | --- |
| In-repo `$id` host file hits | still **6** paths (schema + PoC + MCP test + research notes) — **no** external consumer |
| Claim stack | still **`schema_artifacts_shipped` + `schema_adoption_none` + `corpus_empty_public`** |

**A3 split holds:** schema **artifacts shipped**; **external adoption = zero** (moat unproven).

**Implication:** `$id` is currently an **identifier**, not a **dereferenceable
catalog URL**. That is compatible with self-contained offline validation, but
weakens any claim that third parties can `$ref` the published `$id` without
using raw.githubusercontent (or a future stable docs host). **Does not change**
claim stack: artifacts shipped, adoption none. Optional follow-up (product, not
this research pass): publish a stable raw or docs URL and align `$id`, or
document `$id` as non-resolving URI.

**Claim stack unchanged:** `schema_artifacts_shipped` + `schema_adoption_none` +
`corpus_empty_public`. **Do not** count foreign `evidence-bundle.v1` files as A3
gravity.

**Correct claim stack:**

1. `schema_artifacts_shipped` — internal conformance possible.
2. `schema_adoption_none` — no external gravity.
3. `corpus_empty_public` — no public outcome corpus at thresholds.
4. A3 moat = **unproven**.

### Pass 307 (2026-07-18) — adoption + schema liveness re-probe

| Probe | Result |
| --- | --- |
| Declared `$id` HTTP GET (`github.com/tailrocks/parallax/schema/...`) | Still **404** |
| `raw.githubusercontent.com/.../main/schema/evidence-bundle.v1.schema.json` | Still **200** |
| In-repo paths containing `$id` host string | **6** files (schema v1/v2, a3 notes, MCP tests, poc v0) — **no** external consumer tree |
| Public adoption ledger files | Still **absent** |
| Claim stack | still `schema_artifacts_shipped` + `schema_adoption_none` + `corpus_empty_public` |

**A3 moat still unproven.** Kill "external schema adoption closes moat" still **unfired** (open ≠ failed; adoption is zero, not "failed moat measurement").

### Pass 321 (2026-07-18) — adoption + liveness

| Probe | Result |
| --- | --- |
| Declared `$id` HTTP GET | Still **404** |
| raw.githubusercontent v1 schema | Still **200** |
| In-repo `$id` host file hits | still **6** — no external consumer |
| Adoption ledger | Still **absent** |
| Claim stack | still `schema_artifacts_shipped` + `schema_adoption_none` |

### Pass 332 (2026-07-18) — adoption + liveness

| Probe | Result |
| --- | --- |
| `$id` HTTP | **404** |
| raw v1 schema | **200** |
| In-repo `$id` host hits | still **6** (operator tree) |
| Adoption ledger | **absent** |

### Pass 339 (2026-07-18) — adoption + liveness

| Probe | Result |
| --- | --- |
| `$id` HTTP | **404** |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 348 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 353 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 358 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| `$id` HTTP | **404** |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 364 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 368 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 371 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 374 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 377 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 380 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 383 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

### Pass 386 (2026-07-18) — schema liveness

| Probe | Result |
| --- | --- |
| raw v1 schema | **200** |
| Adoption ledger | **absent** |

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

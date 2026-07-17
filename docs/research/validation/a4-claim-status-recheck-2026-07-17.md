# A4 claim status recheck — product code vs real_pilot ledger (pass 121 + pass 161)

<!-- markdownlint-disable MD013 -->

**Research date:** 2026-07-17 (pass 121); **pass 161 recheck 2026-07-18**  
**Pass:** 121 + **161** (deep-research-parallax indefinite program)  
**Claim under test:** "Deterministic cross-signal correlation is reliable in real, messy telemetry" (assumption A4).  
**Verdict this pass:** product reliability claim remains **`not_measured`**. Query/assembly surfaces and fixture-level derivation exist; **no** `real_pilot` row-level ledger.

## Why this pass

A4 is a standing open gate (agenda A1–A7 set). Pass 62 already flagged empty run
trees; pass 121 re-verifies against current code/docs so A4 is not silently
"closed" by GraphQL feature growth or unified-CLI demos.

## Evidence class

| Layer | Status | Evidence |
| --- | --- | --- |
| Gate design + thresholds | **Present** | [capture/correlation.md](../capture/correlation.md) — rates, false-strong-edge audit, claim levels |
| Ledger *policy* | **Present** | Same note (manifest / per-anchor / repair rows) |
| Ledger *instance* | **Absent** | No `docs/research/correlation-reliability-runs/<run_id>/` (rechecked 2026-07-17; **pass 161:** directory still **missing**) |
| Product correlation *surfaces* | **Shipped** | GraphQL: `tracesByInvocation`, `logsByTrace`, `logsByInvocation`, `linkedTraces`, `story`, `evidenceGaps`, `bundle` (per correlation.md implementation banner; **pass 161 code pin:** resolvers/tests still reference `tracesByInvocation`, `logsByTrace`, `logsByInvocation`, `evidenceGaps` in `parallax-api` / `parallax-server` / CLI) |
| Error derivation / fingerprint | **Shipped (code)** | `parallax-analysis` derive + Sentry/OTLP fingerprint paths; unit tests (cross-source same fingerprint function) |
| Bundle `missing_evidence` | **Shipped (code)** | Product honesty feature — **not** scored A4 audit rows |
| Controlled stack demos | **Exist** | `docs/research/validation/2026-07-unified-cli-observability/` — controlled end-to-end, **not** multi-service messiness `real_pilot` |
| Frontend cross-tier | **Not A4-passed** | Frontend note + correlation frontend targets; no frontend_tiny_default_ready from a dated A4 run |

## Claim levels (policy)

From correlation.md:

| Level | Met? |
| --- | --- |
| `not_measured` | **Yes** for product reliability on real messy telemetry |
| `synthetic_only` | **Arguably partial** if counting unit tests + controlled CLI corpus as synthetic — but **no** formal `correlation-reliability-results.md` publishes that label either |
| `backend_mvp_measured` / `backend_mvp_pass` | **No** — requires real backend anchors + thresholds |
| Frontend / async / baggage gates | **No** dated pass |

### Allowed wording

- "Parallax can join traces/logs/invocations in product queries and assemble bundles with missing-evidence gaps."
- "A4 real-telemetry reliability is **not measured**; no `real_pilot` ledger exists."

### Forbidden wording

- "Correlation is proven reliable in production" / "A4 passes" from resolver existence or playground demos alone.

## Falsification / upgrade path (unchanged)

1. Capture `real_pilot` (or staging) anchors with messy instrumentation.
2. Emit per-run manifest + per-anchor rows + manual false-strong-edge sample.
3. Score vs backend thresholds (e.g. `trace_context_rate` ≥ 80%, `error_in_span_rate` ≥ 70%, `false_strong_edge_rate` ≤ 5%, `missing_evidence_report_rate` = 100%).
4. Publish under `correlation-reliability-runs/<run_id>/` and set claim level + expiry.
5. Frontend continuation and baggage privacy are **separate** promotion gates.

**What would falsify product bet:** strong edges rare / false-strong high on real pilot → product becomes best-effort context + instrumentation coaching, not "evidence-backed reconstruction."

## Pass 161 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Formal results file | **None** found under `docs/research/` |
| Product join surfaces in code | **Still present** (resolver/CLI/test references) |
| Controlled CLI demo tree | **Still exists** (`2026-07-unified-cli-observability/`) — not `real_pilot` |
| Claim level | still **`not_measured`** for real messy telemetry |

Upgrade path in §Falsification unchanged. GraphQL existence ≠ A4 gate.

## Pass 194 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Formal results file | **None** |
| GraphQL join identifiers in crates | **Still present** (`tracesByInvocation`, `logsByTrace`, `evidenceGaps`, …) |
| Claim level | still **`not_measured`** for real messy telemetry |

## Pass 219 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 240 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 247 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `docs/research/correlation-reliability-runs/` | **Still absent** (no run_id ledger) |
| Claim level | still **`not_measured`** for real_pilot product reliability |
| Code paths | still present (analysis derive/fingerprint; not re-measured rates) |

**Not A4 pass:** assembly code ≠ published rate ledger on messy telemetry.

## Pass 267 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 284 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 296 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 302 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 307 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `docs/research/correlation-reliability-runs/` | **Still absent** (no run_id ledger) |
| Claim level | still **`not_measured`** for real_pilot product reliability |
| Product surfaces | still shipped (code); **not** re-measured rates this pass |

**Not A4 pass:** GraphQL/assembly code ≠ published reliability ledger on messy telemetry.

## Pass 319 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 326 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 333 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 339 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 345 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 349 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 353 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 356 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 361 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 365 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 368 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 371 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 374 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 377 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 380 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 383 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 386 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 388 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 390 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Pass 392 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| `correlation-reliability-runs/` | **Still absent** |
| Claim level | still **`not_measured`** |

## Uncertainty

- Did not re-query live GraphQL schema introspection this pass; surface list taken from research implementation banner + **static code references** (pass 161/194).
- Unified-CLI corpus may include useful *synthetic* rates if re-scored under A4 row schema — still would not promote past `synthetic_only` without `real_pilot`.

## Parallax goal fit

Agent bundles depend on **deterministic** edges being common enough to be useful and rare enough false-strong not to hallucinate. Code paths make measurement *possible*; measurement is still **owed**. Keep A4 open on the agenda.

## Sources checked

- Repo: `docs/research/capture/correlation.md` (pass 62 banner + claim table); absence of `correlation-reliability-runs/`; `crates/parallax-analysis` derive/fingerprint; unified-CLI validation tree; pass 161 GraphQL identifier grep in crates.

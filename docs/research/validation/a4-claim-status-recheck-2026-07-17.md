# A4 claim status recheck — product code vs real_pilot ledger (pass 121)

<!-- markdownlint-disable MD013 -->

**Research date:** 2026-07-17  
**Pass:** 121 (deep-research-parallax indefinite program)  
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
| Ledger *instance* | **Absent** | No `docs/research/correlation-reliability-runs/<run_id>/` (rechecked 2026-07-17) |
| Product correlation *surfaces* | **Shipped** | GraphQL: `tracesByInvocation`, `logsByTrace`, `logsByInvocation`, `linkedTraces`, `story`, `evidenceGaps`, `bundle` (per correlation.md implementation banner) |
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

## Uncertainty

- Did not re-query live GraphQL schema introspection this pass; surface list taken from research implementation banner + analysis crate layout.
- Unified-CLI corpus may include useful *synthetic* rates if re-scored under A4 row schema — still would not promote past `synthetic_only` without `real_pilot`.

## Parallax goal fit

Agent bundles depend on **deterministic** edges being common enough to be useful and rare enough false-strong not to hallucinate. Code paths make measurement *possible*; measurement is still **owed**. Keep A4 open on the agenda.

## Sources checked

- Repo: `docs/research/capture/correlation.md` (pass 62 banner + claim table); absence of `correlation-reliability-runs/`; `crates/parallax-analysis` derive/fingerprint; unified-CLI validation tree.

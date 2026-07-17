# Workload Mix — Decision Input Packet (DQ5 flip rule)

<!-- markdownlint-disable MD013 -->

Status: **Run 223 (2026-07-17)** — highest-value product gap from
[`open-questions-and-gaps.md`](open-questions-and-gaps.md) §1. This note does
**not** invent Parallax production traffic (none instrumented yet). It defines
**what to measure**, **how to score** the mix, and **which engine risks fire**
at each mix — so a future product/usage model can flip or re-affirm DQ5 without
re-running the whole internals study.

> **Stack authority unchanged:** GreptimeDB + Turso are mandatory for the
> product. A mix that would “prefer ClickHouse on speed” changes **risk
> statements, upstream priorities, and optional analytics sidecars** — not a
> silent engine swap.

Companions: [`verdict-which-to-choose.md`](verdict-which-to-choose.md) (DQ5 +
flip rule), [`per-signal-verdict.md`](per-signal-verdict.md) (Q1–Q6),
[`platform-fit-and-alternatives.md`](platform-fit-and-alternatives.md).

## Why this is the deciding input

Every load-bearing speed claim in this study is **shape-conditioned**:

| Query family | Who wins (engine study) | Mix weight that makes it matter |
| --- | --- | --- |
| **Anchored keyed fetch** (`trace_id` / `fingerprint` / issue id → rows) | **Tie** interactive on both at laptop+; both prune when keyed (Runs 158, 191, 211) | Dominant → **GT fine**; CH speed edge idle |
| **Evidence-bundle assembly** (multi-signal keyed + app join) | **Tie** ≪300 ms (Q6 Runs 16/56) | Dominant → **GT fine**; assembly is app-side |
| **Selective full-text** (rare token) | **Near-tie** (both prune; CH finer granule) | Secondary → small CH edge |
| **Broad full-text / log tail** | **CH** (scan-bound; vectorized) | Dominant → CH risk material |
| **Wide metric agg / ad-hoc GROUP BY** | **CH ~2–3× warm** (scale widens); GT PromQL tax ~1.5–2× SQL @100k | Dominant analytics → CH risk |
| **Dynamic JSON path analytics** | **CH default**; GT **JSON2** closes most (Runs 173/176) | High JSON analytics without JSON2 → GT self-own goal |
| **Cold selective S3** | **CH** granule locality if GT unpartitioned; **~10×** if GT partitions on `trace_id` (Runs 55/88/220) | Cold deep history + unpartitioned → CH edge |

**Hypothesis under test (operator, long-standing):** Parallax is
**anchored-retrieval-dominant**. If true, CH’s scan wins are **off the hot path**.
If false, re-score.

## Mix model (dimensions to fill)

Score over a **steady 7-day window** (or a written product projection for v1).

### A — Query class shares (must sum ≈ 100% of read QPS)

| Code | Class | Example product surface | How to count |
| --- | --- | --- | --- |
| **A1** | Anchored single-key fetch | Open issue → spans/logs for `trace_id`; fingerprint detail | GraphQL/resolver span with key equality |
| **A2** | Bundle / multi-key assembly | Evidence bundle, AI context pack | One product “bundle build” = 1 event (not N SQL) |
| **A3** | Selective search | “Find this UUID / request id in logs” | Search API with high-selectivity term |
| **A4** | Broad search / explore | “errors last 24h”, service log tail | Search/scan without strong key |
| **A5** | Dashboard / PromQL / metric rollup | Service red-panel, SLO burn | PromQL or metric SQL range agg |
| **A6** | Ad-hoc analytics SQL | Internal BI, arbitrary GROUP BY on attrs | Direct SQL or analyst console |
| **A7** | Admin / export | Backup, reindex, GDPR export | Low frequency; exclude from hot-path % or cap |

**Record:** `qps_share[Ai]`, `p95_latency_budget[Ai]`, `bytes_scanned_est[Ai]`.

### B — Write / retention shape

| Code | Dimension | Why it matters |
| --- | --- | --- |
| **B1** | Ingest rate (OTLP points/s by signal) | Freshness under concurrent ingest (Run 178 ≤1.5×) |
| **B2** | Cardinality (series / services / attrs) | Metric engine vs LowCardinality cliff |
| **B3** | Hot window vs cold retention | Days on hot path vs object-store depth |
| **B4** | Re-read factor | How often historical bundles re-fetch cold data (egress $) |

### C — Deployment constraint

| Code | Dimension | Effect on flip rule |
| --- | --- | --- |
| **C1** | Self-host only vs managed OK | Managed CH Cloud closes OSS CH S3/N× tax (Run 221) |
| **C2** | Air-gap / BYO keys | Favors self-host GT |
| **C3** | Ops FTE budget | Low FTE → managed pressure (Run 175/221) |

## Scoring rubric (map mix → posture)

Define:

```text
anchored = A1 + A2 + A3          # retrieval-shaped
analytics = A4 + A5 + A6         # scan/agg-shaped
```

Use **read QPS share** first; optionally weight by **p95×share** if latency-critical.

| Result | Condition | Product posture |
| --- | --- | --- |
| **Anchored-dominant (expected)** | `anchored ≥ 70%` and `analytics ≤ 25%` | **Keep GT**; CH wins are secondary. Invest in keys/`PARTITION ON trace_id`/JSON2 for the analytics tail. |
| **Mixed** | `anchored 40–70%` | **Keep GT**; add **proxy-side** guards (pre-aggregate, limit broad search windows, Flow rollups). Optional CH **sidecar** only for A6 BI — not product core. |
| **Analytics-dominant** | `analytics ≥ 50%` **and** cold GB–TB scans miss latency budget on GT | **Flip-rule secondary fires** → document CH as speed-optimal; still need product decision (sidecar vs stack change). Mandatory stack does **not** auto-swap. |
| **Cost-cloud flip** | Sized self-host cost ≈ managed CH **and** managed OK (C1) | **Primary flip rule** (verdict) — independent of mix; use Run 221 envelopes + server $/GB. |

### Latency budgets (defaults until product sets SLOs)

| Class | Default p95 budget | Engine study note |
| --- | --- | --- |
| A1 / A2 | **300 ms** | Q6 composite ≪ budget on both |
| A3 | **1 s** | Selective FT tie-ish |
| A4 | **3–10 s** interactive | CH preferred if this is UX-critical |
| A5 | **1–2 s** dashboard | PromQL tax real on GT |
| A6 | **10–60 s** OK for BI | CH preferred |

If product sets tighter A4/A6 budgets than GT can hit at retention scale, that is
**measured flip evidence**, not opinion.

## How to gather the numbers (in order)

1. **Product intent worksheet** (1 page) — fill A1–A7 target shares for v1 / v1.x
   *before* traffic exists (hypothesis, not measurement).
2. **Proxy/access log taxonomy** — tag every read path with `Ai` when the API
   lands; emit Prometheus counters `parallax_query_class_total{class=}`.
3. **Synthetic mix harness** — drive Q1–Q6 (+ broad search + PromQL) at the
   recorded shares against four-way pins; confirm p95 vs budgets
   (`bench/four-way`, server tier for GB).
4. **Re-score this note** — update the result row; link the measurement date.

Do **not** wait for perfect telemetry to write the **intent** row. An explicit
wrong hypothesis is better than an implicit one.

## Provisional product hypothesis (fill-in; not measured)

| Field | Value (operator to confirm) | Source |
| --- | --- | --- |
| A1+A2 target share | **_ %** (expected high) | product vision: evidence bundles |
| A4+A6 target share | **_ %** (expected low for end users) | explore/BI secondary |
| A5 PromQL share | **_ %** | dashboards vs SQL |
| Hot window | **_ days** | retention UX |
| Cold retention | **_ days** | cost axis |
| Deployment | self-host binary default | `AGENTS.md` / decisions |

Until filled, the study continues to **assume anchored-dominant** for risk
language, matching the 2026-05-29 operator lean.

## What this study will **not** re-do for mix

- Another 100k four-way tie on A1/A2 (saturated Runs 173–219).
- Declaring CH the product engine without an explicit operator stack change.

## What this study **will** do when mix arrives

- Re-weight [`per-signal-verdict.md`](per-signal-verdict.md) cells by share.
- Re-open only the mechanisms that sit under high-share classes (e.g. if A4
  spikes → broad FT + cold scan notes first).
- Feed server-tier bench priorities (B1/B10) with the right query mix, not uniform Q1–Q6.

## Research date

2026-07-17 — Run 223. Pins still GT `v1.1.3` / CH `26.6.1.1193` (no bump this
pass; mix is product-input, not version-sensitive).

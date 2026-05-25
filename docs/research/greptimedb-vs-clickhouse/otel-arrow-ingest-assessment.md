# OTel-Arrow (OTAP) Ingest — Assessment for Parallax

<!-- markdownlint-disable MD013 -->

Status: created 2026-05-25. The operator flagged **OTel-Arrow** ("looks like it will give us a huge
benefit") and asked to research it on tech-proven grounds, not marketing. This note answers: what it
is, the *measured* benefit, maturity, whether GreptimeDB's Arrow-native stack gives a real edge, and
whether it should weight the GreptimeDB-vs-ClickHouse decision. Web research (2026 sources) + cross-check
against our findings; **not** a local Docker run (OTAP is not GA on either side, so there is nothing
production-realistic to benchmark locally yet).

## Bottom line — the "huge benefit" is overstated *for Parallax today*; it is transport-only and experimental

OTel-Arrow's **proven, measured** wins are on **network egress + collector CPU** between two collector
pools. The part that would actually favour GreptimeDB at the *storage* layer — "Arrow on the wire →
Arrow in the engine, zero-copy" — is **aspirational / Phase-2, not shipping** on either engine. For a
**self-hosted** backend (Parallax) where the collector and the database may be co-located, the
network-egress savings that are the entire point **largely evaporate**. And at startup ingest volumes
(small batches, the "tiny single-node" phase) the benefit is **marginal and can cost net CPU**. So:
**do not weight OTel-Arrow as a near-term decision factor.** It is a *future, scaling-phase,
direction-alignment* point, not a today-benefit.

## What it is (mechanically)

- A **column-oriented re-encoding of OTLP** telemetry streamed over gRPC via Arrow IPC (a "star schema"
  of Arrow record batches). A **new protocol**, not an OTLP extension; non-lossy round-trip OTLP↔OTAP.
- Sits **collector→collector / collector→backend** — an edge/gateway bridge pair. Receiver/exporter are
  drop-in for OTLP and **fall back to OTLP** if a peer doesn't speak OTAP.
- **It does not come from SDKs.** Standard OTel SDKs do **not** emit OTAP (only an experimental SDK
  does), so in practice OTAP **requires a collector hop** to convert protobuf-OTLP → OTAP.

## The measured benefit (transport-only, conditional)

| Axis | Number (vs OTLP+zstd unless noted) | Condition |
| --- | --- | --- |
| Bandwidth — traces | ~**30%** less | similar pipeline config |
| Bandwidth — logs/metrics | ~**50–70%** less | high attribute repetition |
| The headline "**10×**" | vs **uncompressed**, not vs OTLP+zstd | marketing framing |
| CPU | **+5% to +43% MORE** (one production trial 77 vs 53.7 vCPU) | you spend CPU to save bandwidth |
| Independent (2026) | ~75% at high attribute repetition, but **~35% at high cardinality, ~30% at small batches** | benefit collapses on small/high-card batches |

- Gains need **moderate batches (100–1000 records)**; micro-batches make the Arrow schema overhead
  prohibitive. **The benefit is entirely transport** — it does **not** touch storage size or query speed.

## Maturity (2026) — experimental on every layer

- **OpenTelemetry side:** collector receiver/exporter shipped v0.104 (Jul 2024), declared "ready for
  general use" but stability badge is **beta**, not stable. **Not part of the OTLP spec.** Adoption is
  concentrated in a few large operators; production validation came from essentially one big deployment.
- **GreptimeDB side:** **"initial support," experimental** (blog 2025-05-23) — end-to-end Arrow Flight
  gRPC (DoPut) in Rust. **Zero-copy + direct DataFusion ingest are listed as Phase-2 future objectives,
  not accomplished.** No throughput numbers published (qualitative claims only). *(Corrects any reading
  of OTel-Arrow as a GA GreptimeDB advantage — see `public-performance-claims.md`.)*
- **ClickHouse / ClickStack side:** standard OTLP (gRPC/HTTP) via a collector → native-protocol write.
  **No native OTAP receiver.** (ClickHouse supports generic `FORMAT Arrow` INSERT + has Arrow Flight,
  but that is *not* OTAP and *not* on the ClickStack OTel path.)

## The Arrow-native synergy — real in theory, NOT realized in shipping code

The genuinely interesting claim — Arrow on the wire → Arrow into a DataFusion engine, skipping
protobuf-deserialize + row→column transpose — is **the one thing that would structurally favour
GreptimeDB over ClickHouse's MergeTree.** Verdict from the research: **not realized today.** Even
Arrow-native ingest paths still cross multiple serialization boundaries (protobuf→Arrow, Arrow-IPC
serialize for transport, **engine-side deserialize into native storage**); true zero-copy insertion is
"aspirational, not production reality." Data is re-encoded into the engine's native format on ingest on
**both** engines right now.

**But the structural edge is plausible and worth tracking:** GreptimeDB's whole stack is Arrow/DataFusion,
so it has a credible path to *eventually* shorten the ingest pipeline (OTAP batches → mito2/DataFusion
with minimal re-encode) further than ClickHouse can with MergeTree. If GreptimeDB's **Phase-2 zero-copy
DataFusion ingest** actually ships and is measured, OTel-Arrow becomes a real **GreptimeDB-fit
amplifier** — exactly the kind of Arrow-native synergy the DQ6 "bet on the better-aligned design" thesis
predicts. Today it is a roadmap item, not a measured advantage.

## What this means for Parallax (honest, decision-focused)

- **Not a near-term GreptimeDB-vs-ClickHouse decision factor.** Transport-only + experimental + needs a
  collector hop + marginal at startup scale. The fit/economics/ingest-ergonomics pillars already decided
  the verdict; OTel-Arrow changes none of them today.
- **Self-hosted caveat is decisive against the "huge benefit" framing now.** If Parallax's collector
  (or agents) and the storage node are in the same host/cluster — the normal tiny-single-node start —
  there is little network egress to save, so OTAP's headline benefit mostly disappears, while the CPU
  cost and the extra collector component remain.
- **Where it *would* matter: the scaling phase.** At high, sustained, attribute-repetitive ingest over a
  real network (agents/edge → central Parallax, the "big companies later" phase per
  [[scaling-trajectory]]), OTAP can cut ingest bandwidth ~30–70% — a real cost lever then, not now.
- **Direction-alignment, not speed:** OTel-Arrow leaning on Arrow plays to GreptimeDB's architecture; it
  is a (soft) point on the *trajectory/closability* side of DQ6, not a hot-path or storage win.

## Recommendation

1. **Do not let OTel-Arrow sway the storage choice today.** Keep the verdict on the proven pillars.
2. **Use standard OTLP (gRPC/HTTP) for ingest** — GA on GreptimeDB (native, no collector required) and
   the pragmatic path; it is already a GreptimeDB edge over ClickHouse (collector-only OTLP). OTAP would
   *add* a mandatory collector hop, which cuts against Parallax's no-middleware ingest simplicity.
3. **Track one specific signal:** GreptimeDB Phase-2 **zero-copy / direct-DataFusion OTAP ingest** with
   *published throughput numbers*. If that ships GA and measures a real ingest-CPU/throughput win, revisit
   — that is the only version of "huge benefit" that is GreptimeDB-specific and storage-deep. Until then,
   treat "Arrow end to end" as marketing.

## Cross-refs

`write-path-and-ingestion.md` (ingest path, native OTLP), `public-performance-claims.md` (OTel-Arrow =
experimental, not GA), `verdict-which-to-choose.md` (DQ6 direction/closability; ingest pillar),
`vendor-claims-audit.md` (Run 106 — where the OTel-Arrow status correction originated),
[[scaling-trajectory]] (the volume threshold where transport savings become material). Sources: OTel-Arrow
Phase-2 + production + 2023 announcements (opentelemetry.io), open-telemetry/otel-arrow repo, collector
stability badges, GreptimeDB 2025-05-23 blog, ClickStack OTel docs, independent oneuptime 2026 benchmarks.

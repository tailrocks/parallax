# Playground + Fan-Out Lab — Live Verification & Enrichment Backlog

Research date: 2026-06-23
Status: **live run executed** — the whole stack was brought up on one machine
(Docker) and the fan-out path verified end-to-end across all five backends.
Companion to [otlp-fanout-comparison-lab.md](otlp-fanout-comparison-lab.md)
(plumbing) and [telemetry-playground-sample-project.md](telemetry-playground-sample-project.md)
(payload). This note records what the live run proved, the bugs it surfaced, and
the concrete backlog for making the sample the *richest possible* cross-backend
comparison.

## 1. What the live run proved (2026-06-23)

Single host: **Parallax on the host** (lab config — OTLP offset to
`14317/14318`, `bind 0.0.0.0`), everything else in Docker.

- **Rotel fan-out to all five sinks, identical input.** One `telemetrygen` batch
  (traces+logs+metrics, `service=fanout5`) into Rotel `:4317` landed in every
  backend at once:

  | Backend | Verified copy | How asserted |
  |---|---|---|
  | Parallax (host) | traces+logs+metrics | `parallax traces --service fanout5`, `parallax logs` |
  | OpenObserve | 62 trace spans | `/api/default/_search` SQL count |
  | Maple (chDB) | 20 spans | `maple traces` in-container |
  | SigNoz | 62 spans | `clickhouse-client` on `signoz_traces.distributed_signoz_index_v3` |
  | Sentry v26.6.0 | OTLP ingest 200 + issue grouping | `sentry/verify.sh` (A1/A15/A16) |

- **Sentry stood up for real** — the full ~72-service `getsentry/self-hosted`
  stack, native OTLP ingest, reached over the host bridge
  (`host.docker.internal:9000`). 5 identical errors grouped into one issue
  (`times_seen=5`). Sentry is no longer "deferred."

## 2. Bugs the live run surfaced (all fixed)

Running it for real found things no amount of static review had:

1. **Parallax dropped every gRPC copy with `Unimplemented`** (parallax repo,
   `fix(server): accept gzip on OTLP/gRPC receivers`). Rotel — like the OTel SDKs
   — gzip-compresses gRPC by default; tonic only decompresses encodings it is
   told to accept, so Parallax's trace/logs/metrics services rejected every
   compressed request. **Every other backend accepted gzip, so the failure was
   silent and Parallax-only** — exactly the asymmetric host-hop failure the lab
   doc warned about, but with a different root cause (compression, not
   networking). Fix: `accept_compressed(Gzip)` + tonic `gzip` feature.
2. **`payment` (Java) had no Gradle wrapper** — built locally on a system gradle,
   failed `./gradlew` in Docker with exit 127.
3. **Java image flattened the repo layout** — `payment`'s protobuf
   `srcDir("../../proto")` resolved to a missing path in-image, so gRPC stubs
   never generated. Mirror the repo layout in the build stage.
4. **Rust image lacked OpenSSL** — `openssl-sys` (transitive via OTLP/tonic TLS)
   needs `pkg-config`/`libssl-dev` to build and `libssl3` at runtime.
5. **`onboard.sh` ran a backtick'd word** as a command in an unquoted heredoc.

Bringing up the **full polyglot playload** (5 Rust + 4 Java + web) against the
running lab surfaced a second wave — all of which compile-only checks missed:

6. **Rust runtime glibc mismatch** — built on `rust:1-slim` (Debian trixie),
   ran on `debian:bookworm-slim` → `GLIBC_2.38 not found`; also `openssl-sys`
   needed `pkg-config`/`libssl-dev`. 7. **payment** shipped no Gradle wrapper +
   the Java image flattened the repo so its protobuf `srcDir("../../proto")`
   missed; protoc gencode 4.35.1 also outran the runtime protobuf-java 4.33.4.
   8. **Sentry Spring Boot starter 8.44 is incompatible with Spring Boot 4.x**
   (relocated `RestClientCustomizer`) — crashed catalog/fulfillment at boot; let
   the sentry-opentelemetry **agent** own Sentry init instead. 9. **Spring Boot 4
   modularized auto-config** — plain `spring-kafka` no longer auto-configures
   `KafkaTemplate`; needs `spring-boot-starter-kafka`. 10. **Redpanda advertised
   `127.0.0.1:9092`** → unreachable cross-container, breaking the Kafka
   round-trip; advertise the service name. 11. **The Java OTel agent's okhttp
   gRPC sender fails against Rotel** ("Failed to read response body") — spans
   never exported over gRPC; switching the Java tier to **OTLP/HTTP** (Rotel
   `:4318`) fixed it. This mirrors the Parallax gzip bug from the other side: a
   *standard* exporter/transport combination silently dropping data until tested
   live.

After the fixes, all eight app services emit; Parallax holds spans for every one
(`checkout` 365, `payment` 100, `inventory`/`recommendation` 73, `catalog` 21,
`pricing` 18, `fulfillment` 13, `notifications` 3) plus JVM metrics
(`jvm_*` tables), and the same data fans to OpenObserve/SigNoz/Maple/Sentry.

~~Open refinement~~ **Fixed (2026-06-23, parallax repo):** catalog's GraphQL
spans arrive as `INTERNAL` with no `SERVER` root, so `parallax traces --service
catalog` (which listed by root span) showed nothing though the 21 spans were
stored. Two changes to `traces_search` (memory + GreptimeDB adapters): (1) the
`service` filter now matches any trace the service **participates in** (a span
of that service anywhere), not only the root; (2) the trace's representative is
its root span, or — when no root was stored (all-`INTERNAL` traces) — its
**earliest span**, so such traces list instead of vanishing. Verified live
against the running GreptimeDB: catalog had 0 root spans yet participated in 21
traces, and the new window query returns all 21 (representative = catalog's
earliest span) while rooted traces (e.g. `checkout`) keep their real `SERVER`
root. Unit-tested in `memory.rs`; never was data loss.

Lesson for the thesis: **the fan-out lab is itself a conformance test.** A
backend that mishandles a *standard* exporter default (gzip) silently loses data
while looking healthy. This is precisely the class of defect Parallax must never
have — and the lab catches it.

## 3. Enrichment backlog — toward the richest possible sample

The playload already exercises the full §9 checklist (all span kinds, links,
logs+severity, metrics+exemplars on JVM, exceptions, baggage, feature flags,
`parallax.run.id`, canary corpus, deploy/regression). Enrichments that would make
the *cross-backend comparison* sharper, ranked by signal-per-effort:

1. **Cross-language error-grouping stimulus.** Emit the *same logical failure*
   (`PaymentError: payment failure (chaos)`) from the Rust `checkout` path **and**
   the Java `payment` path, same fingerprint, and compare how each backend groups
   it: one issue or two? language-aware? This is the single highest-signal
   differentiator test — Sentry groups aggressively, OTLP-only backends (SigNoz/
   OpenObserve) mostly don't group at all, Parallax's issue layer sits in between.
   ([Sentry fingerprinting](https://docs.sentry.io/concepts/data-management/event-grouping/))
2. **Protocol/compression matrix.** Drive the same trace over OTLP/gRPC+gzip,
   gRPC+uncompressed, and OTLP/HTTP-protobuf to each backend, and record which
   combinations each accepts. The gzip bug shows this matters and is invisible
   until tested. Cheap to add (telemetrygen flags) and directly exercises the
   conformance surface.
3. **Custom-resource-attribute rendering.** A scripted assertion of how each UI
   surfaces `parallax.run.id` / `parallax.lab=1` (filter? facet? hidden?). This is
   where Parallax should win and competitors show nothing — make it a named,
   repeatable step, not an eyeball.
4. **OTLP Profiles signal (alpha, 4th signal).** Today profiling is Sentry-only
   (Rust pprof / JVM async-profiler). Emit the OTel profiling signal where a
   runtime supports it so backends can be compared on native OTLP profiles vs
   Sentry-envelope profiles. Mark alpha; few backends ingest it yet.
   ([OTel Profiles](https://opentelemetry.io/blog/2026/profiles-alpha/))
5. **eBPF zero-code (OBI) on one service.** Instrument e.g. `recommendation`
   *both* via SDK and via OpenTelemetry eBPF Instrumentation, and compare
   breadth-no-code vs depth-custom-spans across backends.
   ([OBI](https://opentelemetry.io/docs/zero-code/obi/))
6. **Streaming-span fidelity probe.** A7 already emits gRPC server-streaming +
   GraphQL subscription + SSE/WS long-lived spans; add an explicit per-backend
   check of how each renders a span that is *open for minutes* (a known weak
   spot) — does it show live, truncate, or drop?
7. **Canary-redaction diff table.** A18 plants the canary corpus; turn the
   "compare raw-vs-scrubbed" step into a fixed per-field table (token, JWT, DB
   URL, email, …) × backend (stored raw / masked / dropped). Directly feeds
   Parallax's redaction-at-ingest gate.

Out of scope (operator, 2026-06-23): an automated scored harness. These stay
**manual** comparison steps — but written down as concrete, repeatable probes
rather than "open the UI and look."

## 4. Comparison rubric (manual, repeatable)

For each scenario, per backend, record the cell — this is the comparison surface
without building a scoring engine:

| Dimension | What to look for |
|---|---|
| **Ingest** | accepted? (protocol/compression); silent drop? |
| **Fidelity** | attributes preserved / renamed / dropped; semconv version effects |
| **Trace** | waterfall correctness; cross-language stitch; streaming spans |
| **Logs↔trace** | correlation present; severity preserved |
| **Metrics** | rollups; exemplar → trace jump (JVM tier) |
| **Errors** | grouping (one issue vs N); stack-trace fidelity; lifecycle |
| **Run-scoping** | is `parallax.run.id` surfaced/filterable? (Parallax-distinguishing) |
| **Redaction** | canary corpus stored raw vs scrubbed (Parallax-distinguishing) |

Parallax's wins should concentrate in the last two rows; the first six are table
stakes where it must at least match OTLP-native peers.

## 5. Operating the live lab (reproducer)

```bash
# Parallax (host): ~/.parallax/config.toml → bind 0.0.0.0, otlp 14317/14318
parallax serve                                   # auto-loads ~/.parallax/config.toml

cd bench/otlp-fanout
./setup-vendor.sh                                # SigNoz clone
docker compose -f compose.yml -f compose.signoz.yml -f compose.maple.yml up -d
docker exec signoz wget -qO- --post-data='{...}' http://localhost:8080/api/v1/register
./sentry/setup.sh && ./sentry/onboard.sh         # Sentry stack + DSN (own vendored stack)
# paste onboard.sh's ROTEL_EXPORTER_SENTRY_* into rotel.env, enable all 5 exporters
docker compose ... up -d --force-recreate rotel  # full fan-out

# Payload: the polyglot playground (separate repo)
cd parallax-telemetry-playground
SENTRY_DSN=<from onboard.sh> docker compose -f deploy/docker-compose.yml up -d --build
```

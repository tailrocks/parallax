# Fan-out eight-sink comparison (2026-08-16)

Research date: **2026-08-16**
Lab: host Homebrew `parallax-preview` (`668b4736` at start of run) + Rotel
`v0.2.5` on host `4317/4318` + playground compose + competitor overlays under
`bench/otlp-fanout/`.
Emit: playground `a1` (checkout) + `b2` (inventory 503) over OTLP; `c8`
Rust/Java/JS Sentry envelopes to Parallax, rustrak, and Sentry.

Rotel is sequential. A down exporter was never left on `ROTEL_EXPORTERS`.
Reachability of the listed set is asserted by
`bench/otlp-fanout/exporters-reachable.sh`.

## Status table

| System | Protocol judged | Running? | Proof | UI |
|---|---|---|---|---|
| **Parallax** (host) | OTLP 3-signal + Sentry envelopes | running | GraphQL twice: 15 recent traces, 8 logs, **68 issues**, 15 services; exemplars resolve for `http.server.request.duration` **and** listed Prom `http_server_request_duration_seconds`; b2 `out_of_stock.lastTraceId` → `logsByTrace` has `reservation failed (chaos)` | proxy `http://127.0.0.1:4000` (injects Bearer; API `:4002` token-gated) |
| **Maple** | OTLP/HTTP | running (after volume reset) | `POST /local/query` → `traces` count **90**; checkout 36 / payment 20 / inventory 14; `error_events` **2** | `http://127.0.0.1:8081` |
| **OpenObserve** | OTLP/gRPC | running | `/api/default/_search` traces: checkout **5478**, recommendation 2334, inventory 1645 (lab-long window) | `http://127.0.0.1:5080` (`root@example.com`) |
| **SigNoz** | OTLP/gRPC | running on **v0.129.0** | ClickHouse `signoz_traces.distributed_signoz_index_v3`: checkout **63**, payment 53, catalog 28. **v0.137.0 Foundry-only** — last bootable community compose used. OTLP `:4317` opened only after `/api/v1/register` | `http://127.0.0.1:3301` |
| **Grafana stack** (otel-lgtm **0.30.2**) | OTLP → Tempo + Loki + Prometheus | running | Tempo `/api/search` **20** traces; Loki query_range **20** streams; Prom `up` **success** | `http://127.0.0.1:3300` (`admin`/`admin`) |
| **HyperDX** (ClickStack **2.35.0**) | OTLP/gRPC + ingest API key | running | ClickHouse `otel_traces`: checkout **63**, payment 24, inventory 23. `:4317` stayed closed until first team existed (`collectorAuthenticationEnforced`) | host `http://127.0.0.1:18080` (container 8080; login redirects to `localhost:8080` and hits playground catalog) |
| **rustrak** (**v0.14.4**, Sentry protocol) | envelopes only — **no OTLP metrics** | running | project `stored_event_count=3`; issues: c8-js, c8-java, c8-rust | UI `http://127.0.0.1:18082`, ingest `http://127.0.0.1:18081` |
| **Sentry** (**26.7.2**) | OTLP traces+logs + envelopes — **no OTLP metrics** | running | `verify.sh` A1 OTLP **200**; A15/A16 `PaymentError` **times_seen=5**; Groups: chaos PaymentError + c8-java + c8-rust | `http://127.0.0.1:9000` |
| **highlight** | last hobby self-host | **BLOCKED** | Live `docker compose -f compose.hobby.yml up backend` on `docker-v0.5.6` failed: bind `../backend/env.enc` missing (unmaintained tree). Hobby frontend publishes host **8080** (playground catalog) and start-infra ClickHouse **9000** (Sentry nginx). Hosted SaaS ended **2026-02-28**. | n/a |

Scratch evidence:
`/var/folders/8p/h376l_nn3375kyj72czdq2x80000gn/T/grok-goal-1ae92e57b734/implementer/`
(`sinks/*.json`, `parallax-api-1.json`, `parallax-api-2.json`, `c11-walk.txt`,
`coverage-restamp.md`).

## UI / search / navigate / linking (running systems)

### Parallax
agent-browser on the non-401 URL (`:4000` proxy): `/` **Overview**, `/issues`
**Issues** (67 of 67, `http.server.error` + `out_of_stock` with trace chip
`38a60441`), `/traces` **Traces** (2.2k; facets checkout 646 / payment 615),
`/logs` **Logs** (33k), `/metrics` **Metrics** (392). Not Offline, not 401.
`/health` 200 twice.

Search is filter chips + substring, not LogQL/TraceQL. Trace list newest-first
is **leaf-biased** (many 1-span `payment` rows) even though checkout roots exist.
Issue grouping **splits** the same `PaymentError` across rust/java/js
fingerprints; Sentry groups them. `lastTraceId` links b2 chaos to stored logs;
some Kafka disconnect issues have `lastTraceId=null`. Exemplar aliases work
(listed Prom name → stored OTel name).

### Maple
One port for OTLP + SQL + dashboard. `POST /local/query` is immediate chDB SQL
(`ServiceName`, not `service_name`). Fragile under this lab: a host probe storm
on `:8081` crashed the process; recreate then hit
“local store incompatible with this build's chDB” until the volume was wiped.
That is a comparison finding (single-binary store durability), not a Parallax
bug.

### OpenObserve
SQL `_search` is powerful once you know stream/`from`/`size`. Auth headers are
mandatory (Rotel `Authorization=Basic…`). Ingest volume was the largest of the
set because the playground had been emitting for ~40 minutes. Correlation is
SQL joins, not a one-box Lucene explorer.

### SigNoz
Full APM UI after first-org register. OpAMP is a silent empty-store footgun:
compose healthy ≠ OTLP listening. Once registered, ClickHouse counts match
HyperDX’s checkout=63 window (same emit). Heavy (ClickHouse + ZooKeeper).
Latest pin cannot start without inventing Foundry.

### Grafana LGTM
Best **query language** split: Tempo search, Loki LogQL, PromQL. Grafana is
pre-wired. Operator has to know which backend owns which signal. Tempo’s
`/api/search` default order was also **payment-leaf heavy**, same shape as
Parallax’s recent-traces list. No issue product.

### HyperDX / ClickStack
Best **one-box correlation** reputation (Lucene + live tail). This lab’s host
remap broke post-login navigation (`Location: http://localhost:8080/`). Ingest
API key is required; Rotel header `authorization=<team apiKey>`. Session replay
exists in the product; we did not emit replay. OTLP three-signal landed in native
`otel_*` tables.

### rustrak
Fastest Sentry-compat stand-up (SQLite, two containers). Issues list is clean
per-SDK titles; it does **not** collapse rust/java/js `PaymentError` the way
Sentry did. No traces/logs/metrics lake. Judge only envelopes.

### Sentry
Still the **grouping authority** in this emit: five identical chaos errors → one
issue `times_seen=5`; c8 rust+java present. Native OTLP traces+logs 200. Cost is
~72 containers and a 20–40 min `install.sh`. No OTLP metrics — Rotel must omit
`sentry` from `ROTEL_EXPORTERS_METRICS` or the sequential fan-out waits on
rejected metric exports.

## Parallax vs the set — improvements (record only)

Do **not** ship these in the same change as this comparison.

1. **Operator-facing UI auth.** Non-loopback `bind` + `api_token` makes GraphQL
   401; V1 UI never sends `Authorization`. Serve banner should name the working
   URL (loopback no-token **or** a token-injecting proxy). A 401 Overview is
   “Offline”, not “up”.
2. **Trace list roots.** Prefer client/root/checkout spans in the default
   newest-first list. Grafana Tempo showed the same leaf bias — do not copy it.
3. **Cross-language issue grouping.** Sentry won `PaymentError` collapse; Parallax
   stored three c8 issues. Tighten fingerprinting toward Sentry’s grouping for
   the same exception type + message across SDKs.
4. **One-box search.** HyperDX/OpenObserve/SigNoz make “find this checkout” a
   single query. Parallax’s chips are fine for power users; a unified
   log/trace/issue search box is the UX gap.
5. **Query language depth.** Grafana LogQL + TraceQL + PromQL still beat
   Parallax substring filters for “p95 of checkout where inventory 503”. The
   shipped `sql` GraphQL field is the closest peer — surface it next to the
   lists, not only on `/sql`.
6. **Link completeness.** Keep `lastTraceId` on every issue that had a span/log
   with a trace id (Kafka disconnect rows were null). Exemplar aliases already
   work — keep that contract when adding metric names.
7. **Availability of compare-mode sinks.** Sequential Rotel + first-org OpAMP
   (SigNoz/HyperDX) means “stack up” ≠ “ingest open”. A compare-mode preflight
   that TCP-probes listed exporters (this lab’s
   `exporters-reachable.sh`) should be the documented gate before `a1`.
8. **Session replay** remains a HyperDX/historical-highlight win. Out of V1
   scope; do not pretend Parallax covers it.

## Lab wiring added this run

| File | Role |
|---|---|
| `bench/otlp-fanout/compose.grafana.yml` | Grafana otel-lgtm 0.30.2, UI 3300, no host 4317 |
| `bench/otlp-fanout/compose.hyperdx.yml` | ClickStack 2.35.0, UI 18080, no host 4317 |
| `bench/otlp-fanout/compose.rustrak.yml` | rustrak-server/ui v0.14.4, 18081/18082 |
| `bench/otlp-fanout/setup-highlight.sh` | last hobby self-host attempt |
| `bench/otlp-fanout/exporters-reachable.sh` | parse + TCP-probe listed Rotel exporters |
| `bench/otlp-fanout/setup-vendor.sh` | default **v0.129.0** (last bootable compose) |
| `compose.signoz.yml` | collector on `lab` **and** `signoz-net` |

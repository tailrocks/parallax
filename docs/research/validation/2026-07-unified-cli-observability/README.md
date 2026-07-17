# Unified CLI observability — live acceptance evidence (plan 159)

Date: 2026-07-17. Host: operator's Docker-capable macOS machine.
`main` SHAs at closure: parallax `0e0e794`, playground `7192c3a`.
Storage: managed GreptimeDB 1.1.2 + Turso (`/tmp/parallax-qa/data`).

## What ran

- `parallax serve` (ready banner: `serve-banner.txt`) + the playground
  compose stack (`compose-ps.txt`; 12 always-on containers, plus `catalog`
  and `fulfillment` brought up for the Java/Kafka assertions after fixing
  their Spring Boot 4 startup crash — see Deviations).
- Corpus: the full 24-scenario corner-case sweep (plan 161), each CLI mode
  (`drive`, `cron`, `console --seconds 30`, `daemon` incl. a held run via
  `PLAYGROUND_DAEMON_HOLD_SECONDS`), the journey scenarios
  (`j-happy`/`j-error`/`j-outside`/`j-reattach`/`j-parallel`), real browser
  sessions on `:5173` (checkout, orders, RUM error, nopropagate variant),
  and one wrapper-registered observable test session
  (`parallax invocation start -- scripts/observable-test-session.sh rust`,
  83/83 tests, invocation `cc880c5c…`). Exit codes: `corpus-run.log`.

## Machine assertions

`assert.sh` (this directory) exits 0 — 27 PASS across all eight plan-159
assertion groups; raw JSON in `assert-outputs/`, transcript in
`assert-run.txt`. Highlights: four CLI modes with observed `appMode` and
derived terminal outcome; console sessions/screens/actions with the
checkout trace crossing into the `checkout` service; daemon cycles and
conversations; `order_dispatch` + `fulfillment_shipment` jobs with the job
id crossing the Kafka hop (two span kinds share it); service-map kinds
cli/browser/service; all four journey placements incl. parallel-invocation
isolation; and the negative legacy group (`run(runId:)` rejected by the
schema, no corpus signal carries `parallax.run.id`, a hand-posted
legacy-only span mints no invocation).

## Coverage matrix (emitter kind × surface)

| Emitter | Invocations list/hub | Journey | Traces | Logs | Metrics | Errors | Ecosystem |
|---|---|---|---|---|---|---|---|
| CLI one_shot (drive/cron) | assert 1, `ui/a-…` | n/a (no session) | assert 6 | assert 6 | runtime strip on hub | assert 8 outputs | `ui/i-…` (cli node) |
| CLI interactive (console) | assert 1, `ui/b-…` | `ui/f-…`, `ui/g-…`, assert 2/7 | assert 2 (cross-service), `ui/j-…` | hub logs tab | hub metrics strip | `ui/e-…` | cli→checkout edge |
| CLI daemon | assert 1, running: `ui/a-…` | n/a | `ui/c1/c2-…` (live growth) | `ui/d-…` (live tail) | cycles p50/p95 `ui/h-…` | cycle failures | cli node |
| Capsule layer | wrapped run `cc880c5c…` finished/exit 0 | n/a | test spans in store | wrapper logs | n/a | junit-derived issues | n/a |
| HTTP microservice | n/a | n/a | checkout/inventory traces | service logs | service RED page | service issues | service nodes |
| gRPC service | n/a | n/a | p-grpc-err/p-grpc-stream (plan-160 ledger) | pricing logs | RED | deadline issues | checkout→pricing edge |
| GraphQL gateway | n/a | n/a | p-graphql-err panel (plan-160 ledger) | storefront logs | RED | partial-field issues | storefront node |
| Kafka producer/consumer | n/a | n/a | p-kafka-lag + assert 4 | orders logs | queue metrics | dead-letter outcome=failure | orders node |
| Browser frontend | n/a | RUM session spans | web→checkout stitched trace | web logs | web vitals spans | RUM error issue | browser node `ui/i-…` |

Trace-shape cells (deep/wide/multiroot/orphan/skew/zero/links/longnames/
events) are covered cell-by-cell in `ui-defect-ledger.md` (plan 160);
`ui/k-…` and `ui/l-…` re-capture t-wide and t-orphan on this run.

## Screenshot index (`ui/`)

a invocations list with the running daemon and mode badges ·
b console-run hub overview · c1/c2 hub traces tab Live ON 14 s apart
(29→31 rows) · d hub logs live tail · e errors tab after j-error ·
f Sessions & UI with the screen-visit lane · g journey with the error
attributed to the checkout screen/widget · h Jobs & Cycles ·
i /ecosystem with cli/browser/service kinds · j trace detail with the
invocation back-link · k t-wide waterfall mid-scroll · l t-orphan detached
span. Browser console clean throughout (`browser-console.txt`).
Plan-160 audit captures live under `ui/audit/`.

## Deviations found and fixed (all on `main`, both repos)

1. Test-telemetry bridge deadlocked any `PLAYGROUND_TEST_TELEMETRY=1` test
   (tonic simple exporter inside a current-thread tokio runtime), then
   refused the wrapper's `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` env — fixed to
   OTLP/HTTP-binary on the batch thread (playground `9856186`, `37dd3b3`).
2. `catalog`/`fulfillment`/`payment` paired Spring Boot 4.1 with the
   Boot-3 Sentry starter and crashed at startup — moved to
   `sentry-spring-boot-4-starter` (playground `5ab4e90`).
3. Drive/cron log lines carried no `cli.invocation.id` log attribute, so
   invocation-scoped log queries were empty for one-shot runs (playground
   `5ab4e90`).
4. A daemon's capsule child flipped the live daemon to `finished`; capsule/
   daemon-mode spans no longer derive completion (parallax `0e0e794`).
5. Issue titles doubled/tripled the error type (`issue_title` collapse +
   upsert title refresh + errors-tab double prefix; parallax `0e0e794`).
6. Daemon observability: `PLAYGROUND_DAEMON_HOLD_SECONDS` + `app.mode` on
   cycle spans (playground `7192c3a`).

## Verdict

All plan-159 done criteria met on this host: assert.sh green with stored
outputs, thirteen named screenshots with a clean console and no unvisited
coverage cell, bring-up artifacts captured, deviations fixed forward and
re-asserted. This is Wave 1's completion evidence (direct-to-main model).

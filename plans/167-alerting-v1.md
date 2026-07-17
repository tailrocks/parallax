# Plan 167: Alerting v1 — rules, evaluator, incidents, destinations

> **Executor instructions**: Follow this plan step by step. This plan adds a
> new product capability (backend + UI); read `ui/AGENTS.md` before UI work
> and apply its browser-verification checklist after every UI step against
> playground breach scenarios. STOP conditions binding. Update this plan's
> status row in `plans/README.md` when done.
>
> **Drift check (run first)**:
> `git diff --stat <wave2-base>..HEAD -- crates/parallax-metadata crates/parallax-api crates/parallax-server ui/src/components/nav.ts`
> `<wave2-base>` = the `main` commit closing Wave 1 (plan 159's evidence commit `0e0e794`).

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: MED-HIGH (new long-running evaluator inside the server; outbound
  network delivery)
- **Depends on**: plans 156 (contract), 162 (severity tokens), 164
  (attributeFilters reuse for rule scoping — soft; inline filters acceptable
  if 164 not yet landed)
- **Category**: direction / product / alerting
- **Planned at**: `2288011`, 2026-07-17

### Landed by Grok (preliminary) — peer verify/extend + full stack

**Do not retire yet.** Pure evaluation + delivery helpers plus Turso
schema/CRUD only. No evaluator I/O loop, GraphQL, full UI pages, or live
evidence. Index status stays TODO.

**Already landed:**
- `crates/parallax-server/src/alerting/mod.rs` + `state_machine.rs` —
  pure `(rule, prev_state, measurement, now) → (next_state, transition)`:
  comparators gt/gte/lt/lte/between/not_between, consecutive-breach and
  consecutive-healthy hysteresis, min sample count gate, no_data skip vs
  zero, renotify interval, flapping never double-opens while open.
  Unit tests embedded in the module (breach open, healthy resolve, flap,
  min samples, no_data, between, renotify). Wired as private `mod alerting`
  in `parallax-server` (not yet public API surface).
- `crates/parallax-server/src/alerting/delivery.rs` — pure delivery surface:
  `unique_delivery_key`, webhook JSON body (rule/incident/value/threshold/
  window/links), Slack `{text}` payload, backoff 1m/5m/30m, max 5 attempts
  dead-letter, claim lease (30s) availability. Unit tests in-module. **No
  reqwest/I/O** — peer owns the worker.
- `ui/src/lib/alert-rule-form.ts` — draft validation (comparator/threshold_upper
  pairing, metricName for metric signal, error_rate fraction range) + five
  plan templates (high-error-rate / slow-p95 / slow-p99 / throughput-drop /
  log-error-burst) via `draftFromTemplate`. Vitest in
  `__tests__/alert-rule-form.test.ts`.
- `crates/parallax-metadata/src/turso/alerts.rs` (`02d6dd2`, helper agent
  2026-07-17) — Step 1 remainder: DDL for the six tables (alert_rules,
  alert_rule_states, alert_incidents, alert_destinations,
  alert_delivery_events, alert_checks) appended to the idempotent SCHEMA
  block, plus inherent CRUD on `TursoMetadataStore`: rule upsert preserving
  `created_at`, `alert_rule_claim` CAS via `last_scheduled_at`, state
  upserts, `alert_incident_open` with an under-lock (rule, group, open)
  dedupe guard, resolve/touch, destination CRUD, outbox enqueue idempotent
  on the UNIQUE `delivery_key` with lease claim (`alert_deliveries_claim`),
  `mark_delivered`/`mark_failed` (caller-supplied backoff, `dead` flag),
  and audit inserts pruned to `ALERT_CHECKS_KEEP_PER_RULE` (500) per rule.
  Records re-exported from the crate root. Nine in-module tests; crate
  fmt/clippy/nextest green. NOT yet wired into the `MetadataStore` trait
  in `parallax-storage` (that file was under concurrent peer edit for
  plan 164) — peer decides trait vs concrete-store access for the
  evaluator, verifies semantics (esp. claim cutoff arithmetic and prune
  SQL), and extends.
- `crates/parallax-server/src/alerting/evaluator.rs` (`043ca4e`, helper
  agent 2026-07-17) — Step 2 orchestration: `tick_once(store, source,
  now_nanos, claim_interval_secs) → TickReport` doing CAS claim →
  measurement via a new `MeasurementSource` trait (GreptimeDB impl still
  peer-owned) → pure `evaluate_rule` → state upsert + bounded audit row →
  incident open/resolve/renotify + per-destination outbox enqueue with
  `unique_delivery_key` (renotify keys suffixed with the notify second so
  repeats survive the UNIQUE dedupe). Groups with prior state but no fresh
  measurement still tick (no-data path); config parse errors land as
  `status='error'` audit rows without aborting the tick. Five tests
  (lifecycle incl. renotify + resolve, idempotent re-tick, no_data skip,
  bad comparator, disabled rule); crate clippy clean; `async-trait` added
  to `parallax-server`. Peer wires the tokio interval loop + shutdown,
  measurement queries per signal type, delivery worker I/O, and re-verifies
  incident-id determinism and renotify-key policy.
- `crates/parallax-server/src/alerting/delivery_worker.rs` (`493c9e0`,
  helper agent 2026-07-17) — Step 3 I/O pass: `deliver_due_once(store,
  client, claimer, base_url, now_nanos, limit) → DeliveryReport` claims
  due outbox rows (30s lease), builds webhook/slack payloads from stored
  rule/incident/destination, POSTs via reqwest, marks delivered /
  backed-off retry (helpers' 1m/5m/30m, dead at 5 attempts) / permanent
  dead-letter for unbuildable payloads. Four tests against a local HTTP
  listener (success + payload assertions, 500 backoff not-yet-due,
  config-without-url dead-letter, slack text payload). Peer wires the
  tokio interval loops + graceful shutdown in serve.rs, ready-banner line,
  config knobs, and the live end-to-end webhook evidence.
- `crates/parallax-server/src/alerting/measurement.rs` (`be62e9f`, helper
  agent 2026-07-17) — pure measurement math: `SignalType::parse`,
  `service_in_scope` (include/exclude JSON lists, exclude wins),
  `groups_by_service`, `span_measurements` (error_rate fraction with
  zero-span no-data, throughput normalized to spans/minute, p95/p99 with
  worst-service max when ungrouped), `scalar_measurement` for
  log_count/metric. Nine tests. Peer implements the adapter-trait-backed
  `MeasurementSource` I/O shim on top (service_summaries / span_red_series
  / log_count_series / metric_series) plus the live-engine tick test, and
  re-verifies the ungrouped worst-service percentile policy against the
  plan's ROOT-span RED convention decision.
- `ui/src/routes/alerts.index.tsx` + nav entry (`1e4be3f`, helper agent
  2026-07-17) — Step 4 skeleton: rules list cards, create dialog with the
  five template presets + `validateAlertRuleDraft`, `draftToArgs`
  mutation-argument serializer (two vitest cases), and a graceful
  "backend not wired yet" empty state while the `alertRules` field is
  absent (loader catches instead of crashing). Assumed GraphQL shape:
  query `alertRules { id name enabled signalType comparator threshold
  severity windowMinutes }`, mutation `alertRuleSave(name, enabled,
  signalType, comparator, threshold, windowMinutes, minimumSampleCount,
  consecutiveBreachesRequired, consecutiveHealthyRequired, severity,
  renotifyIntervalMinutes, [thresholdUpper, metricName,
  metricAggregation, services]) { id }` — peer aligns the resolver naming
  or adjusts this page, then builds rule detail (threshold chart,
  plan-162 tokens), incidents, destinations, and browser evidence.
  UI typecheck/lint/format green; the four full-suite vitest failures at
  this head reproduce identically without this slice (parallel-run
  timeouts in dashboards/sql/overview/range-picker lanes) and pass in
  isolation — peer's flaky-lane concern, not introduced here.

**Peer owns (verify/deepen/complete):**
- [ ] Re-verify state machine + delivery helpers vs plan exhaustiveness;
  deepen payload schema docs if needed.
- [ ] Step 1 remainder landed at `02d6dd2` (`turso/alerts.rs`, see above):
  verify/deepen it, decide trait wiring, do not clobber.
- [ ] Steps 2–5: evaluator CAS loop, measurement queries, delivery worker
  (native-tls reqwest calling these pure helpers), GraphQL + UI pages,
  playground breach scenarios, ready-banner line, browser + webhook
  evidence under `docs/research/validation/2026-07-wave2/167/`.
- [ ] Full Done criteria; then retire.

## Why this matters

Parallax can show an incident but cannot *tell anyone about it* — there is
no alerting at all. That caps the product at "dashboard you remember to
open". The reference product's alerting model is small and proven, and its
control plane is plain SQL rows + a poll loop — a natural fit for
Parallax's Turso + tokio architecture with zero new infrastructure. V1
scope: threshold rules over the signals Parallax already computes (error
rate, p95/p99 latency, throughput, log severity count, metric aggregate),
a state machine with consecutive-breach hysteresis, incidents with
lifecycle, and webhook/Slack-webhook/email destinations.

## Reference (self-contained)

Model adapted from Maple (`packages/db/src/schema/alerts.ts`,
`apps/api/src/services/AlertsService.ts`, `apps/alerting` worker; clone
`https://github.com/MapleTechLabs/maple` for detail — the contract below is
complete and self-sufficient):

- **alert_rules**: name, enabled, `signal_type`
  (`error_rate|p95_latency|p99_latency|throughput|log_count|metric`),
  scoping (service list / exclude list / optional attribute filters /
  optional `group_by` dimension: service), `comparator`
  (gt|gte|lt|lte|between|not_between), `threshold` (+ `threshold_upper`),
  `window_minutes`, `minimum_sample_count`,
  `consecutive_breaches_required` (default 2),
  `consecutive_healthy_required` (default 2),
  `no_data_behavior` (`skip|zero`), `severity` (`warning|critical`),
  `renotify_interval_minutes` (default 30), destination ids,
  `metric_name/metric_aggregation` for `signal_type=metric`,
  `last_scheduled_at` (evaluator CAS claim).
- **alert_rule_states**: per (rule, group_key) — consecutive breach/healthy
  counters, last status/value/sample-count/evaluated-at/error.
- **alert_incidents**: rule id, group_key, status `open|resolved`,
  `first/last_triggered_at`, `resolved_at`, last observed value,
  `dedupe_key` (rule+group while open), `last_notified_at`.
- **alert_destinations**: type `webhook|slack_webhook|email`, config JSON
  (URL / address); secrets stored in Turso — V1 runs single-operator
  local-first, so plaintext-at-rest is acceptable ONLY if the config file
  documents it; prefer reusing any existing Parallax secret handling if one
  exists (check `crates/parallax-metadata` first).
- **alert_delivery_events**: outbox — incident id, destination id, event
  type (`triggered|resolved|renotify`), attempt count, claim lease
  (`claimed_by`, `claim_expires_at`), delivered_at, unique delivery key.
- **Evaluator loop** (in `parallax-server`, tokio interval 60s): list
  enabled rules ordered by updated_at; CAS-claim each via
  `last_scheduled_at` (skip if claimed <30s ago — safe under multiple
  server instances); evaluate the rule's measurement query over
  `[now - window, now]` per group; apply `minimum_sample_count` +
  `no_data_behavior` + comparator; drive the consecutive counters;
  open/resolve incidents; enqueue deliveries; write an evaluation audit row
  (Turso table `alert_checks`: rule id, ts, group, value, sample count,
  status, error) — bounded retention (keep last N per rule).
- **Delivery worker** (second tokio interval, 10s): claim due outbox rows
  (lease), POST webhook JSON / Slack-webhook payload / send email
  (reuse whatever mail transport exists; if none, `email` type is DEFERRED —
  ship webhook + slack_webhook only and record it), exponential backoff
  (1m/5m/30m, max 5 attempts), mark delivered.
- **Measurement queries** (GreptimeDB, all already-proven shapes):
  error_rate = errored spans / total spans (root or all — decide: ROOT
  spans, matching the service RED convention already used by `service_red`);
  p95/p99 via percentile over span durations; throughput = span count /
  window; log_count = logs at ≥ severity within scope; metric = aggregate
  over a named metric series.
- **UI**: nav entry Alerts; rules list (status summary chips:
  firing/attention/healthy/disabled), rule create/edit (template presets:
  High error rate / Slow p95 / Slow p99 / Throughput drop / Log error
  burst), rule detail with a live chart of the measured signal + threshold
  reference line (plan-162 tokens) + recent evaluations table + incident
  history; incidents list + incident detail (timeline of
  triggered/renotified/resolved, link to the scoped traces/logs view);
  destinations settings page with "send test" button.

## Current state

(verified at `2288011`)

- No alerting anywhere: `grep -rn "alert" crates/ ui/src --include='*.rs' --include='*.tsx'`
  → no product hits.
- Turso bootstrap DDL in `crates/parallax-metadata/src/turso.rs` (tables:
  issues, runs→invocations, dashboards, investigations, saved_views…);
  migrations are idempotent `CREATE TABLE IF NOT EXISTS` blocks; CRUD
  modules per domain under `turso/`.
- Long-running server loops: `parallax-server` already runs tokio tasks
  (ingest workers, SSE broadcast) — follow the existing spawn/shutdown
  pattern in `crates/parallax-server/src/serve.rs`; graceful shutdown must
  stop the evaluator/delivery loops.
- Measurement primitives exist in `crates/parallax-greptime`
  (`service_red`, percentile queries, `log_count_series`, `metric_series`).
- GraphQL: Query/Mutation roots in `crates/parallax-api/src/lib.rs`;
  mutations exist for dashboards/investigations/saved views — model CRUD on
  those.
- UI nav: `ui/src/components/nav.ts` (`primaryNav`, Issues entry is the
  pattern for an "Investigate"-class page).
- Progress-visibility rule (AGENTS.md): the serve ready banner must mention
  the alert evaluator's status once it exists.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Metadata tests | `cargo nextest run --locked -p parallax-metadata` | pass |
| Server tests | `cargo nextest run --locked -p parallax-server -p parallax-api` | pass |
| Live engine | `cargo nextest run --locked -p parallax-server -E 'binary(/greptime/)'` | pass |
| UI gates | `cd ui && bun run typecheck && bun run lint && bun run check && bun run --bun test:ci && bun run build` | exit 0 |
| Breach corpus | playground `scenarios/run.sh a-breach-error-rate a-breach-p95 a-recover` | added by this plan |

## Scope

**In scope:**
- `crates/parallax-metadata` — five tables above + CRUD.
- New `crates/parallax-server/src/alerting/` (or a new
  `crates/parallax-alerting` if the server crate's module budget is
  tight — decide by existing crate-size conventions): evaluator loop,
  state machine (pure, unit-tested), delivery worker, webhook/slack
  senders (reqwest **native-tls**, repo TLS rule).
- `crates/parallax-greptime` — measurement query helpers where existing
  ones don't fit (windowed error-rate per service group).
- `crates/parallax-api` — alert rule/destination/incident queries +
  mutations + `alertChecks(ruleId)`.
- `ui/src` — `/alerts` (+ `/alerts/$ruleId`, `/alerts/incidents/$incidentId`,
  destinations section), nav entry, template presets, threshold-line chart.
- Playground (direct on its main): scenarios `a-breach-error-rate` (sustained >20%
  error rate on one service for ≥3 min), `a-breach-p95` (sustained slow
  handler), `a-recover` (breach then recovery) + matrix rows.
  *Preliminary (helper agent, 2026-07-17): the three scripts landed on the
  playground's main at `eee099a` (paymentFailure-flag breach with restore on
  exit, `?slow=` p95 breach, healthy-traffic recovery; `BREACH_SECONDS`/
  `RECOVER_SECONDS` knobs, default 200s). Catalog rows for `run.sh` sit in
  the playground working tree; peer verifies live against the evaluator and
  adds any missing rows. Corner-case-matrix rows for the three scenarios
  landed on the playground's main at `67be73a` (helper agent, 2026-07-17).*
- Ready-banner line + config knobs (`[alerting] enabled`, intervals) in
  `parallax-server` config with sane defaults.

**Out of scope:** anomaly detection, escalation policies, on-call
scheduling, PagerDuty/Discord destinations, alert-rule creation from
dashboard widgets (deferred to plan 168's graduation hook), AI triage,
multi-tenant concerns.

## Git workflow

- Work directly on `main` in BOTH repositories — no branches, no pull requests (operator
  delivery model, 2026-07-17; see plans/README.md Execution Preflight).
- Commit OFTEN: one small green slice per commit (a step, a component, a
  fixed defect), Conventional Commits, DCO `-s`, exactly one agent trailer.
- **Push to `main` immediately after every commit** — never batch pushes,
  never hold local-only work; never push a slice whose targeted checks are
  red. The parallax ruleset's "Bypassed rule violations" push notice is
  expected.

## Steps

### Step 1: Schema + state machine (pure)

Turso DDL + CRUD; the evaluation state machine as a pure function:
`(rule, prev_state, measurement) -> (next_state, transitions)` where
transitions ∈ {none, open_incident, resolve_incident, renotify}. Exhaustive
unit tests: breach hysteresis (2 consecutive), healthy hysteresis, min
sample count, no_data skip vs zero, between/not_between, renotify interval,
flapping (breach-heal-breach) never double-opens while an incident is open.

**Verify**: `cargo nextest run -p parallax-metadata` + state-machine tests
pass.

### Step 2: Evaluator + measurement queries

Loop with CAS claim; measurement per signal type against GreptimeDB;
audit rows; graceful shutdown. Live-engine test: seed spans shaped like a
breach window, run one evaluator tick synchronously (expose a
`tick_once()` for tests), assert state row + incident + outbox row +
audit row.

**Verify**: live-engine lane passes; `tick_once` deterministic under
repeated invocation (idempotent claims).

### Step 3: Delivery worker

Outbox claim-lease loop; webhook JSON schema (documented in the plan-owned
API docs section: rule, incident, value, threshold, window, links);
slack_webhook payload; backoff + max attempts + dead-letter status.
Tests with a local HTTP test server: success, 500-retry-backoff, lease
takeover after expiry, no double-delivery (unique delivery key).

**Verify**: server tests pass; no delivery without an incident transition.

#### Webhook payload schema (as implemented in `alerting/delivery.rs`)

Generic `webhook` destinations receive one JSON object per event
(content-type `application/json`); keys are stable for external
integrators. `slack_webhook` destinations instead receive Slack's
`{"text": "[FIRING|RESOLVED|STILL FIRING] <rule> (<severity>) group=…
value=… threshold=… — <incident url>"}` shape.

```json
{
  "event": "triggered | resolved | renotify",
  "rule": {
    "id": "…", "name": "…",
    "signal_type": "error_rate | p95_latency | p99_latency | throughput | log_count | metric",
    "severity": "warning | critical"
  },
  "incident": { "id": "…", "group_key": "…" },
  "value": 0.42,
  "threshold": 0.2,
  "threshold_upper": null,
  "window_minutes": 5,
  "links": { "incident": "<ui>/alerts/incidents/<id>", "investigate": "<ui>/traces?service=…" }
}
```

`value`/`threshold_upper` are `null` when absent. Peer re-verifies this
section against the final wired worker before retiring the plan.

### Step 4: GraphQL + UI

CRUD + lists + checks; UI pages per the reference contract (templates
pre-fill rule forms; rule detail chart = measured series + dashed threshold
reference line + breach markers; incidents link to scoped traces/logs).
Severity uses plan-162 ramp; incident rows follow the issues-table pattern.

**Verify**: API tests; UI gates green; component tests for rule form
validation (comparator/threshold_upper pairing), incident timeline
rendering.

### Step 5: Playground breach scenarios + live closure

Add the three scenarios (flag-driven sustained failure/latency via the
existing chaos flags). *Preliminary (helper agent, 2026-07-17): the three
scripts (`eee099a`) and the listener `scripts/webhook-listener.sh`
(`2892fae`, smoke-tested; default port 9099) are on the playground's main —
peer verifies against the evaluator.* Live walk: create a High-error-rate
rule scoped to
the breach service with a webhook destination pointed at a local listener
(`scripts/webhook-listener.sh`); run `a-breach-error-rate`;
watch the incident open in the UI (rule detail chart shows the breach),
webhook received; run `a-recover`; incident resolves; resolved webhook
received. Browser checklist walk over all alert pages; screenshots to
`docs/research/validation/2026-07-wave2/167/`.

**Verify**: end-to-end evidence: incident lifecycle timestamps + two
webhook payload captures + screenshots; clean console.

## Done criteria

- [ ] State-machine tests exhaustive and green; live-engine tick test
  green; delivery tests green (incl. no-double-delivery).
- [ ] UI gates green; alerts pages browser-verified per checklist.
- [ ] End-to-end: breach → open incident → webhook; recovery → resolved →
  webhook, all captured.
- [ ] Ready banner names the evaluator; config knobs documented.
- [ ] Playground scenarios + matrix rows landed on the playground's main.
- [ ] `plans/README.md` status row updated.

## STOP conditions

- Percentile measurement over the window is too slow for a 60s cadence on
  the live engine (>5s per rule) — report timings; pre-aggregation is a
  contract decision (native-tables rule), not an inline hack.
- No existing secret-handling exists for destination configs and the
  operator's threat model is unclear — ship webhook URLs as plain config
  with an explicit README warning and record the deferred decision; do NOT
  invent crypto.
- Email transport does not exist in-tree — defer the email type (record in
  status), ship webhook + slack_webhook.
- The evaluator loop interferes with ingest throughput (measure via the
  existing server benchmarks if present) — report before changing
  scheduling.

## Maintenance notes

- The state machine is pure — every future rule type (anomaly, pattern
  burst) plugs in as a new measurement, not a new machine.
- Reviewer focus: CAS claim correctness under concurrent servers, outbox
  idempotency, native-tls on reqwest, bounded audit retention.
- Plan 168's "graduate metric query → alert rule" pre-fills the rule form
  via URL params — keep form state URL-initializable.

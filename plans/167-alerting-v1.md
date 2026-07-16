# Plan 167: Alerting v1 — rules, evaluator, incidents, destinations

> **Executor instructions**: Follow this plan step by step. This plan adds a
> new product capability (backend + UI); read `ui/AGENTS.md` before UI work
> and apply its browser-verification checklist after every UI step against
> playground breach scenarios. STOP conditions binding. Update this plan's
> status row in `plans/README.md` when done.
>
> **Drift check (run first)**:
> `git diff --stat <wave2-base>..HEAD -- crates/parallax-metadata crates/parallax-api crates/parallax-server ui/src/components/nav.ts`
> `<wave2-base>` = the Wave-1 merge commit.

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
- Playground (linked PR): scenarios `a-breach-error-rate` (sustained >20%
  error rate on one service for ≥3 min), `a-breach-p95` (sustained slow
  handler), `a-recover` (breach then recovery) + matrix rows.
- Ready-banner line + config knobs (`[alerting] enabled`, intervals) in
  `parallax-server` config with sane defaults.

**Out of scope:** anomaly detection, escalation policies, on-call
scheduling, PagerDuty/Discord destinations, alert-rule creation from
dashboard widgets (deferred to plan 168's graduation hook), AI triage,
multi-tenant concerns.

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
existing chaos flags). Live walk: create a High-error-rate rule scoped to
the breach service with a webhook destination pointed at a local listener
(`scripts` helper printing received payloads); run `a-breach-error-rate`;
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
- [ ] Playground scenarios + matrix rows landed (linked PR).
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

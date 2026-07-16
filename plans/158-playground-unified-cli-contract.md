# Plan 158: Migrate the telemetry playground to the neutral CLI contract and enrich the CLI/interactive/jobs simulation

> **Executor instructions**: This plan edits the companion repository
> `tailrocks/parallax-telemetry-playground` (clone it as a sibling checkout;
> never vendor it into this repo). Follow step by step, run every verification
> command, honor STOP conditions, update this plan's status row in
> `plans/README.md` when done.
>
> **Drift check (run first, in the playground checkout)**:
> `git log --oneline -5` — this plan was written against playground `main`
> after PR #6 (all 6 PRs merged, no open branches). If newer commits touch
> `libs/playground-telemetry`, `cli/src`, `web/src/telemetry.ts`,
> `services/orders`, or `services/fulfillment`, compare the excerpts below
> before proceeding; on a mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: LOW-MED (emitter-side only; Parallax reads both old and new keys
  during the window, so ordering with plan 156 is soft)
- **Depends on**: plans/156-unified-cli-observability-contract.md (the
  regenerated semconv modules come from Parallax's contract.yaml)
- **Category**: migration / cross-repository
- **Planned at**: Parallax `39f172c`, 2026-07-17
- **Operator directive (2026-07-17)**: the playground is the realistic
  multi-architecture corpus (microservices + browser + CLI) that plans 157/159
  visualize and verify against. It lands as direct commits to the
  playground's `main` (operator delivery model 2026-07-17: no branches, no
  pull requests, in either repository).

## Why this matters

The playground is how Parallax develops against real-world-shaped telemetry
without depending on jackin❯ releases. Today it emits the retired vocabulary:
`parallax.run.id` resource attrs, `parallax.session.id`/
`parallax.execution.layer`/`parallax.agent.id`, `cron.invocation.id`,
non-namespaced `cli.command`, and messaging legs without `job.*`. After this
plan it emits the same neutral contract jackin❯ ships — so every plan-157
surface (invocation hub, sessions/screens/actions, jobs/cycles,
conversations, ecosystem kinds) renders from playground data, including
`interactive` and `capsule` app modes that no current emitter produces.

## Current state

(verified in the playground clone at recon time; all merged on `main`)

- **Shared Rust telemetry lib**: `libs/playground-telemetry/src/lib.rs` —
  `init(service)` builds traces+metrics+logs+Sentry; resource attrs at
  `:492-517` include conditional `parallax.run.id` from `PARALLAX_RUN_ID` env;
  semconv module `libs/playground-telemetry/src/semconv.rs` is **generated
  from Parallax** (`cargo xtask semconv --playground-root
  ../parallax-telemetry-playground generate`), as are `web/src/semconv.ts`
  and `services/semconv/src/main/java/io/tailrocks/semconv/Semconv.java`;
  wire contract fixture `fixtures/semconv-wire-contract.json`.
- **CLI simulator**: `cli/src/main.rs` — binary `playground`; modes: default
  `drive` (one-shot checkout driver `:96-104`), `cron` (`cron_job` span with
  `cron.job.name`/`cron.schedule`/`cron.invocation.id`/`cron.outcome`
  `:305-352`), `daemon` → child `enter` simulating host CLI → daemon →
  container → agent → tool (`:106-265`) with env-carrier propagation
  (`TRACEPARENT`/`BAGGAGE` + `parallax.run.id`), agent/tool spans carrying
  `gen_ai.operation.name`/`parallax.agent.id`/`tool.name`/`shell.command`
  (`:227-265`), non-namespaced `cli.command` attr (`:113`), flush-on-exit
  (`:60-63`). No `app.mode`, no `cli.invocation.id`, no `interactive`, no
  `capsule`, no `session.start/end`, no `ui.*` events, no `background.cycle`,
  no generic `outcome`.
- **Async legs**: `services/orders/src/main.rs` — in-process PRODUCER "send
  orders" → CONSUMER "process orders" with real span links (`:70-132`), batch
  fan-in (`:156-188`), `messaging.*` attrs, `messaging.queue.depth` gauge
  (`:261-276`). `services/fulfillment/.../FulfillmentApplication.java` — real
  Kafka leg with W3C headers + consumer link → payment gRPC → notifications
  HTTP. Neither carries `job.id`/`job.type`.
- **Browser**: `web/src/telemetry.ts` — WebTracerProvider + fetch/doc-load/
  user-interaction + web-vitals + logs (`:69-115,259-294`); resource includes
  exact `session.id` (`:71-77`); `ui.click`/`ui.submit` span/event names and
  `app.screen.name`/`app.widget.name` via generated semconv; same-origin
  `/v1/traces`+`/v1/logs` proxy to OTLP.
- **Test-run observability** (plan 154's W-work, complete): nextest→OTLP
  bridge `cli/src/test_report.rs`, GraphQL verifier `cli/src/test_verify.rs`,
  Java `OpenTelemetryTestExtension`, Playwright OTLP reporter
  `web/e2e/telemetry-reporter.ts`, orchestrator
  `scripts/observable-test-session.sh` gated on `PARALLAX_RUN_ID`+
  `TRACEPARENT`.
- **Orchestration**: `deploy/docker-compose.yml` (14 always-on containers +
  k6 demo profile; OTLP anchor `x-otlp` → `host.docker.internal:4317/4318`),
  `demo.sh`, `scenarios/run.sh`, overlays `docker-compose.xlang.yml`/
  `docker-compose.limits.yml`.

## Contract decisions (fixed)

1. Every emitter mints/propagates **`cli.invocation.id`** (UUIDv4) as the
   correlation id. The Rust CLI stamps it on its root spans and logs
   (jackin shape); service containers keep receiving it via the env carrier →
   resource attr path only where a process is genuinely part of a wrapped
   invocation (the CLI's own children). Long-running services do NOT carry an
   invocation id — they correlate by trace, exactly like production.
2. `parallax.run.id`, `parallax.session.id`, `parallax.execution.layer`,
   `parallax.agent.id`, `cron.invocation.id`, bare `cli.command` are
   **removed entirely** — plan 156 deletes their contract.yaml rows, so the
   regenerated modules no longer contain them; no emission, no fallback
   reads, anywhere (operator, 2026-07-17: generic attributes only).
3. `app.mode` values map: `drive`→`one_shot`, `cron`→`one_shot` (each firing
   is an invocation), `daemon`→`daemon`, container layer→`capsule`, new
   `console` mode→`interactive`.
4. Agent/tool sim upgrades to the fuller `gen_ai.*` set:
   `gen_ai.agent.name` (`claude|codex|amp`…), `gen_ai.conversation.id`
   (UUID per agent lifetime), `gen_ai.provider.name`; keep
   `gen_ai.operation.name` values `invoke_agent`/`execute_tool`.
5. Generic bounded `outcome` replaces `cron.outcome`; stable `error.type`
   stays as-is (already exact).
6. Jobs: the orders in-process leg and the fulfillment Kafka leg mint
   `job.id` (UUID) + `job.type` (`order_dispatch`, `fulfillment_shipment`)
   on their PRODUCER and CONSUMER spans (types enter Parallax's
   contract.yaml owner `playground`).
7. Background cycles: the daemon sim gains a periodic reconciliation loop
   emitting `background.cycle` root spans (`background.cycle.name` ∈
   `{queue_health, price_refresh}`) with no-op ticks metric-only.
8. Interactive sim (`playground console`): a scripted TUI session emitting
   `session.start` → `ui.screen.entered/exited` visits over screens
   `{home, cart, checkout}` with `ui.screen.visit.id`+`ui.navigation.sequence`,
   `ui.action` root spans (`ui.action.name` ∈
   `{cart.add, checkout.submit, screen.back}`) whose checkout action calls
   the real checkout service (trace crosses into the microservices), then
   `session.end`. Runs for `--seconds N` (default 30), suitable for the
   plan-159 live demo.
9. Browser telemetry keeps standard web semconv + `session.id`; no jackin
   `ui.*` TUI attrs forced onto the browser — the browser is its own emitter
   kind (ecosystem `browser` node), already correct.

## Commands you will need

(inside the playground checkout; toolchain via `mise install`)

| Purpose | Command | Expected on success |
|---|---|---|
| Regenerate semconv (from the parallax checkout) | `cargo xtask semconv --playground-root ../parallax-telemetry-playground generate` (run in the PARALLAX repo) | regenerated Rust/TS/Java modules + wire fixture |
| Rust build/tests | `cargo nextest run --workspace --profile ci --no-tests=fail` | all pass |
| Web | `cd web && bun install --frozen-lockfile && bun run build && bun run test` | exit 0 |
| Java | `./gradlew test` per service (or the repo's aggregate — read the CI workflow) | pass |
| Full stack | `docker compose -f deploy/docker-compose.yml up --build -d` | 14 containers healthy |
| CLI sims | `./target/debug/playground` / `… cron` / `… daemon` / `… console --seconds 30` | exit 0; spans visible at the OTLP endpoint |
| Teardown | `docker compose -f deploy/docker-compose.yml --profile demo down` | clean |

## Scope

**In scope (playground repo):**
- `libs/playground-telemetry/src/{lib.rs,semconv.rs,propagation.rs}` —
  resource attrs, ambient invocation/session plumbing, env carrier renames
  (`PARALLAX_RUN_ID` → `CLI_INVOCATION_ID`).
- `cli/src/main.rs` (+ new `cli/src/console_sim.rs`, `cli/src/cycles.rs`) —
  modes, identity, events, outcome.
- `cli/src/{test_report.rs,test_verify.rs}` — parent/verify by invocation id.
- `services/orders/src/main.rs`, `services/fulfillment/**` — job keys.
- `web/src/semconv.ts` (generated), `web/src/telemetry.ts` (only if the
  generated module forces renames).
- `deploy/docker-compose.yml` env keys, `.env.example`,
  `scripts/observable-test-session.sh`, `scenarios/**` where they export
  `PARALLAX_RUN_ID`.
- Docs: `README.md`, `docs/execution-stack-design.md`,
  `docs/frontend-telemetry-contract.md`, `docs/telemetry-events.md`.
- In the PARALLAX repo: only `telemetry/semconv/contract.yaml` additions with
  owner `playground` (`job.type` values, screen/action value sets) +
  regeneration — coordinate with plan 156's registry step (same branch).

**Out of scope:**
- Parallax backend/UI code (plans 156/157).
- New services, new infra containers, k6 profiles, chaos flags.
- Sentry paths, exemplar strategy, Java agent versions.
- Removing the test-observability W-work (it is complete; only its
  correlation env/attrs re-key).

## Git workflow

- Work directly on the playground's `main` (operator delivery model
  2026-07-17: no branches, no pull requests). Conventional Commits, DCO
  `-s`, exactly one agent trailer, push after every durable green commit.

## Steps

### Step 1: Registry + regeneration (coordinates with plan 156 step 1)

Add playground-owned rows to Parallax `telemetry/semconv/contract.yaml`
(`job.type` values `order_dispatch`/`fulfillment_shipment`,
`background.cycle.name` values `queue_health`/`price_refresh`,
`app.screen.id` values `home`/`cart`/`checkout`, `ui.action.name` values
`cart.add`/`checkout.submit`/`screen.back`, `gen_ai.agent.name` +
`gen_ai.provider.name` sample sets) unless 156 already added them. Run the
generator; commit regenerated modules in BOTH repos (each repo's commit in
its own branch).

**Verify**: playground `cargo check --workspace` + `cd web && bun run build`
compile against the regenerated modules.

### Step 2: Identity plumbing in the shared lib

`libs/playground-telemetry`: replace the `parallax.run.id` resource attr with
nothing (ids are not Resource); add an `invocation` module minting
`cli.invocation.id`/`session.id` UUIDs, ambient storage (OnceLock), and
helpers `stamp_invocation(span)` / log-attr injection so root spans and logs
carry the ids; env carrier reads/writes `CLI_INVOCATION_ID` only —
`PARALLAX_RUN_ID` is neither read nor written. Long-running services
(`checkout` etc.) simply never call the mint — `init(service)` gains no new
required arguments.

**Verify**: `cargo nextest run -p playground-telemetry` → new tests: root
span carries the id; resource does NOT; child env carrier round-trips.

### Step 3: CLI modes

`cli/src/main.rs`:
- All modes: mint invocation id at startup; root span per mode named
  `cli.command`; attrs `cli.command.name`
  (`drive`/`cron`/`daemon`/`console`/`test-report`), `app.mode`, `outcome`,
  `process.exit.code`; logs stamped with the id.
- `cron`: drop `cron.invocation.id`/`cron.outcome` in favor of the generic
  keys (keep `cron.job.name`/`cron.schedule`).
- `daemon`/`enter`: execution layers re-tagged via `app.mode`
  (`daemon`/`capsule`); agent/tool spans gain `gen_ai.agent.name`,
  `gen_ai.conversation.id`, `gen_ai.provider.name`; drop
  `parallax.agent.id`/`parallax.execution.layer`; daemon gains the periodic
  `background.cycle` loop (decision 7) while it runs.
- New `console` mode (decision 8) in `cli/src/console_sim.rs`: session
  events, screen visits with monotonic sequence, `ui.action` roots, real
  checkout call inside `checkout.submit`, session end + flush.

**Verify**: `cargo nextest run -p playground-cli` (or the crate's real name —
check `cli/Cargo.toml`) → unit tests for command-name/app-mode mapping,
outcome mapping, screen-visit pairing; manual:
`./target/debug/playground console --seconds 5` exits 0.

### Step 4: Jobs on the async legs

`orders`: mint `job.id`/`job.type=order_dispatch` at the PRODUCER site
(`:70-77`), carry through the queue payload, stamp on CONSUMER spans
(`:113-132`) including batch fan-in and dead-letter paths (poison →
`outcome=failure`). `fulfillment`: same via Kafka headers
(`job.type=fulfillment_shipment`), consumer link already exists.

**Verify**: orders unit tests assert producer/consumer share `job.id`; Java
`./gradlew :fulfillment:test` passes with the header round-trip test.

### Step 5: Test-observability re-key + scripts + docs

`test_report.rs`/`test_verify.rs`/`observable-test-session.sh`/compose env/
`.env.example`: `PARALLAX_RUN_ID` → `CLI_INVOCATION_ID`, hard rename, no
fallback; `test_verify` queries the renamed GraphQL fields (`invocation`,
`tracesByInvocation` — plan 156's SDL). Update the four doc files to the
neutral vocabulary.

**Verify**: `rg -n "PARALLAX_RUN_ID|parallax\.run\.id|parallax\.session\.id|parallax\.agent\.id|parallax\.execution\.layer|cron\.invocation\.id"`
→ zero matches repo-wide (docs included); full playground test suite green.

## Test plan

- playground-telemetry: identity minting/ambient/carrier tests (step 2).
- CLI: mode→(`cli.command.name`,`app.mode`) exhaustive match test; console
  sim screen/action/session pairing test; cron outcome mapping.
- Orders/fulfillment: shared-job-id tests both legs.
- Wire-contract: regenerate `fixtures/semconv-wire-contract.json` and keep
  the cross-language `SemconvWireContractTest` green.
- End-to-end (consumed by plan 159): compose up + one pass of every CLI mode
  + one browser session + one observable test session.

## Done criteria

- [ ] Playground workspace tests, web tests, Java tests all green.
- [ ] Step-5 grep shows no legacy-key emission.
- [ ] Every CLI mode run against a live Parallax (`parallax serve`) produces
  an invocation visible via GraphQL `invocation(invocationId:)` with correct
  `appMode`/`commandName`/`outcome` (use `playground test-verify` or a curl
  check — exact assertion lives in plan 159).
- [ ] `console` mode produces ≥2 screen visits, ≥2 ui.action roots, one
  session pair; `daemon` mode produces ≥1 `background.cycle` root; orders +
  fulfillment produce linked PRODUCER/CONSUMER spans sharing `job.id`.
- [ ] Docs updated; `plans/README.md` status row updated (in the Parallax
  repo).

## STOP conditions

Stop and report back (do not improvise) if:
- The semconv generator cannot express a needed row (value-set constants) —
  extend the generator in the Parallax repo first (plan 156's owner), do not
  hand-edit generated modules.
- Plan 156's GraphQL renames have not landed on parallax main yet when
  `test_verify` is re-keyed — coordinate ordering, do not point the verifier
  at legacy fields.
- The Java OTel agent drops unknown span attributes (it must not — they are
  plain attrs); if job keys vanish on the wire, capture the OTLP payload
  evidence and report.
- Kafka header size or payload schema issues force protocol changes on the
  fulfillment leg.

## Maintenance notes

- The playground is the reference emitter for the neutral contract — when
  jackin❯'s registry grows, mirror the keys here through contract.yaml, never
  ad hoc.
- Reviewer focus: ids on root spans/logs but never Resource for the CLI; no
  invocation id on long-running service resources; producer/consumer job ids
  equal across process boundaries (Kafka headers).
- No legacy key survives anywhere (operator, 2026-07-17); if an external
  tool still exports `parallax.run.id`, it is unsupported — do not add
  compatibility reads back.

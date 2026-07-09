# Plan 056: Playground typed log events + structured logs in every tier (Rust business events, Java app logging, web OTLP logs)

> **Executor instructions**: This plan targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update the
> status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- libs/playground-telemetry services/checkout services/orders services/catalog services/payment web scenarios`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plan 036 (shared-lib propagation/identity helpers) — land
  036 first; plan 037's scenario catalog receives a new row
- **Category**: direction
- **Planned at**: commit `408be17`/`ed5b10f` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

The OTel Logs Data Model separates plain log records from **typed events**
(`EventName` + known attributes). Parallax now stores and displays native log
event identity — but nothing in the playground emits one, so the feature would
demo empty. Structured-log coverage is also lopsided today:
Rust services emit rich key/value `tracing` fields, the Java tier emits **no
application logs at all** (only the OTel agent's auto-captured framework
logs), and the web app logs only through Sentry. This plan defines a small
business-event taxonomy and emits it from all three tiers, giving Parallax's
Logs/Field-Explorer/story surfaces cross-language typed evidence.

## Current state

Verified at playground commit `ed1f975`.

- Rust logs are structured but untyped —
  `services/checkout/src/main.rs:97`:

  ```rust
  tracing::info!(tenant = %tenant, user.tier = %p.tier, "baggage business context");
  ```

  The shared bridge (`libs/playground-telemetry/src/lib.rs:89-108`) turns
  `tracing` events into OTLP LogRecords via `OpenTelemetryTracingBridge`,
  trace-correlated — but the `tracing` macro path has no way to set OTel
  `EventName`, so every record lands as a plain log.

- Java services have zero application logging: grep for
  `log.info|LoggerFactory|slf4j|System.out` across
  `services/{catalog,payment,fulfillment}/src/**/*.java` → no hits. Only a
  logback MDC pattern exists
  (`services/catalog/src/main/resources/application.yml`:
  `trace_id=%mdc{trace_id} span_id=%mdc{span_id}`), and the OTel Java agent
  (`deploy/Dockerfile.java`) auto-captures logback output.

- Web emits logs only through Sentry
  (`web/src/instrument.client.ts:29,36` — `enableLogs: true`,
  `consoleLoggingIntegration()`); OTel browser logs are not wired
  (`web/src/telemetry.ts` configures traces only: `WebTracerProvider`,
  `telemetry.ts:24-33`).

- No `EventName` anywhere: repo grep for `event.name|EventName|LogRecord`
  in emit paths → only comments.

- Scenario runner/catalog: plan 037 creates `scenarios/run.sh` +
  `scenarios/README.md` catalog; this plan appends one row. If 037 has not
  landed, add a plain `scenarios/a29-typed-events.sh` and note it.

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Rust build | `rtk cargo build` | exit 0 |
| Rust lint | `rtk cargo clippy --all-targets -- -D warnings` | exit 0 |
| Java build | `cd services/catalog && ./gradlew build -x test` (verify wrapper location first) | exit 0 |
| Web build | `cd web && bun run build` | exit 0 |
| Script lint | `bash -n scenarios/<new>.sh` | exit 0 |

## Scope

**In scope** (playground repo):
- `libs/playground-telemetry/src/lib.rs` (typed-event emit helper over the
  OTel logs API)
- `services/checkout/src/main.rs`, `services/orders/src/main.rs` (emit
  business events)
- `services/catalog/` and `services/payment/` (SLF4J structured app logs +
  one typed event each)
- `web/src/` (one typed browser event via the OTel logs SDK **only if** the
  browser logs packages are stable enough — see Step 4's decision gate)
- `scenarios/a29-typed-events.sh` (create) + catalog row
- `docs/` note listing the event taxonomy (small table in an existing doc or
  a new `docs/telemetry-events.md`)

**Out of scope**:
- Parallax-side rendering — already landed in the native Event column.
- RUM journey routes/vitals — plan 050 (its `route_view`/`user_step` events
  should adopt this taxonomy when it lands — coordination note only).
- Log field-spike / uncorrelated-log scenarios — plan 054.
- Changing the Sentry log path — it stays as-is for comparison.

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Event taxonomy (10 minutes of design, written down)

Define these typed events (names low-cardinality, values in attributes):

| event.name | Emitted by | Attributes |
|------------|-----------|------------|
| `checkout.completed` | checkout (Rust) | `sku`, `quantity`, `order.total` |
| `checkout.failed` | checkout (Rust) | `sku`, `error.type` |
| `order.consumed` | orders (Rust) | `order_id`, `poison` |
| `catalog.products.served` | catalog (Java) | `product.count`, `catalog.promo` |
| `payment.authorized` | payment (Java) | `payment.method` (static demo value) |
| `web.checkout.submitted` | web (TS, if Step 4 proceeds) | `sku`, `quantity` |

Record the table in the docs note. Do not invent more events.

### Step 2: Rust typed-event helper + emissions

The `tracing` macros cannot set OTel `EventName`; emit typed events directly
through the OTel logs API alongside the existing `tracing` line:

1. In `libs/playground-telemetry/src/lib.rs`, add a public helper:

   ```rust
   /// Emit a typed OTel log event (EventName set) on the global logger,
   /// correlated to the current span context.
   pub fn emit_event(name: &'static str, attrs: &[(&'static str, String)]) { ... }
   ```

   Implementation: use `opentelemetry::logs` API — a `LogRecord` with
   `set_event_name(name)`, severity INFO, current `Context` for trace
   correlation, attributes from `attrs`. Keep the provider handle accessible
   (the `SdkLoggerProvider` is already built at `lib.rs:93-96`; store a
   global logger or return one from `init`). Check the installed
   `opentelemetry` crate version's logs API for the exact method names —
   `set_event_name` exists in the 0.27+ logs API. If the installed version
   has no event-name setter, STOP (see conditions).
2. Call it at the natural points in checkout
   (`services/checkout/src/main.rs` — success return ~line 166+, failure
   branch at `:137-155`) and orders' consumer path, per the Step 1 table.
   Keep the existing `tracing::…` lines — they are the plain-log
   comparison data.

**Verify**: `rtk cargo build && rtk cargo clippy --all-targets -- -D warnings`
→ exit 0. Unit test in `libs/playground-telemetry`: helper sets the event
name (assert via the in-memory exporter if the SDK offers one; else mark the
check as covered by Step 5's live run).

### Step 3: Java structured logs + typed event

1. In `services/catalog` and `services/payment`, add an SLF4J logger and log
   the business moment with key/values. The OTel Java agent maps logback
   key/value pairs (or MDC) to log attributes; use the
   `org.slf4j.Logger` fluent API (`log.atInfo().addKeyValue(...)`) so pairs
   become structured attributes — verify against the agent version in
   `deploy/Dockerfile.java`.
2. Typed event: the agent's logback bridge does not set OTel `EventName`
   from SLF4J. Use the `io.opentelemetry.api` logs API (available on the
   agent classpath) via `GlobalOpenTelemetry` → `logRecordBuilder` …
   `setEventName("catalog.products.served")` if the API version exposes it.
   If the bundled API has no `setEventName`, emit the log with an
   `event.name` **attribute** instead and record that honestly in the docs
   note (Parallax's `event_name` column stays empty for Java until the agent
   catches up — that asymmetry is itself demo-worthy).

**Verify**: Java build exit 0; with the stack up, catalog logs appear with
key/value attributes (check via Parallax `/logs` or the lab).

### Step 4: Web typed event (decision-gated)

Check `web/package.json` and the npm registry for
`@opentelemetry/sdk-logs` + `@opentelemetry/exporter-logs-otlp-http` at
versions compatible with the pinned `@opentelemetry/sdk-trace-web`. Browser
logs are still experimental upstream:
- If compatible stable(ish) versions exist: wire a minimal `LoggerProvider`
  in `web/src/telemetry.ts` exporting to the same-origin proxy (mirror the
  trace exporter at `telemetry.ts:31`), and emit `web.checkout.submitted` on
  the existing checkout button handler.
- If not: **skip this step**, note the version matrix in the docs note, and
  leave the web tier Sentry-only. Do not force incompatible packages.

**Verify**: `cd web && bun run build` → exit 0 either way.

### Step 5: Scenario + catalog row

`scenarios/a29-typed-events.sh`: drive one clean checkout + one failed
checkout + one orders publish/consume round; print "Check in Parallax:
Logs → enable the Event column → `checkout.completed`, `checkout.failed`,
`order.consumed` rows appear; doc viewer shows event.name". Register in
`scenarios/run.sh` + `scenarios/README.md` (plan 037 format) if present.

**Verify**: `bash -n scenarios/a29-typed-events.sh` → exit 0; live run
recorded (requires the Parallax Event column; verify via SQL if needed:
`SELECT "event.name", body FROM opentelemetry_logs WHERE "event.name" IS NOT NULL LIMIT 10`,
and say which check you ran).

## Test plan

- Rust: `libs/playground-telemetry` unit test for `emit_event` (name +
  attribute mapping) as far as the SDK allows in-process.
- Java/web: build gates + the a29 live scenario are the integration tests
  (match the repo's existing no-unit-test convention for services; note it).
- Scenario script: `bash -n` + live run.

## Done criteria

- [ ] `rtk cargo build` + clippy `-D warnings` → clean
- [ ] Java services build; catalog/payment emit structured app logs
- [ ] `emit_event` helper exists and checkout/orders call it at the Step 1
      taxonomy points
- [ ] `scenarios/a29-typed-events.sh` exists, lints, and is cataloged
- [ ] Live verification recorded (Parallax Event column, or SQL fallback)
- [ ] Docs note with the taxonomy table + any honest gaps (Java EventName,
      web skip) committed
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- The installed Rust `opentelemetry` logs API has no event-name setter —
  report the crate version and the API you found; upgrading the OTel crate
  family is its own change (version policy: latest stable — if an upgrade is
  needed, say so, don't do it silently here).
- The OTel Java agent version rejects direct `GlobalOpenTelemetry` log
  emission — fall back to the `event.name` attribute (allowed) but STOP if
  even attribute-carrying app logs don't reach OTLP.
- Web packages incompatible (Step 4) — skip is the designed outcome, not a
  failure; report the matrix.

## Maintenance notes

- Plan 050 (RUM journey) should reuse this taxonomy for `route_view`/
  `user_step` events instead of inventing parallel names.
- Plan 066 (semconv registry) will lift the Step 1 table into the shared
  registry — keep the docs note table machine-readably simple.
- Reviewer: event names must stay low-cardinality; values belong in
  attributes (research-brief rule).

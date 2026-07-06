# Plan 050: Frontend RUM journey — multi-route web app, OTLP web vitals + session.id, propagation-break case, unload flush

> **Executor instructions**: Targets the **playground repository**
> (`parallax-telemetry-playground`), `web/` tier. Follow step by step; run
> every verification. On any STOP condition, stop and report. When done,
> update the status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- web scenarios deploy/Dockerfile.web deploy/docker-compose.yml`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: LOW
- **Depends on**: plan 036 (checkout CORS allows `traceparent` — without it
  the browser→backend join fails at preflight)
- **Category**: direction
- **Planned at**: commit `408be17` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

Parallax's frontend/RUM lane (brief section H "Frontend/session
observability", backlog A28) needs a browser journey: routes, user steps,
web vitals, errors linked to backend traces, and a session identity — all
via **OTLP**, not only Sentry. Today the web app is one route with three
buttons; web vitals exist only inside Sentry's integration (invisible to
Parallax); there is no `session.id`, no user-step span events, no
deliberate propagation-break case, and the `BatchSpanProcessor` never
flushes on unload so the last spans of any journey are dropped. The
Parallax-side `frontendSessions` API was audited as **blocked precisely on
this missing data** — this plan unblocks it and defines the data contract.

## Current state

Verified at playground commit `ed1f975`.

- One route only: `web/src/routes/` = `__root.tsx` + `index.tsx`. The page
  (`index.tsx:9-50`): a checkout fetch button, an intentionally-dead
  "apply promo (unresponsive)" button (rage-click demo is Sentry-Replay-only,
  `:30-37`), and a `break (RUM error)` button whose bare `throw` goes to
  `Sentry.captureException` only (`:39-46`) — no OTel event, no backend
  link.
- OTel web setup (`web/src/telemetry.ts:23-46`): `WebTracerProvider` with
  resource `service.name=web`, `service.version=VITE_RELEASE|dev`,
  `deployment.environment.name=playground` (`:25-29`) — **no `session.id`**;
  `BatchSpanProcessor` → same-origin `/v1/traces` proxy (`:30-33`) — **no
  unload flush**; propagator `W3CTraceContextPropagator` only (`:36`) — no
  baggage; instrumentations: DocumentLoad, Fetch (propagating to
  `apiTargets`), UserInteraction (`:38-45`).
- Sentry-only vitals: `web/src/instrument.client.ts:31-37`
  (`browserTracingIntegration` = LCP/CLS/INP/FCP/TTFB inside Sentry).
- The OTLP proxy route exists (`web/src/routes/v1.traces.ts` per repo
  inventory) targeting `ROTEL_OTLP_HTTP_ENDPOINT`
  (`deploy/docker-compose.yml:144`) — works unchanged for `parallax serve`
  (`:4318`).
- Bun-only rule (playground + Parallax convention): all web tooling through
  Bun.

## Commands you will need

| Purpose | Command (from `web/`) | Expected |
|---------|----------------------|----------|
| Install | `bun install` | exit 0 |
| Typecheck/build | `bun run build` | exit 0 (repo README cites build + type-check as the gate) |
| Dev | `bun run dev` | serves |

## Scope

**In scope** (playground repo):
- `web/src/routes/` (new routes: `checkout.tsx`, `orders.$orderId.tsx` or
  similar — 2-3 real pages), `web/src/routes/index.tsx` (nav + journey
  start)
- `web/src/telemetry.ts`, `web/src/instrument.client.ts` (session.id,
  vitals→OTLP, baggage propagator, unload flush, break-propagation knob)
- `web/package.json` (web-vitals emission dep if needed — prefer the
  standard `web-vitals` package or OTel's browser event conventions;
  decide in Step 2)
- `scenarios/a28-rum-journey.sh` (create; drives via `curl`-able SSR +
  documented manual browser steps, or Playwright ONLY if it already exists
  in the repo — it does not; so: manual-steps doc + optional Bun+fetch
  smoke)
- Data-contract note for the Parallax repo (see Step 5 — a doc file in the
  PLAYGROUND repo; the Parallax resolver is a separate future plan)

**Out of scope**:
- Session replay (explicit brief rule: journey/story, not replay).
- The Parallax `frontendSessions` resolver/UI (future Parallax plan; this
  plan produces its data + contract).
- Adding Playwright to the repo (brief allows it only "if it already fits";
  it's not present — defer).
- Checkout-service CORS (plan 036).

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: session.id + baggage + unload flush

In `web/src/telemetry.ts`:
1. Mint a session id per tab session:
   `sessionStorage`-backed UUID (`crypto.randomUUID()`), set as resource
   attr `session.id` (OTel session convention — the brief's identity table
   prefers standard `session.id` for browser sessions).
2. Propagator → composite: `W3CTraceContextPropagator` +
   `W3CBaggagePropagator` (from `@opentelemetry/core`); put
   `session.id=<id>` into baggage on init so backends can correlate.
3. Unload flush: on `visibilitychange === "hidden"` and `pagehide`, call
   `provider.forceFlush()` (keep a module ref to the provider).

**Verify**: `bun run build` → exit 0; dev run: spans in the network tab to
`/v1/traces` include the `session.id` resource attr (record).

### Step 2: Web vitals over OTLP

Emit vitals into the OTLP path (not just Sentry): add the `web-vitals`
package (latest stable, via `bun add`); on each metric (LCP/CLS/INP/FCP/
TTFB) emit an OTel **span event on a dedicated short span** OR a log-style
event via the events API if the installed OTel-web version exposes it —
check `@opentelemetry/sdk-trace-web`'s current guidance; the pragmatic
shape: one `browser.web_vital` span per report with attrs
`web_vital.name`, `web_vital.value`, `web_vital.rating`, `app.screen.name`.
Keep names aligned with the brief's `browser.web_vital` event convention
(Development-status semconv — note that in a comment).

**Verify**: `bun run build` exit 0; dev run shows `browser.web_vital` spans
arriving (record one LCP value).

### Step 3: Real journey — routes + user-step events

1. Add routes: `/checkout` (SKU picker + quantity + submit → checkout
   fetch; success/failure rendered) and `/orders` (calls orders `POST
   /order` — note the orders port 8092 must be reachable from the browser;
   add `VITE_ORDERS_URL` env mirroring the checkout one, and orders needs
   the same CORS treatment — if plan 036 scoped CORS to checkout only,
   apply the same layer to orders here or STOP if that edit conflicts).
2. Emit user-step span events on the interactions the brief names:
   `app.screen.name` on route enter (a small route-change span via the
   router's subscribe hook), `ui.click`/`ui.submit` events with
   `app.widget.name` (the UserInteraction instrumentation gives generic
   click spans; ADD the semantic attrs via a shared `trackStep(name, attrs)`
   helper that wraps `tracer.startSpan` — keep names low-cardinality).
3. Give the "break (RUM error)" button a real story: throw inside a traced
   fetch handler so the error span links the in-flight backend trace; ALSO
   report it as an OTel exception event (`span.recordException`) so
   Parallax sees it, not just Sentry.
4. Keep the rage-click button; add a `ui.click` event emission so the
   repeated clicks are OTLP-visible too (frustration signal without
   replay).

**Verify**: `bun run build` exit 0; manual journey (documented clicks)
produces: route spans for `/` → `/checkout`, a submit → backend-stitched
trace, an error span with exception event + trace link (record trace ids).

### Step 4: Propagation-break case

Add `?nopropagate=1` handling in the checkout page's fetch: when set, do
the fetch with a plain `fetch` bypassing instrumentation propagation (e.g.
construct a Request in a way the FetchInstrumentation ignores, or point at
an origin NOT in `apiTargets` — simplest honest mechanism: add
`VITE_CHECKOUT_URL_NOPROP` that is the same backend via `127.0.0.1` instead
of `localhost`, which won't match the propagation allowlist). The result:
browser span exists, backend trace exists, but they are disconnected — the
missing frontend→backend continuation evidence gap (brief section L
"Telemetry quality demo"). Document in the scenario.

**Verify**: dev run with the knob: two disconnected traces confirmed
(record both ids).

### Step 5: Scenario + data contract note

1. `scenarios/a28-rum-journey.sh`: prints the manual journey script (open
   URL, click sequence, the nopropagate variant) and curls the SSR pages as
   a smoke check; registers in `scenarios/run.sh` + README ("Check in
   Parallax: Traces — browser root span stitched to checkout; web_vital
   spans; the nopropagate run shows the broken-continuation gap").
2. Write `docs/frontend-telemetry-contract.md` (playground repo): the exact
   resource attrs (`service.name=web`, `session.id`), span/event names
   (`browser.web_vital`, `ui.click`, `ui.submit`, `app.screen.name`), and
   the propagation-break semantics — this is the input contract for the
   future Parallax `frontendSessions` resolver (Parallax-side audit
   confirmed storage handles it: span events JSON + auto-widened
   `resource_attributes.session.id` column).

**Verify**: `bash -n` the scenario; contract doc lists every emitted
name/attr introduced by this plan (cross-check against the code).

## Test plan

- `bun run build` is the web gate (repo convention). If the web app has a
  test runner configured (check `web/package.json` scripts), add a unit
  test for the session-id helper (stable within a tab session); if none
  exists, note it and rely on the recorded manual journey.

## Done criteria

- [ ] `session.id` resource attr + baggage on all browser telemetry;
      unload flush wired
- [ ] `browser.web_vital` spans arrive over OTLP (recorded values)
- [ ] ≥3 routes; user-step events (`ui.click`/`ui.submit` +
      `app.screen.name`) emitted; error button produces an OTel exception
      linked to a backend trace
- [ ] `?nopropagate` run produces the documented disconnected-trace gap
      (recorded)
- [ ] `a28-rum-journey.sh` + `docs/frontend-telemetry-contract.md`
      committed; catalog rows added
- [ ] `bun run build` exit 0; no non-Bun lockfile appears
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- The installed OTel-web packages' versions lack a workable events/span
  API for vitals without major upgrades — report the version matrix before
  upgrading everything.
- Orders CORS/port wiring conflicts with plan 036's scope decision —
  coordinate rather than duplicating CORS layers divergently.
- UserInteractionInstrumentation double-counts your manual `ui.click`
  spans in a confusing way — pick ONE source per interaction and document
  it; report if the instrumentation can't be scoped.

## Maintenance notes

- The Parallax-side consumer (a `frontendSessions` resolver + a sessions
  UI) is deliberately NOT planned yet — it should be specced against
  `docs/frontend-telemetry-contract.md` once this data flows; record that
  in the plans index when closing this plan.
- Plan 054's TOUR doc should include the RUM journey beat.
- Reviewer: session.id must NOT leak into metric labels (cardinality rule);
  vitals span names stay `browser.web_vital` (not per-metric names);
  the nopropagate mechanism must not accidentally break the normal path's
  allowlist.

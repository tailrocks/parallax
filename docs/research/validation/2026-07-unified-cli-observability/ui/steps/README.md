# Plan 157 per-step browser verification (2026-07-17)

Stack: `parallax serve` (managed GreptimeDB 1.1.2 + Turso, embedded UI build)
on `127.0.0.1:4000`; corpus seeded over OTLP/HTTP protobuf (interactive
jackin-style invocation `9f6b…` with sessions/screens/actions/errors/cycle/
job/conversation signals, an observed-only one-shot `b1e0…`, and a plain
`payments-api` service). The `agent-browser` CLI daemon failed on
`Page.captureScreenshot` on this host (CDP timeout reproducible on
example.com), so captures use the sanctioned fallback, Chrome DevTools MCP;
interaction checks used both the agent-browser a11y snapshots and the
DevTools a11y tree.

Checklist (plan 157 protocol) — all six items pass on the walked pages:

1. **Data correctness** — every list/hub value traces to the seeded corpus
   (commands, modes, session `sess-1`, screen dwells 16.0s/11.9s/49.9s,
   cycle p50/p95, job attempt, conversation token totals 1.2k/310).
2. **Links** — invocation rows → hub; actions/cycles/jobs → `/traces/$id`;
   journey errors → `/issues/$fingerprint`; ecosystem cli nodes →
   `/invocations?q=<service>` (verified in the a11y tree URLs).
3. **States** — populated tabs, the observed-only invocation renders the
   hub without a registration row, and the Sessions tab shows the explicit
   empty card (`10-hub-sessions-empty-1440.png`).
4. **Layout** — 1440px and 375px captures; wide tables scroll inside their
   own container (fixed from `overflow-hidden` during this walk); long ids
   truncate with tooltips.
5. **Live behavior** — `live=true` opens both SSE streams (green indicator),
   re-seeded events prepend newest-first without duplicates
   (`09-hub-logs-live-1440.png`).
6. **Console** — zero errors/warnings across the walk (DevTools
   `list_console_messages` empty, preserved across navigations).

Defects found and fixed during this walk (same-day commits):

- Auto-registered external invocations rendered a `cli` source badge — the
  derived `status` had replaced the raw registration marker; GraphQL now
  exposes `Invocation.registration` and the UI consumes it.
- `ObservedInvocation.lastCommand` / `appMode` were populated in storage but
  not exposed over GraphQL.
- Invocation table clipped columns at 1440px (`overflow-hidden` →
  `overflow-x-auto`).
- Cycle percentiles used biased rounding (p50 == p95 for two samples) —
  switched to standard nearest-rank.

Captures:

| File | What it proves |
|---|---|
| `01-invocations-list-1440.png` | merged list: commands, modes, statuses, outcome/error/session columns, External badges |
| `02-hub-overview-1440.png` | hub stat cards, invocation-scoped process metrics, journey-aware story beats |
| `03-hub-sessions-1440.png` | session picker, screen-visit Gantt lane, actions table |
| `04-hub-journey-1440.png` | the journey narrative: session → screens (dwell) → actions (outcome) → **errors attributed to the Capsules screen** → session end; conversation token totals |
| `05-hub-jobs-cycles-1440.png` | cycle aggregates (runs/errors/p50/p95) + job producer/attempt chips |
| `06-hub-errors-1440.png` | error.type breakdown bars + correlated issues |
| `07-ecosystem-kinds-1440.png` | cli/browser/service node kinds + legend |
| `08-invocations-list-375.png` | mobile layout, in-container horizontal scroll |
| `09-hub-logs-live-1440.png` | live streaming logs, newest-first prepend |
| `10-hub-sessions-empty-1440.png` | explicit empty state for a one-shot invocation |

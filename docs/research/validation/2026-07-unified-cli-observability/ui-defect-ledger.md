# UI defect ledger (plan 160)

Audit date: 2026-07-17. Corpus: playground `docs/corner-case-matrix.md`
(plan 161), seeded fresh against the live 12-container compose stack into a
live `parallax serve` (http://127.0.0.1:4000, managed GreptimeDB 1.1.2).
Browser tooling: Chrome DevTools MCP (sanctioned fallback; the agent-browser
CDP screenshot path is broken on this host — see the plan-157 evidence
README). Checklist per cell: (1) data correctness vs matrix expectation,
(2) links navigate, (3) empty/loading/error states, (4) layout 1440px +
375px (no body horizontal scroll on traces/logs/invocations at 375px),
(5) live behavior, (6) clean console (verified zero console messages on the
trace surfaces).

Screenshots: `ui/audit/*.png` in this directory. Public task names are used in corpus references; screenshot filenames retain internal fixture IDs.

## Defect records

All fixes landed as individual commits on `main` with red→green regression
tests; every non-cosmetic defect was browser re-verified on the live corpus.

| Id | Surface | Corpus id | Defect | Root cause | Fix commit |
|---|---|---|---|---|---|
| D-001 | Traces list | sweep | List showed 3–6 traces regardless of traffic (835 in window) | Live engine mis-executes subquery-to-subquery equi-joins on tag columns; service filter used an `IN (SELECT …)` semi-join that returns zero rows | `b6c3b36` |
| D-002 | Waterfall | `traces:deep` | Span names rendered one character per line past depth ~6 | 11rem label column + uncapped linear depth padding + `break-words` | `06d490c` |
| D-003 | Waterfall | `traces:deep` | All bars fallback-grey, chips printed `SPAN_KIND_INTERNAL` | kindMap keyed by bare names, wire sends `SPAN_KIND_*` | `06d490c` |
| D-004 | Trace detail | `traces:wide` | 521-span trace silently truncated to 500 | Whole-trace reads capped at list-page `MAX_ROWS` | `06d490c` |
| D-005 | Links/events panels | `traces:cross_links`, `traces:events` | Links 0/0, events empty/garbage | `SELECT *` returns raw JSONB for Json columns over the arrow HTTP path; now projected via `json_to_string` | `1d04b29` |
| — | Waterfall | `traces:orphan` | Orphan indistinguishable from a true root | Added amber `detached` badge | `1d04b29` |
| D-006 | GraphQL panel | `protocols:graphql_errors` | Panel absent for single-span operations | Field attribution only walked ancestors; an op span that is its own field produced zero roots | `bac5eee` |
| D-007 | Logs | `logs:bodies` | Equal-timestamp rows shuffled between refreshes | `ORDER BY timestamp` alone; now tiebroken by body | `f5f2b9d` |
| D-008 | Logs, Issues | `logs:bodies` | Raw ANSI escape bytes in table cells, doc sheet, issue titles, and fingerprints | No ANSI stripping anywhere; added `stripAnsi` (UI display) and `fingerprint::strip_ansi` (grouping + titles) | `f5f2b9d`, `821b4fe` |
| D-009 | Logs | `logs:bodies` | 32 KiB body inlined whole into its table cell | No preview cap; now 512 chars + explicit char count, full body in the sheet | `f5f2b9d` |
| D-010 | Dashboards | `metrics:shapes` | Lines bridged missing metric buckets (gauge gap invisible) | Only observed buckets became rows; now null-filled at the bucket step + dots for isolated points | `ec21f8b` |
| D-011 | All badge surfaces | `issues:burst` | Identifiers title-cased (`Playground-Shapes`, `Main.Rs`) | shadcn Badge base carried `capitalize`; removed | `821b4fe` |
| — | Invocation hub | `journeys:happy_path` | External invocations stuck at `running`/`stale` after exit | status/outcome/endedAt now derive from the completed root `cli.command` span's outcome | `ad62530` |
| — | Journey | `journeys:outside_screen` | Between-screens error attributed to the previous screen | Journey errors used grouped issues' ms-truncated lastSeen; now per-occurrence `errorEvents` with ns timestamps | `ad62530` |
| D-013 | Journey | `journeys:error_path` | Error titles tripled (`x: x: x`) | UI prepended errorType to `issue_title`, which already leads with it | `ad62530` |
| — | Journey/actions | `journeys:error_path` | Widget attribution missing | `app.widget.name` now projected through UiAction → GraphQL → journey (`via checkout.submit.button`) | `ad62530` |
| D-014 | Ecosystem | `ecosystem:full` | Node cards overlapped unreadably in large columns | Fixed 420px canvas; now grows with the largest column | `1c7c519` |
| D-015 | Ecosystem | `ecosystem:full` | Quiet services' edges missing (CLI edge vanished) | Edges sampled from the 100 most-recent traces; now one whole-window self-join | `1c7c519` |

Corpus-side fixes (playground repo, same audit): `traces:clock_skew` could never trigger
the skew flag (same-service, 3 ms — now cross-service, 120 ms, `90d7d2b`);
stream spans carried no `rpc.system` so the RPC stream panel could not
classify them (`6265fef`); the `metrics:shapes` exemplar referenced a never-exported
trace, missed the exemplar surface, and its gauge gap sat below bucket
resolution (`4c2d1c0`); the web app replaced the default OTel resource and
lost `telemetry.sdk.language=webjs`, so Parallax classified the browser as a
plain service (`d9d4761`).

## Audit grid (final walk)

Legend: `pass` (all six checklist items) / `pass*` (with deferred cosmetic
note). Every cell re-verified after the last fix landed; the closing sweep
re-asserted `traces:deep`=14 spans, `traces:wide`=521, `traces:cross_links` bidirectional, `traces:events`=51,
7 service-map edges with all three kinds, 1 dead-letter job, 3 multi-lang
fingerprints — all against the live server.

### Traces × t-* / p-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Waterfall deep nesting | `traces:deep` | pass | `ui/audit/t-deep-after.png` |
| Waterfall wide + minimap | `traces:wide` | pass (521 spans, 24 virtualized DOM rows) | `ui/audit/t-wide.png` |
| Waterfall multi-root | `traces:multi_root` | pass (both roots depth 0) | `ui/audit/t-multiroot.png` |
| Waterfall orphan | `traces:orphan` | pass (detached badge + evidence gap) | `ui/audit/t-orphan.png` |
| Waterfall skew | `traces:clock_skew` | pass (120 ms banner, non-negative bars) | `ui/audit/t-skew.png` |
| Waterfall zero-duration | `traces:zero_duration` | pass (0µs bar 4.2px, no NaN) | `ui/audit/t-zero.png` |
| Links panel | `traces:cross_links` | pass (both directions navigable) | `ui/audit/t-links.png` |
| Long names inspector | `traces:long_names` | pass (truncate+tooltip, no page overflow) | `ui/audit/t-longnames.png` |
| Span events panel | `traces:events` | pass (51 events, preformatted stacks) | `ui/audit/t-events.png` |
| RPC status codes | `protocols:grpc_errors` | pass (rpc.grpc.status_code per attempt) | `ui/audit/p-grpc-err.png` |
| RPC stream panel | `protocols:grpc_stream` | pass (SENT/RECEIVED ordered, failure visible) | `ui/audit/p-grpc-stream.png` |
| GraphQL ops panel | `protocols:graphql_errors` | pass (field error distinct from request error) | `ui/audit/p-graphql-err.png` |
| Kafka lag + jobs | `protocols:rabbitmq_lag` | pass (producer-gap evidence, outcome=failure + job.id in inspector; unscoped jobs API shows 4 failed attempts) | `ui/audit/p-kafka-lag.png` |

### Logs × l-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Live tail + histogram burst | `logs:burst` | pass (15k histogram, tail picks up fresh emission) | `ui/audit/l-burst.png` |
| Bodies: JSON/32KiB/ANSI/blank/equal-ts | `logs:bodies` | pass (stable 0–4 order, ANSI stripped, `… (32,784 chars)`) | inline probes in transcript |

### Metrics × m-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Counter reset / gauge gap / exemplar | `metrics:shapes` | pass (rate non-negative, hard line break at gap, exemplar marker deep-links to the anchor trace) | `ui/audit/m-shapes-gap-after.png`, `ui/audit/m-shapes-exemplar.png` |

### Issues × e-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Grouped burst + type breakdown | `issues:burst` | pass* (300 events one issue, 6 distinct types; single-bucket trend bar stretches wide — cosmetic) | `ui/audit/e-burst.png` |
| Multi-language fingerprints | `issues:multi_language` | pass (3 fingerprints, language-appropriate titles, folded stacks with Caused-by) | `ui/audit/e-multi-lang.png` |

### Invocations / journey × j-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Journey happy narrative | `journeys:happy_path` | pass (chronological beats, every action links to its trace, status finished/success) | `ui/audit/j-happy.png` |
| Journey error attribution | `journeys:error_path` | pass (failed status; error on checkout via checkout.submit.button) | `ui/audit/j-error.png` |
| Journey outside bucket | `journeys:outside_screen` | pass ("outside any screen" between exit/enter) | `ui/audit/j-outside.png` |
| Sessions chain | `journeys:reattach` | pass (3 sessions, ↳ continuation, clickable) | `ui/audit/j-reattach.png` |
| Parallel isolation | `journeys:parallel` | pass (4 rows incl. daemon; per-hub sessions/traces isolated) | `ui/audit/j-parallel.png` |

### Ecosystem / overview

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Ecosystem graph kinds + edges | `ecosystem:full` | pass* (browser/cli/service icons, all cross-service edges incl. cli→checkout and web→checkout; edge labels can overlap node cards in dense layouts — cosmetic, labels stay readable on top) | `ui/audit/eco-full-final.png` |
| Overview charts | sweep | pass (KPIs, deltas, what-changed, spans/errors + latency charts) | `ui/audit/overview.png` |

## Generic-attributes conformance sweep (step 4)

`grep -rn "parallax\.\|jackin" ui/src crates/parallax-api/src`, classified:

- `ui/src/shared/semconv.ts` `PARALLAX_LAB`/`PARALLAX_SOURCE`/
  `TEST_CASE_ID`: generated generic export from the contract; unused by any UI
  component (no branching), kept as generated surface.
- `crates/parallax-*`: `parallax.lab` (comparison label stamped by
  Parallax's own forwarding CLI) and `parallax.source` (Parallax-stamped
  resource lane marker read alongside `service.namespace`) are
  Parallax-owned instrumentation declared in the contract with
  `owner: parallax` — not foreign application attributes.
- `ui/src/routes/sql.tsx` `parallax.sql.history`: localStorage key, not a
  telemetry attribute.
- `jackin` hits: one test fixture service-name string and one doc comment.

No component or resolver branches on an application-specific attribute name.
Opaque vendor-attribute display proven: a manual OTLP/protobuf post with
`custom.vendor.attr=opaque-value-42` and `acme.internal.flag=whatever`
renders both verbatim in the span inspector attribute table
(`ui/audit/vendor-attr-opaque.png`; command: python + opentelemetry-proto
posting to `http://127.0.0.1:4318/v1/traces`).

## Closure summary (step 5)

- Defects found: 15 numbered + 4 unnumbered product fixes + 4 corpus-side
  shape fixes. Fixed and re-verified: all non-cosmetic.
- Deferred cosmetic (with rationale): issue-trend single-bucket bar
  stretches to fill the chart (bucketing correct, display exaggerated);
  ecosystem edge labels can overlap node cards in dense layouts (labels
  render on top and stay readable — a full label-placement pass is a
  layout-research task, not a correctness fix).
- Structural note: two live-engine defect classes (subquery-to-subquery
  equi-join collapse; `IN (SELECT …)` semi-join returning zero rows) now
  have no remaining instances in the query layer — swept via
  `grep -rn "IN (SELECT\|JOIN (" crates/parallax-greptime` (only the
  single-table self-join and `ON TRUE` cross join remain, both verified
  correct on the live engine). Upstream consult packet is plan 159's
  evidence follow-up.
- Full gate set green at closure: `cargo fmt --check`, `cargo clippy
  --workspace --all-targets -D warnings`, `cargo nextest run --workspace`
  (305 passed), `cargo xtask policy` (0 violations), and in `ui/`:
  `typecheck`, `lint`, `check`, `test:ci` (218 passed / 48 files), `build`.
- Operator's span-rendering area explicitly closed: all 13 `t-*`/`p-*`
  trace cells pass.

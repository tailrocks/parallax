# Sub-traces and cross-trace correlation: OTel span links

Research date: 2026-06-12 (operator question: "can one span reference a
separate operation's trace, open it, and investigate the correlation —
does OpenTelemetry support that?").

## Answer: yes — span links, first-class in OTel

OpenTelemetry's tracing API has exactly this concept: **span links**. A
span carries zero or more links, each holding the SpanContext of another
span — its trace id and span id — plus optional attributes. The target can
live in the same trace or a completely separate one. A link implies causal
relation without a parent/child edge.

Why links instead of parent/child for "sub-operations":

- **A span has exactly one parent.** When one operation relates to many
  source operations (a batch job processing requests that each arrived in
  their own trace), parent/child can represent at most one of them; links
  represent all of them.
- **Separate operation = separate trace** is the OTel-recommended shape
  for async/batch boundaries: the messaging semantic conventions use links
  (not parent/child) as the default producer↔consumer correlation, and the
  spec's link API exists precisely for "associate spans across traces".
- Links are set **at span creation** (the API contract); the SDKs expose
  `add_link` for after-the-fact additions where supported.

So the operator's "from one span, link to another trace, open it,
investigate" is not a custom invention — it is the standard mechanism, and
any OTel SDK can emit it (Rust: `tracing-opentelemetry` exposes
`span.add_link(SpanContext)`; the raw API takes links in the span builder).

## What Parallax does with them (implemented 2026-06-12)

- **Storage:** span links are read from GreptimeDB's native `opentelemetry_traces.span_links`
  (`JSON`), which the native trace model already stores — no custom column needed. *(History: the
  earlier hand-rolled design added a `links` JSON column to `otel_spans`; superseded by the native-OTLP
  decision, [native-otel-tables.md](../decisions/native-otel-tables.md).)*
- **API:** `Span.links` (JSON string) on the GraphQL `trace(traceId:)`
  read.
- **UI:** the trace waterfall marks spans carrying links with an
  `↗ N linked` badge; the span detail pane lists **Linked traces** as
  clickable trace ids — one click jumps into the linked operation's own
  waterfall, which is the investigation flow the operator described.
- **Demo emitter:** `cargo run --example seed_links -p parallax-server`
  seeds a source trace and a batch trace whose span links back to it.

Verified live: the seeded batch span stored
`[{"traceId":"a0a1…","spanId":"b0b1…","attributes":{"link.kind":"batch-source"}}]`,
the trace page showed the badge and the Linked traces entry, and the link
navigated to the source trace.

## Follow-ups (not in this slice)

- **Reverse direction** ("which traces link TO this one?") needs an index
  or scan over `links` — natural once linked traffic exists; GreptimeDB
  JSON functions or a promoted link table both work.
- **jackin'/SDK guidance:** when jackin' (or any integrated tool) spawns a
  logically separate operation, it should start a new trace and attach a
  span link to the spawning span — document in the integration guide when
  the first real emitter needs it.

## Sources

- OTel spec, Tracing API — links semantics (creation-time, SpanContext +
  attributes, same-or-different trace):
  <https://opentelemetry.io/docs/specs/otel/trace/api/>
- OTel docs, "Creating links between traces":
  <https://opentelemetry.io/docs/languages/dotnet/traces/links-creation/>
- OTel concepts, traces/spans (links overview):
  <https://opentelemetry.io/docs/concepts/signals/traces/>
- SigNoz, span links for async/batch correlation:
  <https://signoz.io/docs/traces-management/guides/span-links/>
- OneUptime, span links patterns (batch fan-in, messaging defaults):
  <https://oneuptime.com/blog/post/2026-01-07-opentelemetry-span-links/view>

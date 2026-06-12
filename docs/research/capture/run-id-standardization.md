# Run ID vs the OTel Standards: What the Industry Calls It

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-12. Question (operator): we correlate everything a CLI
invocation produced — logs, traces (plural — one run spawns many), metrics —
under a `run_id`. Does OpenTelemetry have a standard for this concept, and
should Parallax and jackin' adopt it instead of the custom
`parallax.run_id` / `jackin.run_id` attributes?

## Conclusion first

**There is no OpenTelemetry standard for a CLI run id.** The CLI semantic
conventions exist but define no invocation-correlation identifier at all.
The closest concepts are `session.id` (client-app sessions; Development
stability) and `cicd.pipeline.run.id` (CI pipeline runs; Development
stability — the one place OTel uses the literal words "run id"). The only
*stable* identifier in the area, `service.instance.id`, has the wrong
granularity (per process — a wrapped `cargo test` run spans many processes).

**Decision (2026-06-12):** keep `parallax.run_id` as the canonical key — a
vendor-namespaced custom attribute is exactly what the OTel naming
guidelines prescribe when no convention covers the concept. Meet the
standards halfway on both sides of the wire:

1. **Accept aliases at ingest** — the normalizer resolves the run id as
   `parallax.run_id` → `session.id` → `cicd.pipeline.run.id` (first present
   wins). An OTel-conventional emitter (a mobile-style session, a CI task)
   correlates in Parallax with zero Parallax-specific wiring.
2. **Dual-emit on the wrapper** — `parallax run start` injects
   `OTEL_RESOURCE_ATTRIBUTES=parallax.run_id=<id>,session.id=<id>`, so any
   *other* OTel backend the user points the same app at groups the run by
   its session-id support.
3. **Recommendation for jackin'**: same pattern — keep `jackin.run_id` +
   `parallax.run_id`, add `session.id=<run id>` to the OTLP resource.

Revisit-trigger: the OTel CLI or CICD SIGs adding an invocation/run
identifier to the CLI conventions — adopt it as another (eventually the
canonical) alias the moment it exists.

## What was checked (primary sources)

| Candidate | Where | Stability | Verdict for "CLI run id" |
| --- | --- | --- | --- |
| CLI semconv (`cli/cli-spans`) | [Semantic conventions for CLI programs](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | Development | Defines span name `{process.executable.name}`, `process.exit.code`/`process.pid`/`process.command_args` — **explicitly no run/invocation/session id and no cross-trace correlation mechanism**. The gap is real, not an oversight of ours. |
| `session.id` / `session.previous_id` | [Session semantic conventions](https://opentelemetry.io/docs/specs/semconv/general/session/) | Development, opt-in | "The period of time encompassing all activities performed by the application and the actions executed by the end user" — a collection of logs, events, and spans under one id. Semantically the closest match to a run, but scoped to *client-side applications* (mobile/browser) and attached per-signal, not as a resource. Good alias, weak canonical. |
| `cicd.pipeline.run.id` / `cicd.pipeline.task.run.id` | [CICD resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/cicd/) | Development | "The unique identifier of a pipeline run within a CI/CD system" — the literal industry "run id", with an explicit high-cardinality opt-in warning. A local interactive CLI invocation is not a CI pipeline; borrowing the namespace would misstate the semantics. Good alias for actual CI emitters. |
| `service.instance.id` | [Service resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/service/) | **Stable** | "The string ID of the service instance" (UUID recommended). Identifies one process/instance — but a Parallax run wraps child processes (`cargo test` spawns many), so instance ≠ run. Wrong granularity; not an alias. |
| Custom-attribute naming rules | [Naming](https://opentelemetry.io/docs/specs/semconv/general/naming/), [How to name your span attributes](https://opentelemetry.io/blog/2025/how-to-name-your-span-attributes/) | guidance | Company/product-prefixed names (`parallax.*`) are the prescribed pattern for concepts no convention covers; never squat on existing semconv namespaces. `parallax.run_id` is conformant as-is. |

Mechanism note: resource attributes (what the wrapper env-injects) are the
OTel-native carrier for process-lifetime constants and flow to every signal
(traces, logs, metrics) automatically — which is exactly why one run id
correlates the run's *multiple traces*. W3C Baggage is the alternative for
request-flow propagation; irrelevant for the run case.

Adjacent industry usage of the word "run": ML experiment trackers (MLflow
`run_id`, W&B runs) — same concept, no OTel bridge; reinforces that "run"
is the right product word.

## What changed in code (2026-06-12)

- `parallax-core/normalize.rs`: run-id resolution accepts the alias chain on
  spans, logs, and metric points.
- `parallax-cli` wrapper + bare mode: dual-emits `session.id` beside
  `parallax.run_id`.
- Spec §7 mapping row records the aliases and the precedence.
- Guide `conventions.md` documents the aliases for integrators.

Sources: [CLI spans semconv](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) ·
[Session semconv](https://opentelemetry.io/docs/specs/semconv/general/session/) ·
[CICD resource semconv](https://opentelemetry.io/docs/specs/semconv/resource/cicd/) ·
[Service resource semconv](https://opentelemetry.io/docs/specs/semconv/resource/service/) ·
[Attribute naming](https://opentelemetry.io/docs/specs/semconv/general/naming/) ·
[OTel blog: how to name your span attributes](https://opentelemetry.io/blog/2025/how-to-name-your-span-attributes/) ·
[CNCF: OTel expanding into CI/CD observability](https://www.cncf.io/blog/2024/11/04/opentelemetry-is-expanding-into-ci-cd-observability/)

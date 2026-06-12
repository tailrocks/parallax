# Run ID Standardization — Position, Upstream Proposal, and Tracking

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-12 (standing page — update on every upstream movement
or internal migration step). Owner question (operator): one CLI invocation
("a run") produces many traces, logs, and metrics; we correlate them under a
run id. What is the standard, and how do we get to one?

## Position (operator, 2026-06-12)

1. **There is no OTel standard for this concept today**, and `session.id`
   **is not what we need** — it is a client-side-application convention
   (mobile/browser user sessions, Development stability) that we accept and
   emit only as an **interop bridge**, not as the answer.
2. **We want a real standard and intend to help make one.** Parallax will
   bring its run concept to the OpenTelemetry semantic-conventions
   discussion as a proposal and participate in the threads where the gap is
   already being felt (see *Upstream proposal* and *Tracking* below).
3. **Internal standardization comes first.** The ladder:

   | Step | What | Status |
   | --- | --- | --- |
   | 1 | `jackin.run_id` migrates to **`parallax.run.id`** as the primary key on jackin's OTLP resource (keep `jackin.run_id` as a legacy alias during the transition) | recommended to jackin', 2026-06-12 |
   | 2 | `parallax.run.id` is the one canonical run key across Tailrocks tools — vendor-namespaced exactly as the [OTel naming guidance](https://opentelemetry.io/docs/specs/semconv/general/naming/) prescribes for concepts no convention covers | **current state** |
   | 3 | When an OTel standard exists (ours or someone else's), it becomes an accepted ingest alias immediately, then the canonical key once it reaches stability — `parallax.run.id` demotes to the legacy alias | future, tracked here |

## Why the existing conventions don't fit (primary sources, checked 2026-06-12)

| Candidate | Verdict |
| --- | --- |
| [CLI semconv](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) (Development) | Defines span name `{process.executable.name}`, `process.exit.code`, `process.pid`, `process.command_args` — **no invocation/correlation id at all and no mechanism to tie one CLI execution's traces together**. This is exactly the gap; it is where the fix belongs. |
| [`session.id`](https://opentelemetry.io/docs/specs/semconv/general/session/) (Development, opt-in) | Semantically the closest: "the period of time encompassing all activities performed by the application and the actions executed by the end user", a collection of logs/events/spans under one id across traces. But scoped to **client-side applications**; a CLI run is not a user session. Interop bridge, not the standard we need. |
| [`cicd.pipeline.run.id`](https://opentelemetry.io/docs/specs/semconv/resource/cicd/) (Development) | The literal "run id" words in semconv — for CI/CD systems. A local interactive invocation is not a pipeline run; borrowing the namespace would misstate semantics. Alias for genuine CI emitters. |
| [`service.instance.id`](https://opentelemetry.io/docs/specs/semconv/resource/service/) (**stable**) | Per process-instance. A wrapped run (`parallax run start -- cargo test`) spans many processes under one run id — wrong granularity. |

Mechanism: resource attributes are the OTel-native carrier for
run-lifetime constants (they flow to every signal, which is why one id
correlates many traces); W3C Baggage covers request flows, not runs.
Adjacent industry: ML experiment trackers (MLflow `run_id`, W&B runs) use
the same word for the same shape — no OTel bridge exists there either.

## Upstream proposal (draft to bring to OTel)

**Thesis:** bounded executions that produce telemetry across multiple traces
need a first-class correlation id. Two acceptable shapes, in preference
order:

1. **Generalize the Session conventions** beyond client-side applications:
   redefine a session as *a bounded period of activity by one actor*
   (end-user app session, GenAI agent session, **CLI invocation**), keep
   `session.id`/`session.previous_id` as-is, and add a CLI note to the CLI
   semconv ("a CLI program SHOULD stamp `session.id` on its resource for
   the lifetime of one invocation, including child processes"). Precedent
   already moving this way: the GenAI SIG asks for exactly this
   generalization in
   [semantic-conventions#2883](https://github.com/open-telemetry/semantic-conventions/issues/2883)
   ("sessions are a generic concept… across all computing contexts, not
   just browsers"; hierarchy `session.id` > `gen_ai.conversation.id`).
   Our CLI-run case is the second independent demand for the same change —
   that convergence is the argument.
2. **A dedicated attribute in the CLI namespace** (`cli.run.id` or
   `cli.invocation.id`) if the Session owners insist sessions stay
   user-centric: same semantics (resource-level, spans child processes,
   one id per invocation), narrower blast radius.

What Parallax brings to the table: a shipping reference implementation
(resource-attribute injection by a wrapper, child-process inheritance via
`OTEL_RESOURCE_ATTRIBUTES`, column promotion for exact run-scoped reads,
run-anchored evidence bundles), plus jackin' as a second real CLI emitter.

## Tracking (update this table as threads move)

| Thread | Why it matters | State (2026-06-12) | Our move |
| --- | --- | --- | --- |
| [semantic-conventions#2883 — Add session.id to GenAI conventions](https://github.com/open-telemetry/semantic-conventions/issues/2883) | The live generalize-`session.id` push; our strongest ally thread | Open since 2025-10-07, triage "Needs Info", no owner, no PR | Comment with the CLI-run use case + offer Parallax/jackin' as implementations |
| [CLI semconv](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) ([docs/cli in the semconv repo](https://github.com/open-telemetry/semantic-conventions/tree/main/docs)) | Where a CLI run id would land | Development; no correlation id | Open a dedicated issue: "CLI invocations need a cross-trace correlation id" referencing #2883 and this page |
| [CICD conventions](https://opentelemetry.io/docs/specs/semconv/resource/cicd/) (heritage: [oteps#223](https://github.com/open-telemetry/oteps/pull/223), [CNCF announcement](https://www.cncf.io/blog/2024/11/04/opentelemetry-is-expanding-into-ci-cd-observability/)) | Owns `*.run.id` naming; would review any general "run" attribute | `cicd.pipeline.run.id` Development | Watch for stabilization; cite as naming precedent in the proposal |
| [Session conventions](https://opentelemetry.io/docs/specs/semconv/general/session/) | The text our preferred option amends | Development, client-side scoped | Track wording changes; a scope broadening = adopt-as-alias trigger |

Engagement order: (1) comment on #2883, (2) dedicated semconv issue for the
CLI case, (3) if traction, a PR amending the session/CLI docs with the
wording above. Every step gets a dated row appended here.

## Current implementation state (Parallax, 2026-06-12)

- **Ingest aliases** (`parallax-core/normalize.rs`): run id resolves
  `parallax.run.id` → `session.id` → `cicd.pipeline.run.id`, first present
  wins, on spans, logs, and metric points (spec §7).
- **Wrapper dual-emit**: `parallax run start` injects
  `OTEL_RESOURCE_ATTRIBUTES=parallax.run.id=<id>,session.id=<id>` — other
  OTel backends correlate the same run today.
- **Guide**: [conventions.md](../../guide/conventions.md) documents the
  aliases for integrators.
- **jackin'**: recommended to adopt `parallax.run.id` as primary (step 1 of
  the ladder) and emit `session.id` alongside.

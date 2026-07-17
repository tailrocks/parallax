# Run ID Standardization — Position, Upstream Proposal, and Tracking

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-12; updated 2026-06-15; **re-verified 2026-07-17 against
source**; **upstream tracker recheck pass 53 + pass 92 + pass 127 + pass 163 +
pass 207 + pass 252 (2026-07-18)**.
Owner question
(operator): one CLI invocation ("a run") produces many traces, logs, and
metrics; we correlate them under an invocation id. What is the standard, and
how do we get to one?

> **Current authority (operator, 2026-07-17; verified in source
> 2026-07-17): Parallax correlates CLI invocations on `cli.invocation.id`
>  (+ `session.id`), not `parallax.run.id`.** The vendor-namespaced
> `parallax.run.id` key is **retired** — it is never read, written, or
> COALESCE'd at ingest (negative contract: `crates/parallax-ingest/src/tests.rs`,
> `crates/parallax-server/tests/m8_invocation_contract_greptime.rs`). This is
> the internal realization of *Upstream proposal* option 2 below (a dedicated
> CLI-namespace attribute). The `runs`/`run` GraphQL surface and CLI verbs
> were renamed to `invocations`/`invocation` (`invocationStart`/`invocationFinish`).
> Program: plans 156–161; decision:
> [decisions/native-otel-tables.md](../decisions/native-otel-tables.md);
> generated constants in [parallax-semconv](../../../crates/parallax-semconv/) from
> [`telemetry/semconv/contract.yaml`](../../../telemetry/semconv/contract.yaml).

## Position (operator, 2026-07-17; re-verified in source)

1. **There is still no OTel standard for a CLI invocation correlation id**,
   and `session.id` alone is not sufficient — it is a client-side-application
   convention (mobile/browser user sessions, Development stability), not a
   local CLI invocation boundary. Parallax now emits **both** `cli.invocation.id`
   and `session.id` so a future generalized-session or CLI-invocation standard
   can adopt either (see *Tracking*; historical #2883 link is dead).
2. **We want a real standard and intend to help make one.** Parallax will
   bring its invocation concept to the OpenTelemetry semantic-conventions
   discussion as a proposal and participate in the threads where the gap is
   already being felt (see *Upstream proposal* and *Tracking* below).
3. **Internal standardization is generic-attribute-only.** Parallax resolves
   an invocation id from **`cli.invocation.id`** — signal attribute first
   (root-span / log attrs, the jackin shape), then resource attribute — and
   does not accept `cicd.pipeline.run.id` or tool-specific ids as aliases. If
   `cli.invocation.id` is absent, telemetry is not invocation-scoped. The
   retired `parallax.run.id` is explicitly **not** a fallback.

Historical ladder:

   | Step | What | Status |
   | --- | --- | --- |
   | 1 | `jackin.run_id` migrates to **`parallax.run.id`** as the only Parallax-facing OTLP resource key | superseded 2026-07-17 |
   | 2 | `parallax.run.id` is the canonical run key across Tailrocks tools | **superseded 2026-07-17** by `cli.invocation.id` (plans 156–161) |
   | 3 | Adopt a generic OTel attribute; `parallax.run.id` demotes to a legacy alias | **realized 2026-07-17 as `cli.invocation.id`** — no legacy alias kept (forward-only cutover); `parallax.run.id` is dropped outright, not demoted |
   | 4 | When an OTel standard reaches stability, adopt it as alias then canonical | future, tracked here |

## Why the existing conventions don't fit (primary sources, checked 2026-06-12)

| Candidate | Verdict |
| --- | --- |
| [CLI semconv](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) (Development) | Defines span name `{process.executable.name}`, `process.exit.code`, `process.pid`, `process.command_args` — **no invocation/correlation id at all and no mechanism to tie one CLI execution's traces together**. This is exactly the gap; it is where the fix belongs. |
| [`session.id`](https://opentelemetry.io/docs/specs/semconv/general/session/) (Development, opt-in) | Semantically adjacent: a bounded collection of logs/events/spans under one id. But it is scoped to **client-side applications** and user sessions; a CLI run is not a user session. Not accepted as a Parallax run alias. |
| [`cicd.pipeline.run.id`](https://opentelemetry.io/docs/specs/semconv/resource/cicd/) (Development) | The literal "run id" words in semconv — for CI/CD systems. A local interactive invocation is not a pipeline run; borrowing the namespace would misstate semantics. Not accepted as a Parallax run alias. |
| [`service.instance.id`](https://opentelemetry.io/docs/specs/semconv/resource/service/) (**stable**) | Per process-instance. A wrapped run (`parallax run start -- cargo test`) spans many processes under one run id — wrong granularity. |

Mechanism: resource attributes are the OTel-native carrier for
run-lifetime constants (they flow to every signal, which is why one id
correlates many traces); W3C Baggage covers request flows, not runs.
Adjacent industry: ML experiment trackers (MLflow `run_id`, W&B runs) use
the same word for the same shape — no OTel bridge exists there either.

## Upstream proposal (draft to bring to OTel)

> **Status (2026-07-17): option 2 below was realized internally as
> `cli.invocation.id`.** The proposal to the OTel community stands unchanged —
> Parallax now ships the reference implementation that proves option 2, plus
> `session.id` for the option-1 generalization case.

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
   that convergence is the argument. Parallax will not emit `session.id`
   before that semantic broadening lands.
2. **A dedicated attribute in the CLI namespace** (`cli.run.id` or
   `cli.invocation.id`) if the Session owners insist sessions stay
   user-centric: same semantics (resource-level, spans child processes,
   one id per invocation), narrower blast radius.

What Parallax brings to the table: a shipping reference implementation
(resource-attribute injection by a wrapper, child-process inheritance via
`OTEL_RESOURCE_ATTRIBUTES`, column promotion for exact run-scoped reads,
run-anchored evidence bundles), plus jackin' as a second real CLI emitter.

## Tracking (update this table as threads move)

| Thread | Why it matters | State (**2026-07-18 pass 163**; prior 53/92/127) | Our move |
| --- | --- | --- | --- |
| ~~`open-telemetry/semantic-conventions#2883`~~ | Historical citation for "generalize session.id" | **Pass 127:** GitHub **redirects** this number to **genai#51**. Not a CLI-invocation issue. Do not cite as a separate live thread. | Drop; use GenAI issue below |
| [semantic-conventions-genai#51 — Add session.id to GenAI conventions](https://github.com/open-telemetry/semantic-conventions-genai/issues/51) | Live GenAI `session.id` reuse push (multi-step/multi-agent workflows) | **Still open** (pass 163: `updated_at` still **2026-05-05**; label `enhancement`). GenAI-scoped, **not** CLI-invocation. | Comment only if framing expands beyond GenAI |
| [CLI semconv](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) ([model/cli/spans.yaml](https://github.com/open-telemetry/semantic-conventions/blob/main/model/cli/spans.yaml)) | Where a CLI invocation correlation id would land | **Pass 163:** `model/cli/` still only **`spans.yaml`**; code search `cli.invocation` in semconv repo **total_count 0** — **no** OTel `cli.invocation.id` | **Primary move:** open dedicated semconv issue citing Parallax/`jackin` shipping reference |
| [CICD conventions](https://opentelemetry.io/docs/specs/semconv/resource/cicd/) | Owns `*.run.id` naming | `cicd.pipeline.run.id` still Development (not re-diffed this pass) | Watch; naming precedent only — do not alias pipeline run to CLI invocation |
| [Session conventions / registry](https://opentelemetry.io/docs/specs/semconv/general/session/) | Scope of `session.id` | Still **development** / client application + end user wording (not re-diffed body this pass) | Scope broadening = adopt-as-alias trigger; **not fired** |

**Pass-53 + pass-92 + pass-127 + pass-163 verdict:** still **no OTel standard** for a
CLI invocation correlation id. GenAI#51 idle since 2026-05. CLI model still process
attrs only. Parallax's `cli.invocation.id` remains a justified generic-CLI
choice, not a standard. Upstream engagement order unchanged: (1) **dedicated
CLI semconv issue**, (2) optional GenAI#51 comment, (3) PR if SIG interest.

### Pass log (upstream)

| Date | Check | Result |
| --- | --- | --- |
| 2026-06-12 | Initial tracker table | Cited #2883 as GenAI session push |
| 2026-07-17 | Internal cutover to `cli.invocation.id` | Documented above |
| 2026-07-17 pass 53 | Re-fetch #2883 + CLI docs + session registry | **#2883 link broken**; GenAI work lives at **genai#51**; CLI still lacks invocation id; session still client-scoped |
| 2026-07-17 pass 92 | Re-fetch genai#51 + CLI spans + session registry | genai#51 **still open**, last activity **2026-05-05**; CLI/session **no drift** |
| 2026-07-17 pass 127 | API re-fetch genai#51 + #2883 + model/cli | genai#51 **still open/idle** (2026-05-05); #2883 **redirects to genai#51**; CLI spans.yaml **unchanged** (no invocation id) |
| 2026-07-18 pass 163 | API re-fetch genai#51 + model/cli listing + code search | genai#51 **still open/idle** (`updated_at` 2026-05-05); `model/cli/` = `spans.yaml` only; `cli.invocation` search **0 hits**; product still ships `CLI_INVOCATION_ID = "cli.invocation.id"` |
| 2026-07-18 pass 207 | API re-fetch genai#51 + model/cli + code search | genai#51 **still open/idle** (2026-05-05); CLI model **spans.yaml** only; `cli.invocation` **0 hits** — still no OTel standard |
| 2026-07-18 pass 252 | API re-fetch genai#51 + model/cli + code search | genai#51 **still open/idle** (`updated_at` **2026-05-05**); `model/cli/` still **only `spans.yaml`** (process attrs; **no** invocation id); `cli.invocation` code search **total_count 0** — **still no OTel standard** |
| 2026-07-18 pass 278 | API re-fetch genai#51 + model/cli + code search | genai#51 **still open/idle** (2026-05-05); CLI **spans.yaml** only; `cli.invocation` **0** — **still no OTel standard** |
| 2026-07-18 pass 293 | API re-fetch genai#51 + code search | genai#51 **still open/idle** (2026-05-05); `cli.invocation` **0** — **still no OTel standard** |
| 2026-07-18 pass 311 | API re-fetch genai#51 + model/cli/spans.yaml | genai#51 **still open/idle** (`updated_at` **2026-05-05**); `model/cli/` still **only `spans.yaml`** (process attrs only; **no** invocation id); authenticated code search unavailable this pass — raw spans.yaml has **0** `invocation` hits — **still no OTel standard** |

## Current implementation state (Parallax, 2026-07-17; re-verified in source)

- **Canonical constant** (`crates/parallax-semconv/src/lib.rs`):
  `CLI_INVOCATION_ID = "cli.invocation.id"` (generated from
  `telemetry/semconv/contract.yaml`); `SESSION_ID` likewise.
- **Ingest** (`crates/parallax-ingest/src/lib.rs`): `invocation_id` resolves
  the signal attribute first (root-span / log attrs — jackin shape), then the
  resource attribute, for traces and logs. The retired `parallax.run.id` is
  never read (negative contract in `crates/parallax-ingest/src/tests.rs` and
  `crates/parallax-server/tests/m8_invocation_contract_greptime.rs`).
- **Wrapper emit** (`crates/parallax-cli/src/commands/forwarding.rs`):
  `forward_resource_attrs` stamps `cli.invocation.id` (and `session.id`) into
  `OTEL_RESOURCE_ATTRIBUTES` for the wrapped child.
- **Storage reads** (`crates/parallax-storage/src/adapter/traits.rs`):
  `traces_by_invocation`, `logs_by_invocation`, and invocation-scoped metric
  points key solely on `cli.invocation.id`.
- **GraphQL surface** (`crates/parallax-api/src/lib.rs`): `runs`/`run` were
  renamed to `invocations`/`invocation` (`invocationStart`/`invocationFinish`,
  plus `tracesByInvocation`, `logsByInvocation`, `invocationMetrics`).
- **Guide**: [conventions.md](../../guide/conventions.md) documents the
  generic-attribute rule for integrators.
- **jackin'**: adopts `cli.invocation.id` + `session.id` (the unified-CLI
  observability program, plans 156–161).

# Problem, Audience, and Product Shape

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-11. Operator vision statement #2 recorded 2026-06-11. This is the
front-door answer to three questions every other note assumes: **what problem Parallax solves,
who it is for, and what shape the product takes.** The operator's standing instruction is to keep
this framing sharp as the vision evolves — when a pass changes the answer to any of the three
questions, this file is updated in the same change.

> **One paragraph.** Parallax is for developers — human and AI — who can now build and ship
> software fast but lose all of that speed the moment something breaks at runtime. It combines
> the best concept from three worlds — **OpenTelemetry** (how data is collected, as an open
> standard), **Sentry** (how collected failures are organized into grouped, workflow-ready
> issues), and **Grafana** (how humans see across signals to understand what is going on) — into
> one open-source Rust engine that is **agent-first**: the same evidence that renders in a UI for
> a human is served as bounded, redacted, citable bundles to an AI that proposes the fix. It
> starts on one developer's laptop and scales, by topology change rather than rewrite, to
> companies that fix this month's bugs next quarter.

## 1. The problem

In the post-AI development world, the speed profile of software work has inverted:

| Activity | Speed today | Why |
| --- | --- | --- |
| Writing code | Fast | Coding agents |
| Deploying | Fast | Modern CI/CD |
| **Diagnosing and fixing runtime failures** | **Slow** | Evidence is fragmented across Sentry, Grafana, Kibana, Jaeger, CI, and deploy systems, and none of it is shaped for an agent to consume |

The bottleneck is not model capability; it is **context**. An AI that could fix the bug cannot
see the panic, the logs around it, the trace it sat in, the metric window, and the deploy that
introduced it — as one connected, trustworthy picture. Humans page through five tools to build
that picture by hand ([world-before-parallax.md](world-before-parallax.md)); agents mostly are not
given it at all. The end state this serves is the
[autonomous fix loop](north-star-autonomous-fix-loop.md).

## 2. The concept: best of three worlds, agent-first

| World | What it got right | What Parallax takes |
| --- | --- | --- |
| **OpenTelemetry** | One vendor-neutral standard for emitting traces, logs, metrics | The only collection path: standard OTel SDKs + resource conventions, no proprietary SDK ([integration-contract.md](../architecture/integration-contract.md)) |
| **Sentry** | Errors become grouped, deduplicated, workflow-ready *issues*, not log lines | Deterministic fingerprinting, issue model, release/deploy linkage — derived from OTLP, Sentry-protocol ingest only as a future adapter |
| **Grafana** | Humans understand systems by looking *across* signals | The cross-signal investigation UI (plus Kibana's object-centric log view, Tempo's waterfall) — [simple-ui-v2.md](../architecture/simple-ui-v2.md) |

The difference from all three: Parallax is designed **for AI first**. Every view a human gets is
a projection of the same evidence graph an agent receives as a bounded bundle. The platform's job
is to give the agent the full picture so it can make reasonable decisions about fixes — and give
people the UI to see the same truth.

## 3. Product shape: three surfaces, one API

```text
                 ┌──────────────  Parallax server (Rust) ─────────────┐
  OTel SDKs ───► │ ingest → derive → group → correlate → evidence API │
  deploy events ─►└───────────────────────┬───────────────────────────┘
                                          │ one canonical API (GraphQL query + OTLP ingest)
              ┌───────────────┬───────────┴────────────┬──────────────┐
            CLI             HTTP API                  UI            MCP (read-only, gated)
        (kubectl model)   (canonical)          (human window)     (agent transport)
```

- **API** is canonical. Everything else is a client. No surface ever touches storage directly
  ([api-concept.md](../architecture/api-concept.md), [agent-access-surface.md](../decisions/agent-access-surface.md)).
- **CLI follows the kubectl model.** The server runs anywhere — laptop, VM, cluster — and the CLI
  connects to it by context, exactly like `kubectl` against a cluster: `parallax --context local
  run inspect`, `parallax --context prod issue list`. A coding agent on a desktop drives the same
  CLI against a production server and gets the whole picture remotely. Universal: it does not
  matter where the server is deployed or which storage backend it runs.
- **UI** is the human window over the same API — Sentry-style issues plus Grafana/Kibana-style
  cross-signal inspection. Humans need to know what is going on too; the UI is how.
- **MCP** is a fourth, read-only projection for agents, after safety gates.

One API + swappable `StorageAdapter` is what makes the scale ladder below a topology change
rather than a rewrite.

## 4. The audience ladder (priority order)

**Rung 1 — the developer on a dev machine (first priority).** "I am developing a tool locally; I
want to point my app at something and say *send all your data here*." One command starts a local
Parallax; a Rust backend connects via standard OTel env vars; the developer (or their coding
agent) inspects the run, sees the panic with its logs/trace/metrics, and finds the bug they just
introduced. This is [local-first-v1.md](../architecture/local-first-v1.md), and it is the wedge:
the operator is user #1.

**Rung 2 — the team with a deployed server.** Same binary deployed remotely; developers connect
from their desktops with the kubectl-style CLI; coding agents connect through CLI/API/MCP and
propose fixes — Rust panics first, but any OTel-speaking app can send: OTLP ingest is
language-agnostic by standard, so Java exceptions, Go errors, or browser events arrive the same
way. (Capture *depth* stays Rust-first per scope; the engine and infra stay Rust. Telemetry
**sources** are polyglot by design — same clarification as the frontend-as-source rule.)

**Rung 2, sharpened (operator statement #4): one binary, deployment profiles.** The old world
made a growing startup assemble and operate Sentry + Grafana + Loki + a trace store + a metrics
stack — and "they always never do this properly." Parallax replaces that with **one binary plus a
profile**: `--profile local` for the laptop, `--profile cloud` for the server — where the cloud
profile adapts to the environment (which cloud, object-storage-backed retention, cloud-suited
defaults) instead of making the operator design a deployment. You are not "managing Sentry"; you
are running one thing that was designed to run in the cloud from the start.

**Rung 3 — the big company.** Bugs are not fixed the day they fire; they are fixed next month or
next quarter. That makes **retention economics** the product feature: object storage as the only
copy, hot/cold tiering, pre-aggregation, evidence pinning so bundle-cited raw slices outlive TTL
([north-star §4](north-star-autonomous-fix-loop.md)). Storage scales horizontally on the
GreptimeDB production profile (ClickHouse fallback behind the adapter); ingest/workers scale as
Tier 2/3 topology ([implementation-concept.md](../architecture/implementation-concept.md)).
Local-first is not a toy tier: if GreptimeDB serves everything from laptop to cluster, the same
engine runs the whole ladder — which is exactly the current lean
([storage-engine.md](../decisions/storage-engine.md)).

**Priority (operator statement #4): rungs 1 and 2 are the first-priority audiences.** The local
developer and the deployed startup are who Parallax must win first. Rung 3 is not a focus to
build *for* yet — it is a constraint to design *under*: horizontal scalability and cheap
infrastructure are baked in from the start so the same product carries a team from small startup
to the largest company by topology change, never by rewrite.

## 5. Lifecycles: how the priority audiences actually use Parallax

Operator statements #3 and #4 (2026-06-11) gave the concrete usage stories for the two priority
audiences. They are the product's first reality, so they are recorded here
verbatim-in-substance:

**Lifecycle 1 — the feature-development loop.** The developer runs Parallax locally as the
telemetry server; their application sends logs, metrics, and traces over OTLP while they build.
When a Rust test fails or behaves oddly, the coding agent pulls the run's telemetry and usually
fixes it unaided — agents are smart enough once they can see the runtime facts. The Sentry
protocol enters only where OTLP genuinely cannot express something. The operator's own example is
the right one: **breadcrumbs**. OpenTelemetry has no first-class breadcrumb signal — an
interaction trail can be approximated with log records (and span events are deprecated toward
log-based events), but Sentry's breadcrumb model (typed, categorized, auto-collected by SDKs) is
richer today. That is exactly the gap-filler criterion that keeps the future Sentry adapter on
the roadmap ([capture/sentry-ingest.md](../capture/sentry-ingest.md)) without ever making it the
primary path.

**Lifecycle 2 — the QA verification loop.** A human (developer-as-QA or QA engineer) exercises a
finished feature — clicking through it, driving a TUI/CLI — and reports what feels wrong in
*behavioral* terms: "I did this, and that's not right; fix it." The agent translates the
complaint into evidence by pulling the interaction's runs, traces, and logs from Parallax instead
of interrogating the human for details they cannot articulate.

**Lifecycle 3 — the reproduce-and-instrument loop.** From the operator's own tool, Jackin (runs
multiple coding agents in isolated Docker containers with its own terminal multiplexer): the
multiplexer sometimes renders a dirty screen state. A bug like that is *hard to describe* to an
agent but *easy to point at*: "I just saw it — check this `run_id`/`trace_id`." The agent pulls
the debug information through CLI/MCP and diagnoses immediately — **if** the app followed good
observability practice. When it didn't, the loop has a second gear that turns the bundle's
`missing_evidence` report into action: the agent sees what instrumentation was absent, **adds the
tracing/debug logging itself**, asks the human to reproduce the glitch once more ("just tell me
when you see it"), and then extracts the now-complete evidence. Observe → notice the gap →
instrument → reproduce → diagnose. This kills the most expensive loop in agent-assisted
development — the blind "no, you still didn't fix it" iteration — because the agent stops
guessing and starts measuring. It is also why `missing_evidence` is a load-bearing schema field,
not a politeness: it is the machine-readable signal that drives gap-closing.

**Lifecycle 4 — the trace-ID complaint loop (rung 2).** The startup has deployed its
microservices with Parallax as the ecosystem's observability source of truth; everything emits
OTLP exactly as in the local workflow. A user hits an error on the website. The application
**surfaces the trace ID to the user** — printed on the error page or one click away (an
integration convention, see
[integration-contract.md](../architecture/integration-contract.md)). Support asks one question:
"can you give me the trace ID?" The developer hands it to the agent: "user complained about X,
here is the trace ID — analyze it and figure out the fix." The agent walks the whole user
workflow through the Parallax CLI/API — what the user did, which services the request crossed,
where it failed — and proposes the fix or opens the PR.

Two boundaries inside this lifecycle, both operator-confirmed:

- **Production database state is a side-channel, not a Parallax feature.** When telemetry is not
  enough, the agent may ask the *developer* for read-only production-database access to inspect
  state. That grant flows between developer and agent directly — Parallax stays the
  traces/logs/metrics/observability layer and never proxies raw database access (consistent with
  [production-db-evidence.md](../capture/production-db-evidence.md) and the access-surface
  rejection of generic SQL tools).
- **The UI is the human trust surface for agent fixes.** The developer reviews the agent's PR,
  opens the Parallax UI (a Sentry-like website over the same API), checks the charts, metrics,
  and logs the fix claims to address, and concludes "the agent was right — this fix makes
  sense." Human verification of agent work is a first-class workflow
  ([simple-ui-v2.md](../architecture/simple-ui-v2.md)), which is also why `human_review` is a
  required field in the outcome records.

## 6. Who this is for (personas)

| Persona | What they get | Which rung |
| --- | --- | --- |
| **AI coding agent** (primary consumer) | Bounded, redacted, citable evidence bundles + dispatch wakes; enough context to propose or make the fix without a human gathering it | All |
| **Developer building with agents** (primary first user) | Local-first evidence server for "what did my app just do"; their agent debugs with runtime facts instead of guesses | 1 |
| **Team/SRE operating services** | Self-hosted, low-ops alternative to the five-tool stack; kubectl-style remote access; Sentry-grade issue workflow without 72 containers | 2 |
| **Hard-boundary organization** (air-gap, sovereign, compliance) | The paying segment: data ownership, open schema, audit-grade evidence and outcome records ([monetization-and-paying-segment.md](../validation/monetization-and-paying-segment.md)) | 3 |

## 7. What this note adds to the record

Relative to the prior vision notes, operator statement #2 (2026-06-11) contributes: the
**best-of-three-worlds** formulation (§2); the **three-surfaces-one-API** product shape with the
**kubectl context model** for the CLI (§3); the **audience ladder with local-dev-first priority**
made explicit (§4); the **polyglot-sources clarification** (OTLP ingest accepts any language SDK;
Rust-first remains the capture-depth and engine rule) (§4); and the **standing documentation
duty** — keep problem/audience/who-for sharp in every revision (header).

Operator statement #3 (2026-06-11) adds the three rung-1 lifecycles (§5): the feature-development
loop, the QA verification loop, and the reproduce-and-instrument loop — including the
breadcrumbs example as the concrete Sentry-gap-filler criterion, and the agent-led
instrumentation gap-closing mechanic that makes `missing_evidence` a load-bearing field.

Operator statement #4 (2026-06-11) adds: the **one-binary-plus-profile** deployment model
(`--profile local|cloud`, cloud-adapted defaults) replacing the assemble-a-Sentry-Grafana-Loki
stack ritual (§4); the **trace-ID complaint loop** as lifecycle 4, including the surface-the-
trace-ID-to-users integration convention (§5); the operator-confirmed boundary that **production
database state is a developer↔agent side-channel, never a Parallax feature** (§5); the **UI as
the human trust surface** for verifying agent fixes (§5); and the priority ruling that **rungs 1
and 2 are the first-priority audiences**, with rung-3 scalability as a design constraint rather
than a build focus (§4).

Nothing here changes the fixer boundary, the gates, or the claim-wording discipline.

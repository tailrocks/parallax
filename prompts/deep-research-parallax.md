# Deep Research Prompt — Project Parallax

I want to deeply research and validate a product/system idea currently called “Project Parallax.”

The goal is NOT to build another dashboard like Sentry, Datadog, or Grafana.

The goal is to research whether the industry is moving toward an AI-native debugging and investigation architecture where observability systems become context engines for autonomous or semi-autonomous agents.

This research should be extremely deep, technical, architectural, strategic, and critical.

Do not just summarize products.
I want:
- architectural analysis
- industry trends
- infrastructure tradeoffs
- operational complexity analysis
- AI-agent implications
- observability evolution
- distributed systems design implications
- open-source ecosystem analysis
- economic/business implications
- future platform direction

The research should be opinionated, technical, and evidence-based.

---

# Prompt Maintenance Rule

This prompt is a living source of operator intent for `/goal` and `/loop` runs.
When the operator clarifies the research direction, confirms a decision, changes
evaluation criteria, adds target domains, or names tools to compare, update this
file in the same change if future runs would otherwise use stale instructions.

Do not keep important direction changes only in chat or only in generated
research notes. The prompt must stay aligned with the current research target so
future autonomous runs continue from the latest operator decisions.

---

# Primary Objective and Research Sequence

Run this research in two phases. Do not jump to implementation before the first
phase has a defensible answer.

## Phase 1 — Go / No-Go (answer this first)

The first deliverable is a clear verdict on whether Parallax is worth building at
all. Answer, with evidence and an engineer's skepticism:

- Is the problem real and painful, or only assumed?
- Does this approach actually solve it, or only reframe it?
- Who are the direct competitors, and where exactly do they fall short for this
  goal?
- Does it make sense in the market and technically — or is it just a feature of
  Sentry / Grafana / Datadog?

Write the verdict to `docs/research/verdict.md` as an explicit GO or NO-GO with
the reasoning behind it. Do not soften the call. If the honest answer is NO-GO,
say so and explain why — that is a valid and useful outcome.

## Phase 2 — Implementation blueprint (only if Phase 1 is GO)

If, and only if, Phase 1 concludes GO, continue into the full technical
implementation concept defined under "Required Output" below: the API decision,
the component boundary, the three implementation tiers, and a named stack per
layer. Phase 2 is where the bulk of the deep technical research lives.

Keep questioning the idea across multiple passes. Going deeper and challenging
prior conclusions is expected; stopping shallow or early is not.

## Ongoing Research Mode

The Phase 1 verdict and Phase 2 blueprint are gates, not the end of the research
program.

For open-ended `/goal` runs, keep this prompt active as an indefinite research
brief after the verdict and blueprint exist. Do not mark the goal complete merely
because `docs/research/verdict.md` and
`docs/research/technical-implementation-concept.md` exist. Continue in repeated
passes until the operator explicitly stops the goal, replaces it, or says the
research program is complete.

Each pass should:

1. Re-read this prompt and the current `docs/research/` state.
2. Identify the weakest, least-proven, most stale, or most strategically
   important claim.
3. Check current primary sources and current project docs before relying on old
   conclusions.
4. Add or revise a focused, source-linked Markdown note under `docs/research/`.
5. Update this prompt, `README.md`, and `PROJECT_STRUCTURE.md` when durable
   direction or repository shape changes.
6. Commit and push each durable research section.
7. Move to the next highest-value research gap instead of declaring the overall
   program finished.

Keep challenging the GO verdict, storage choice, API boundary, CLI/MCP access
surface, scaling tiers, frontend direction, agent/CLI tracing model, safety
model, and market positioning with fresh evidence. A good ongoing run should
improve, replace, or narrow prior conclusions when the evidence changes.

---

# Project Vision and Overall Target

This is the north star the whole research serves. Everything is moving to an
AI-centric way of building and running software, and in that world observability
becomes a key layer — eventually we need to know what is happening in every
system, and an AI model is often capable of fixing issues itself. The gating
factor is not the model's ability; it is context. An AI is only as good as the
context it is given.

## The belief

Software is now fast to build and slow to debug in production. Code generation,
CI/CD, and provisioning are fast; finding out what actually went wrong in
production is still slow. Two reasons dominate:

1. The AI does not have access to everything it needs (telemetry, the database,
   the runtime state).
2. Even with access to the data, the AI still has to reconstruct the lifecycle —
   how the system reached the error, and what led to that state.

The perfect world this project assumes:

- A monorepo that stores as much as possible in one place: frontend, backend, and
  everything else. More of the system in one place means more context for an AI
  to understand how the whole thing works.
- Not only code. The repository should also hold documentation, design decisions,
  tasks (what was worked on, what was finished, and why), and the roadmap. Code
  alone is slow for an AI to derive meaning from; explicit intent, decisions, and
  direction give it the "why," the purpose, and what each thing is meant to
  solve. With that in the repo, an AI can work far more autonomously and build
  according to the plan.
- Parallax is the runtime half of that same idea. The repository explains why the
  code is the way it is; Parallax explains what happened at runtime and how the
  system got to a failure. Together they give an AI a near-complete picture.

## What the system must make possible

When a production error fires, an AI (or a human) should be able to pull
everything connected to that moment and reconstruct the path to it:

- detailed error messages and the Sentry-style error event;
- logs, including debug logs, as an audit trail of what happened and why
  decisions were taken;
- traces and spans, including how long each span took;
- metrics describing what the system was doing overall;
- the release/deploy and change context around the window.

The storage layer should let us always extract this data and reconstruct "how we
got to that stage and what led to it." Cheap, durable retention matters: object
storage / S3-style backends are close to a requirement, because the value depends
on being able to keep and re-extract history without cost anxiety.

## The end state

Given all of this context, the expectation is that in most cases an AI can act
without hand-holding:

- open a pull request that fixes the issue directly, or
- open a pull request that states the problem, proposes a few candidate fixes,
  shows the research and evidence behind them, recommends the one it finds most
  logical, and asks only when it needs a human to choose a direction.

That is the point of the approach: with enough structured context, the agent
makes the call and brings evidence, instead of asking a human to gather context
first.

## Separation of concerns: Parallax stores, a separate agent fixes

Be precise about the component boundary, because it shapes the whole design:

- Parallax itself does NOT fix issues. Parallax is the system that stores and
  serves the data — the runtime evidence and context engine. Its job is
  ingestion, storage, correlation, and handing back the right context.
- A separate component (playing a Sentry-like role) sits on top: it pulls the
  relevant evidence from Parallax, pulls the source code from the repository,
  connects to a coding agent, and opens the pull request as the fix. The fix
  itself happens inside the agent, which is already very capable.
- For the agent to do this it must reach Parallax for context. The first access
  path is the CLI — a coding agent already operates through CLI commands, so
  Parallax must expose its evidence through a CLI from the start.
- Whether Parallax also needs a dedicated MCP server, or whether the CLI is
  enough for an agent to consume context, is an open question this research must
  answer directly. Treat the MCP-server decision as a focused research item, not
  an afterthought.

So the layering is: Parallax (store and serve context) → access surface (CLI,
and maybe MCP) → separate fixer component → coding agent → pull request. Keep
Parallax scoped to evidence and context; keep fixing in the agent layer.

## Confirmed extension: observe agents and CLI work too

The project is not only about production services. The current confirmed
direction is that Parallax should become an evidence engine for software
execution across:

- services;
- CI runs;
- CLI applications;
- coding-agent sessions.

Since engineering work is moving toward agents, we need to understand what an
agent is doing when it works with a system. The system should preserve an audit
trail of what happened, including what the agent saw, what it decided, which
tools it used, which files it read or changed, which commands it ran, which
tests it executed, which PR or patch it produced, and what outcome followed.

This matters for normal debugging and for bad outcomes. If a user reports an
error, an agent makes the wrong change, a migration corrupts data, or a database
table is dropped, we need to reconstruct how it happened. The problem with
agents is that many actions happen quickly and across layers, but without a
trace we cannot see the causal chain.

Parallax should add observability across these layers so a human or agent can
ask questions later:

- What did the agent do before this incident?
- Which command or tool touched this database/table/resource?
- Which context did the agent use, and was it stale?
- Which files changed before this deploy?
- Which policy, approval, or guardrail allowed the action?
- Did the agent fix the issue, worsen it, or leave it inconclusive?

CLI applications are part of the same model. A CLI run is bounded and
reproducible, and coding agents often touch the world through CLI commands.
Research should treat CLI invocations as first-class traces, including command
name, subcommand, sanitized args/env, cwd, repo/branch/commit, config refs,
stdout/stderr excerpts, exit code, panic/error chain, spawned child processes,
tests/build/deploy steps, and redaction policy.

## Confirmed extension: the frontend is a first-class collection target

Parallax is not backend-only. The frontend is a large system in its own right,
and a major share of real incidents are user-facing: a user hits an error and we
must reconstruct what led to it. So the confirmed direction is that Parallax must
be able to collect from the frontend too, and — critically — correlate that
frontend evidence with the backend and the rest of the microservices
architecture, because the frontend is connected to the backend and the real cause
often crosses systems.

What the frontend capture must make possible, for a user-facing error:

- the error/exception itself with a usable stack (source-mapped, not minified);
- the sequence of steps that led to it — user actions, navigation, and
  breadcrumbs ("what previous steps led here");
- relevant frontend state at the time (route, component/view, feature flags,
  sanitized app state) and console/network logs;
- the outbound request(s) that crossed into the backend, and the trace/span IDs
  that tie the frontend session to backend spans, logs, errors, and metrics;
- the release/deploy/build context of the frontend, separate from but joinable to
  the backend release context.

Cross-system reconstruction is the point. The research must treat
frontend↔backend↔microservices correlation as a core requirement, not a later
add-on: propagate a trace from the browser through the API gateway into backend
services (W3C `traceparent` / OTLP context propagation), and make a single
evidence bundle able to span the user's frontend session and the backend
lifecycle it triggered. "How did we get to this user-facing error" must be
answerable across the boundary, not only inside one tier.

Research must answer, specifically:

- collection method for the frontend: browser/JS-TS SDK emitting OTLP and/or the
  Sentry browser envelope, source maps for symbolication, session/breadcrumb
  capture, optional session replay, and Real User Monitoring (RUM) signals — and
  what is essential versus nice-to-have;
- how to propagate and join trace context across the frontend↔backend boundary so
  a frontend error links to the exact backend spans/logs/errors it caused or was
  caused by;
- privacy: the frontend carries heavy PII and user content, so redaction,
  consent, and data-minimization for breadcrumbs, state, replay, and logs are
  harder here and must be designed in;
- the evidence bundle and schema must extend to frontend nodes (session, user
  step/breadcrumb, frontend error, route/view, frontend release) and cross-tier
  edges, without breaking the open schema.

Scope note: this does not change the language/runtime filter. The frontend is a
telemetry **source** (a JS/TS browser client SDK), exactly like any app that
emits Sentry/OTLP data; the Parallax engine and its infrastructure stay
Rust-first and within the Rust/Go/Zig/C++/C filter. The Rust-applications-first
initial scope still holds for the backend; frontend collection is a confirmed
roadmap target alongside services, CI, CLIs, and coding agents.

## What this research must prove

This vision is a strong belief that needs verification, not assumption. The
research must answer, from a technical perspective:

- Does this make sense, and what is missing?
- What is genuinely essential versus nice-to-have?
- Is causal/lifecycle reconstruction ("how did we get here") actually achievable
  from telemetry, or only partially?
- What are the hard problems and dangers: giving an agent access to systems and
  data (including a database), privacy and secrets, trust in autonomous pull
  requests, and the cost/scale of retaining enough history?
- Concretely, how would we build it and what should we use — which infrastructure
  projects are capable of serving this goal under the evaluation lens and
  benchmark axes below?
- Whether agent-session and CLI tracing are technically feasible as a core
  Parallax capability, not a later side feature.
- Whether the system can answer audit and question-answering workflows over
  agent actions, CLI side effects, runtime evidence, CI, deploys, and outcomes.

The job of Parallax-as-research is to prove or disprove that a Rust-first,
open-source, self-hostable observability system can become the runtime context
engine that makes autonomous AI debugging real.

---

# Evaluation Lens (Most Important)

This is the lens for the entire research. Apply it to every candidate.

I am an engineer. I care about which system delivers the performance and which
project is most deeply connected, by design, to the goal stated here.

Judge every candidate ONLY by:
- raw performance
- architecture and design quality
- how deeply the project is purpose-built, down to the small details, for this goal
- operational simplicity
- technical fit for an AI-native debugging/investigation context engine

Explicitly IGNORE:
- team size
- company backing or funding
- popularity, fame, hype
- market share or community size
- age or "maturity as credibility"

None of that matters. It does not matter how big the team is or how famous the
project is.

We do NOT care about legacy systems. This is the AI world. I care about new
solutions designed from first principles for AI-native workflows. A legacy
system retrofitted to this goal is not a point in its favor; being incumbent is
not an advantage here.

## Why Rust

In the AI world I believe a Rust-based project has the advantage, so I weight
Rust projects heavily:
- AI coding agents work extremely well with Rust
- strong compiler feedback loops and compile-time guarantees help agents
- systems-programming performance
- low memory, high throughput, minimal infrastructure
- operational simplicity and predictability

When two candidates are close on the technical lens above, the Rust-native one
wins.

## Why Open Source

In the AI world, open source is the key. With AI agents, anyone can open a pull
request and propose an improvement, so the rate of contribution and iteration on
open projects compounds far faster than on closed ones. An open, agent-friendly
codebase is itself a performance and survival advantage.

The whole system is designed around this concept first: open source, Rust-native,
agent-contributable. Favor projects whose license, architecture, and codebase
make external and agent-driven contribution easy.

"Best fit" means the system whose core design most directly serves, from the
smallest details up:
- append-only event ingestion
- unified observability storage
- correlation
- causal reconstruction
- context-graph building
- agent-consumable, machine-readable structured output

For each layer, answer two things concretely:
1. Who provides what performance?
2. Which project is architecturally closest to this goal — which one looks like
   it was designed, from scratch and in every detail, to achieve this perfectly?

The purpose of this research is to decide, purely from the technical
perspective, which direction to take and which system to use to build this the
right way the first time.

---

# Benchmarking and Comparison Methodology

Comparison is part of the research, and it has two halves.

Version freshness rule: every comparison must use the latest reasonably
available stable/public version of each candidate as of the research date. Do
not compare a current project against an older major release, stale benchmark,
old architecture doc, or outdated feature matrix unless the point is explicitly
historical. When versions matter, state the version or release date being
compared and call out if a source is stale. This applies to every named
candidate, including Aduce if it is compared in future research.

1. Study how systems are designed. Read public benchmarks and, more importantly,
   the architecture and design docs behind each candidate: data layout, ingest
   path, query path, indexing, compaction, retention, replication, and
   clustering model. Public benchmarks are a starting signal, not the verdict —
   most are vendor-published and measure the workload that flatters the vendor.

2. Build a prototype of our own benchmark, do not only describe one. The
   research must produce a concrete, runnable evaluation harness — not just a
   plan — that measures each storage candidate (GreptimeDB, ClickHouse, and any
   other in-scope system) against THIS system's purpose, not a general-purpose
   leaderboard. A system can be excellent for the job it was designed for and
   still be wrong for us. The question is never "which system is best in general"
   — it is "which system is best designed for OUR purpose."

   The benchmark prototype is a required deliverable and must be specific enough
   to run, including:
   - a representative dataset and a generator for it: mixed error events, OTLP
     logs/traces/metrics, deploy markers, CI/CLI/agent traces, and frontend
     sessions, at realistic cardinality and trace-linkage;
   - the actual schema/DDL per candidate and the exact queries that matter to
     Parallax — evidence-bundle and correlation queries (issue context,
     trace/log/metric join, release-regression window, cross-tier
     frontend↔backend reconstruction), not generic scans;
   - the metrics tied to the axes below: ingest-to-queryable freshness,
     evidence-bundle/correlation query latency, behavior under concurrent
     ingest+query, retained size and compression by signal, and object-storage
     cost/retention math;
   - a harness that drives load and records results reproducibly, with the
     candidate behind a storage abstraction so candidates can be swapped.

   The storage benchmark has veto power over the default storage choice: no
   storage winner is declared until the prototype is run against the latest
   stable versions. Grow `docs/research/observability-storage-benchmark-plan.md`
   from a plan into this runnable prototype spec, and keep the metadata-store
   benchmark aligned the same way.

Judge every candidate on these axes, in priority order:

## 1. Speed — time to see real data

How fast can we see what is happening right now? Measure the end-to-end path,
not just raw ingest:
- ingest-to-queryable latency (data freshness): how long after an event arrives
  until it is visible in a query;
- query latency for our access patterns (evidence-bundle and correlation
  queries, not generic scans);
- behavior under concurrent ingest plus query (real-time view while data still
  streams in).

The winning system lets a human or agent see "what is going on" with the lowest
delay.

## 2. Cost — storage size and money

How expensive is this to run and keep?
- retained storage size per unit of telemetry, and compression ratio by signal
  (logs vs traces vs metrics vs error events);
- money cost of that storage (object storage vs local SSD/block), including
  request and egress cost where relevant;
- compute cost for ingest and query;
- retention math: cost to keep N days or weeks of real data.

If a candidate is fast but its storage is huge or expensive, say so explicitly
and quantify it.

## 3. Scaling — how hard to grow, horizontal first

Some systems are designed for a single machine. We must know what happens when
one machine is no longer enough:
- single-node ceiling: where does one box stop being enough (ingest rate, data
  size, query concurrency)?
- scale-out difficulty: how hard is it to go past one node — config, operational
  burden, new dependencies, rebalancing, data movement?
- does performance hold when scaling? Adding parallel servers must keep
  processing fast; a system that scales but degrades sharply is a weak fit.

Horizontal scaling matters most: when we add parallel servers, does throughput
and query speed keep up? Vertical scaling (a bigger box) also matters but is
secondary. A candidate that only scales vertically is a serious limitation and
must be flagged. For each candidate, state both the single-node story and the
horizontal-scale story, and whether the architecture was designed for scale-out
from the start or had it bolted on.

---

# Background and Motivation

My current workflow changed dramatically because of AI coding agents.

I heavily use:
- Claude Code
- Codex
- Amp
- OpenCode
- multiple autonomous agents
- parallel agent orchestration systems

This massively accelerated software development speed.

I mostly use Rust because:
- AI works extremely well with Rust
- compiler feedback loops are strong
- compile-time guarantees help agents
- systems programming performance matters
- infrastructure complexity can be reduced

As development speed increased, debugging and observability became the new bottleneck.

CI/CD is fast.
Code generation is fast.
Infrastructure provisioning is fast.

But debugging production systems is still primitive.

The current workflow is fragmented:
- Sentry for errors
- Grafana for metrics
- Loki/ELK for logs
- Jaeger/Tempo for traces
- GitHub Actions for CI
- GitHub/Linear/Jira for issues
- deploy systems for release metadata

The debugging workflow remains mostly manual.

---

# My Main Frustration With Existing Systems

## Sentry

I currently use self-hosted Sentry.

Sentry itself is a strong product.
The issue is not product quality.

The issue is operational complexity.

Problems:
- self-hosted Sentry is extremely heavy
- too many moving parts
- Kafka
- Snuba
- ClickHouse
- workers
- Relay
- Redis
- Postgres
- operational burden
- difficult upgrades
- high infrastructure requirements
- Kubernetes support is weak/community-driven
- maintaining production-grade deployments is painful

Cloud Sentry also has issues:
- event pricing concerns
- unpredictable costs
- fear of losing observability during spikes
- limited freedom regarding ingestion/storage
- concerns around sending sensitive data externally

I strongly prefer:
- self-hosting
- infrastructure ownership
- operational predictability
- low-cost infrastructure
- efficient resource utilization
- simple deployments
- high performance
- control over retention and ingestion

I ended up with the feeling that:
- Sentry is useful
- but operationally overcomplicated for many teams

---

# The Key Insight

I believe AI agents fundamentally change observability requirements.

Traditional observability systems optimize for:
- dashboards
- visual exploration
- humans manually correlating information

But AI agents do not need dashboards.

Agents need:
- structured context
- evidence graphs
- semantic relationships
- correlated telemetry
- deploy metadata
- issue grouping
- timelines
- causal reconstruction
- machine-readable explanations

This changes the entire architecture.

The future observability system may become:
- API-first
- agent-first
- context-first
- explanation-first
- investigation-oriented

instead of:
- dashboard-first
- visualization-first
- human-navigation-first

---

# Core Thesis To Research

I want to research whether the future of observability is:

“turning telemetry into evidence-backed explanations for humans and AI agents.”

Instead of:
“showing charts and logs.”

The core idea is:
- ingest logs
- ingest metrics
- ingest traces
- ingest errors
- ingest CI data
- ingest deploy metadata
- ingest issue tracker context
- ingest frontend errors, user-step breadcrumbs, and session/RUM context, joined
  to the backend via propagated trace context

Then:
- correlate everything
- reconstruct causality
- build investigation context
- provide structured explanations
- feed AI agents enough context to autonomously debug systems

The system should eventually answer:

“What happened?”
“Why did it happen?”
“What changed?”
“What is the most likely root cause?”
“What evidence supports this?”
“What should be checked next?”
“What fix is likely?”

---

# Important Architectural Direction

Hard exclusion — language / runtime:
- Only high-performance, low-resource systems languages are in scope: Rust, Go,
  Zig, C++, and C. These are compiled and lean on memory and startup, which is
  the operational profile this system needs.
- Exclude heavyweight managed or interpreted runtimes entirely — Java/JVM,
  Python, Ruby, PHP, and similar. Remove them from every list (storage,
  messaging, collectors, anything), do not merely down-weight. The JVM profile
  (heap tuning, GC pauses, fat runtime) and interpreted-runtime overhead are
  exactly what this system avoids; they are also part of why self-hosted Sentry
  is heavy.
- Prefer Rust. When two candidates are close, the Rust-native one wins (see
  Why Rust). C++ examples that stay in scope: ClickHouse, Redpanda.

I do NOT want:
- another giant JVM-heavy operational monster
- enormous distributed infrastructure
- complicated dependencies unless necessary

I strongly prefer:
- Rust ecosystem
- operational simplicity
- low memory usage
- high throughput
- append-only/event-driven systems
- minimal infrastructure requirements
- modern systems written from scratch
- systems designed for AI-native workflows

I want to research whether modern Rust-native infrastructure stacks are now capable of replacing older operationally-heavy architectures.

---

# Technologies and Directions To Research

## Messaging / Streaming Layer

I suspect the system should be event-driven.

Research:
- Apache Iggy
- Redpanda
- NATS JetStream
- Liftbridge

Excluded by the language filter (JVM):
- Kafka, Pulsar — keep only as the baseline-to-beat for throughput and
  persistence numbers, not as deployable candidates.

Watch-list, not a candidate:
- WarpStream — Go, but proprietary and SaaS-shaped (Confluent-owned); fails the
  open-source lens.

Lead candidate: Apache Iggy. It looks like the fastest option that can replace
Kafka, and it is Rust-native, single-binary, and append-only — the right
operational profile. Kafka wire-protocol compatibility is NOT a requirement for
this system; do not weight it. What matters is raw speed, persistence guarantees,
and operational simplicity. Evaluate Iggy first and hard, then compare Redpanda
(C++) and NATS JetStream (Go) against it.

Questions:
- do we actually need Kafka-scale complexity?
- can Rust-native systems replace Kafka?
- what are operational tradeoffs?
- throughput comparisons
- persistence guarantees
- consumer group support
- backpressure handling
- partitioning models
- cloud-native operational simplicity
- self-hosting experience
- latency characteristics
- suitability for observability ingestion

Especially research:
Apache Iggy
https://iggy.apache.org/

I want deep analysis of:
- architecture
- maturity
- ecosystem
- limitations
- operational model
- suitability for observability pipelines

---

# Unified Observability Storage

Research:
- GreptimeDB
- ClickHouse
- VictoriaMetrics
- Mimir
- Tempo/Loki storage architectures

Especially:
GreptimeDB

Explicitly exclude as a storage engine:
- Elasticsearch/OpenSearch — too slow for high-volume observability ingest and
  query, and the search-index architecture is not the performance or operational
  profile this system wants. The only thing worth taking from
  Elasticsearch/Kibana is UI/UX: showing a log as a structured object (fields,
  surrounding context, what happened in that time window), not just a flat
  message string. Treat it as a presentation reference only, never a storage
  candidate.
- QuestDB, Apache Doris, Apache Pinot — Java/JVM stack, excluded by the language
  filter. Doris carries a Java frontend even though its backend is C++.

Questions:
- can one backend realistically store:
  - logs
  - metrics
  - traces
  - errors
  - events
- what are cardinality limitations?
- ingestion performance
- query performance
- object storage / S3 backend support (close to a requirement: cheap, durable
  retention so history can always be re-extracted for AI context)
- retention models
- compression
- distributed architecture
- operational complexity
- OpenTelemetry compatibility
- scalability
- schema evolution
- suitability for AI/semantic querying
- suitability for correlation workloads

Research whether GreptimeDB is truly a viable “unified observability database.”

---

# Metadata Store

Use Turso Database as the default metadata-store direction instead of C SQLite:

https://github.com/tursodatabase/turso

Research Turso as the Rust-first, SQLite-compatible metadata engine for
low-volume product state: users, projects, DSNs, issue status, redaction
policies, audit records, agent sessions, CLI invocations, and fix outcomes.

Treat Postgres only as a scale-out fallback if Turso production behavior,
ecosystem maturity, backup/restore, replication, or operational safety does not
hold up under research and benchmarks.

Do not describe the metadata layer as "use SQLite" except when discussing
SQLite compatibility. The operator preference is Turso.

---

# OpenTelemetry Research

Research:
- OpenTelemetry architecture
- logs/traces/metrics correlation
- semantic conventions
- OTEL collector pipelines
- ingest pipelines
- OTLP protocol internals
- scaling patterns
- production deployment architectures
- Rust-native collectors, especially Rotel (https://rotel.dev/,
  https://github.com/rotel-dev/rotel) — a Rust OTLP collector to evaluate as a
  reference design and possible component

Key question:
Is OpenTelemetry becoming the universal observability protocol layer?

And:
What opportunities remain ABOVE OTEL?

---

# Sentry-Compatible Ingestion

Research:
- Sentry SDK protocol
- Relay architecture
- grouping algorithms
- issue fingerprinting
- stacktrace normalization
- event processing pipeline

Questions:
- how difficult is it to build a Sentry-compatible ingest layer?
- what are the hardest parts?
- how much value is in compatibility?
- what parts are overengineered?
- what parts are genuinely difficult?

---

# Data Collection Method

How should the system actually collect data from applications? This is a core
open question. Research and compare:

- in-process language SDKs (Sentry SDK, OpenTelemetry SDK) emitting OTLP
- the Sentry ingestion API (envelopes)
- the OpenTelemetry API / OTLP
- eBPF-based, zero-instrumentation collection

The priority is getting as much useful data as possible. Specifically research
eBPF: what it is, how it works (kernel, syscalls, network, perf events), and
whether it can magically solve data collection — or whether it fundamentally
cannot capture app-level error semantics (language exceptions, Rust panics with
messages, typed error chains, span attributes, release/environment context).

Key questions:
- what can eBPF capture with zero code changes, and what can it NOT capture?
- for Rust specifically: symbol resolution, frame pointers, and stack unwinding
  limits under eBPF
- is eBPF a primary collection path, a complement (network/syscall/profiling),
  or a distraction for error-context capture?
- is the right answer SDK/OTLP for app errors plus optional eBPF for
  infrastructure-level signals?

---

# Initial Scope: Rust Applications First

The first target is Rust applications, because that is what we build. Research
the Rust-specific collection story end to end:

- how to detect and capture errors in Rust apps: panic hooks
  (`std::panic::set_hook`), `std::backtrace::Backtrace`, the `sentry` crate's
  panic integration, `tracing` + `tracing-error` (`SpanTrace`),
  `anyhow`/`eyre`/`color-eyre` error chains, and `opentelemetry` /
  `tracing-opentelemetry` for traces/logs/metrics
- what data to store: panic message, error type, backtrace frames
  (crate/module/function/file/line), error source chain, span/trace IDs,
  release, environment
- symbol/debuginfo requirements to get useful Rust backtraces (line tables,
  split debuginfo) without bloating binaries
- how to store and process this data efficiently in the Parallax pipeline

The goal: a clear, detailed Rust-first capture-and-storage design that produces
the best possible evidence for agents.

---

# AI-Native Observability

This is probably the MOST important research area.

Research:
- AI-assisted debugging systems
- autonomous incident investigation
- failure explanation systems
- root-cause analysis systems
- context engines
- observability for agents
- causal reconstruction systems
- telemetry knowledge graphs
- debugging copilots
- incident intelligence platforms
- agent integration surfaces: MCP (Model Context Protocol) servers, tool/function
  interfaces, structured context APIs
- how agents consume evidence (API/MCP/CLI) versus how humans consume dashboards

Research companies/products/projects like:
- Datadog AI features
- Sentry AI features
- Grafana AI
- New Relic AI
- Microsoft RCA systems
- Google SRE debugging research
- Uber flaky-test systems
- Meta observability tooling
- Amazon incident tooling

Direct open / self-hosted competitors to track every run (these sit closest to
the Parallax wedge and must be re-checked for whether they have closed the gap on
open + self-hosted + agent-native + evidence bundles + Sentry-compatible ingest):
- OpenObserve (Rust, object-storage, AGPL; AI SRE agent + MCP, currently
  Enterprise-gated, OTLP-only — watch whether the agent layer moves into the
  free tier or Sentry ingest is added)
- SigNoz (Go/ClickHouse; open self-hostable agent-native MCP — watch for a
  lighter footprint, Sentry ingest, or an evidence-graph/bundle abstraction)
- Coroot (eBPF, Go; self-hosted AI RCA)

Also track lightweight Sentry-compatible or OTLP-native self-hosted challengers
that pressure the migration and simplicity claims from below:
- Bugsink
- Rustrak
- Traceway
- GoSnag
- Urgentry

For these, watch specifically for Sentry SDK/envelope compatibility, OTLP
logs/traces/metrics correlation, low-resource setup, evidence-bundle export,
agent/MCP/CLI context access, coding-agent side-effect audit, and fix outcome
feedback loops.

Also perform a technical review of similar tools that provide observability for
agents and use them as references, not as a final product definition. Compare
their latest public/stable versions and focus on implementation details:

- LangSmith
- Langfuse
- Arize Phoenix and OpenInference
- Braintrust
- Datadog LLM Observability
- Helicone Sessions
- OpenLLMetry / Traceloop
- OpenLIT
- AgentOps
- Comet Opik
- Langtrace
- HoneyHive
- AgentTrace research
- AgentSight research

For each relevant tool, research:

- instrumentation model: SDK, proxy, framework callbacks, decorators,
  OpenTelemetry, eBPF, or manual spans;
- trace model: trace/session/thread/run root, span hierarchy, agent/tool/model
  spans, session grouping, parent/child relationships;
- storage and ingest architecture when public: OLAP store, relational metadata,
  queue, object storage, workers, OTLP endpoint;
- redaction/privacy controls and whether full prompts, args, tool outputs, and
  logs are opt-in or default;
- evaluation loop: scores, human review, datasets, experiments, accepted-fix
  feedback, production recurrence;
- self-hosting complexity and operational profile;
- whether it can audit coding-agent side effects such as files read/written,
  shell commands, DB actions, deploys, tests, patches, PRs, and outcomes.

The key question is not "which LLM tracing product is best?" The key question is
what technical patterns Parallax should reuse, and where existing tools stop
short of the Parallax goal: runtime evidence plus coding-agent and CLI action
audit.

I want to understand:
- what already exists
- what is missing
- whether agents truly change observability UX
- whether dashboards become less important
- whether APIs/context become more important
- whether current agent-observability systems can explain what happened across
  the real software system, not only inside an LLM application trace

---

# Flaky Test Investigation

Research whether flaky-test investigation is a strong initial wedge.

Research:
- Uber Testopedia
- Google flaky-test research
- CI observability
- test failure clustering
- retry analysis
- deterministic replay systems
- test root-cause analysis
- flaky-test detection algorithms

Questions:
- can flaky-test investigation become a standalone product?
- is it a sufficiently painful problem?
- are there existing strong solutions?
- why do companies still struggle with this?
- can AI agents actually fix flaky tests autonomously?

---

# Core Architectural Question

I want deep analysis of whether the future architecture should look like:

SDKs / OTEL / Sentry SDK
    ↓
append-only event stream
    ↓
stream processors
    ↓
unified observability storage
    ↓
correlation engine
    ↓
context graph
    ↓
AI investigation engine
    ↓
CLI / API / lightweight UI

instead of traditional:
SDK → DB → Dashboard

---

# Important Product Philosophy

I strongly believe:
- AI reduces the importance of complex dashboards
- CLI-first systems become more valuable
- APIs matter more than visual exploration
- machine-readable context is more important than visualization
- agents need structured evidence, not charts
- agent audit trails become essential as agents become a primary interface for
  operating software systems

One UI/UX exception worth keeping: when a human does look, a log should be shown
as a structured object — fields, surrounding context, and what happened in that
time window — not a flat message string. Elasticsearch/Kibana is the reference
for this object-centric log view (and only this), even though it is excluded as a
storage engine.

Research whether this philosophy is correct or naive.

---

# Critical Strategic Questions

Please critically evaluate:

1. Is this actually a company-sized opportunity?
2. Is this just a feature for Grafana/Sentry?
3. Is the market already too crowded?
4. What are the hardest technical problems?
5. What are the hidden operational problems?
6. What are the likely scaling bottlenecks?
7. What parts are commodity?
8. Where could a moat emerge?
9. Is the true moat:
   - correlation?
   - causality reconstruction?
   - agent integration?
   - telemetry graph modeling?
   - debugging datasets?
10. Does AI fundamentally change observability architecture?
11. Does the autonomy end state hold — can an agent realistically open a correct
    PR (or a well-reasoned proposal) from telemetry context alone, and how often?
12. Is causal/lifecycle reconstruction ("how did we get to this error")
    achievable from logs/traces/metrics/errors, or only partial?
13. The vision assumes a context-rich monorepo (code plus docs, decisions, tasks,
    roadmap). How much does Parallax's value depend on that, and what happens for
    teams that do not work this way?
14. What are the dangers of giving an agent access to systems and data, including
    a database — secrets, privacy, blast radius — and how is that bounded?
15. How should Parallax technically trace coding agents: model calls, tool
    calls, MCP/API calls, shell commands, files read/written, tests, patches,
    PRs, approvals, and outcomes?
16. How should Parallax technically trace CLI applications as first-class
    execution units?
17. Which existing agent-observability tools provide the strongest technical
    references, and where do they fail the Parallax audit/evidence-graph goal?
18. Can the system answer operational audit questions after bad outcomes, such
    as "which command changed this database object?" or "what did the agent do
    before this deploy?"

---

# Important Direction

I do NOT want shallow startup advice.

I want:
- deep systems analysis
- distributed systems thinking
- operational reality
- infrastructure tradeoffs
- AI-agent implications
- future platform evolution
- extremely critical feedback

Challenge assumptions aggressively.

Tell me:
- what is naive
- what is realistic
- what is technically difficult
- what is strategically dangerous
- what could become genuinely important

---

# Required Output: Technical Implementation Concept

Research is not finished at analysis. It must end in a concrete, opinionated
technical implementation concept — a blueprint someone could start building from.
Deliver, with reasoning tied to the evaluation lens and the benchmark axes:

- Which system to use at each layer, named and justified: collection/SDK, ingest
  gateway, messaging/stream (if any), storage, correlation/processing, and the
  agent-facing context surface (API/MCP). Include CLI tracing and coding-agent
  tracing layers. Make an actual recommendation, not a menu.
- What storage to use as the default, and why it wins on speed, cost, and scaling
  for this purpose — including the object-storage / S3 story.
- What metadata store to use, with Turso as the preferred default and Postgres
  only as a scale-out fallback if Turso fails the technical gates.
- The API decision: whether to follow the OpenTelemetry standard, the Sentry
  standard, or both — and concretely what the API must support, how it behaves,
  what data it stores, and how that data is stored. Do not leave this abstract.
- The component boundary and agent access path: state plainly that Parallax
  stores and serves evidence while a separate component plus a coding agent
  performs the fix, and decide the access surface — CLI first, and whether a
  dedicated MCP server is actually required or the CLI is sufficient.
- The technical view of the best way to implement it: component diagram, data
  flow from event to evidence bundle, the event/error data model, deterministic
  grouping and correlation approach, agent/CLI trace model, audit graph, and
  where causal/lifecycle reconstruction happens.
- A runnable benchmark prototype for evaluating and comparing storage systems
  (GreptimeDB, ClickHouse, and any other in-scope candidate) against the Parallax
  goal: dataset + generator, per-candidate schema/DDL, the exact
  evidence-bundle/correlation queries, the freshness/latency/cost/size metrics,
  and a reproducible harness with the candidate behind a storage abstraction.
  This is a deliverable, not a description, and it has veto power over the default
  storage choice (see "Benchmarking and Comparison Methodology").
- Tradeoffs and the rejected alternatives, so the choice is defensible.

## Scaling Trajectory: Three Tiers, Startups First

Treat scale as a trajectory with three explicit tiers, and design so moving
between them is a configuration and topology change, not a rewrite:

- Tier 1 — Simple (now, the priority): a tiny, single-node, low-resource,
  cheap-to-run deployment that beats self-hosted Sentry on simplicity. This is
  where the first version must win — startups and small teams.
- Tier 2 — Scalable: the same system handling meaningful growth in signal volume
  and query load by scaling out the seams (stream, storage, stateless
  processors) without changing the core design.
- Tier 3 — Very scalable: big-company horizontal scale to large volumes, with
  the architecture, data model, and interfaces proven not to paint us into a
  single-node corner.

The implementation concept must show all three: the smallest viable deployment,
the scale-out story, and the large-volume path — with the seams identified up
front so the small system grows into the large one by topology change, not a
rewrite.

---

# Final Goal

Help me determine whether this direction could evolve into:

“an AI-native debugging and investigation platform that becomes the intelligence layer between telemetry systems, CI pipelines, CLI applications, issue trackers, deployments, and autonomous coding agents.”

Or whether this idea is fundamentally flawed.

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

1. Study how systems are designed. Read public benchmarks and, more importantly,
   the architecture and design docs behind each candidate: data layout, ingest
   path, query path, indexing, compaction, retention, replication, and
   clustering model. Public benchmarks are a starting signal, not the verdict —
   most are vendor-published and measure the workload that flatters the vendor.

2. Prototype our own benchmark concept during the research. Define and sketch a
   benchmark (datasets, queries, metrics, harness) that measures each candidate
   against THIS system's purpose, not a general-purpose leaderboard. A system can
   be excellent for the job it was designed for and still be wrong for us. The
   question is never "which system is best in general" — it is "which system is
   best designed for OUR purpose."

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

I want to understand:
- what already exists
- what is missing
- whether agents truly change observability UX
- whether dashboards become less important
- whether APIs/context become more important

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

# Final Goal

Help me determine whether this direction could evolve into:

“an AI-native debugging and investigation platform that becomes the intelligence layer between telemetry systems, CI pipelines, issue trackers, deployments, and autonomous coding agents.”

Or whether this idea is fundamentally flawed.

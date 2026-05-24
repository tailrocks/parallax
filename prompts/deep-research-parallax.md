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
- Kafka
- Pulsar
- WarpStream
- Liftbridge

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
- QuestDB
- VictoriaMetrics
- Mimir
- Elasticsearch/OpenSearch
- Apache Doris
- Pinot
- Tempo/Loki storage architectures

Especially:
GreptimeDB

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

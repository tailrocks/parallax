# AI-Native Observability and Incident Intelligence

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note answers the AI-native observability section of the Parallax research
brief: whether agents change observability UX, what already exists, what is
missing, whether dashboards become less important, and where Parallax could
still build a defensible product.

Version freshness rule: this recommendation is based on current public docs and
source material checked on 2026-05-25. Every future benchmark or comparison must
use the latest reasonably available stable/public version of each candidate as
of the benchmark date, and must label older benchmark posts or architecture docs
as historical evidence.

## Short Answer

The thesis is directionally correct, but the market moved fast:

> AI-native observability is already becoming an agentic investigation layer, not
> only a dashboard assistant.

Datadog, Sentry, Grafana, Dynatrace, New Relic, Splunk, Robusta/HolmesGPT,
Causely, Meta, Microsoft, and others now expose the same core pattern:

```text
telemetry + topology + changes + code/context
  -> hypothesis-driven investigation
  -> evidence trace
  -> probable root cause or inconclusive result
  -> recommended action, PR, ticket, or remediation hook
```

That validates Parallax's direction, but it also removes any easy moat around
"AI explains observability data." The open product opportunity is narrower:

> Build the self-hosted, Rust-first, Sentry-compatible and OTLP-native evidence
> engine that produces portable, auditable context bundles for humans and coding
> agents.

Do not compete as a generic AI SRE over every signal. Compete as the system that
turns production errors, traces, logs, deploys, CI runs, and repo intent into a
bounded failure dossier an agent can safely act on.

## Current Product Signals

| Product / system | What exists now | Strategic implication |
| --- | --- | --- |
| Datadog Bits AI SRE | Always-on SRE agent, automatic monitor investigations, Slack prompts, hypothesis loop, evidence-backed conclusions or inconclusive output, Agent Trace view, data from metrics, APM traces, logs, events, change tracking, source code, RUM, network, DB, profiler, and preview third-party integrations. | Datadog is building the full enterprise incident agent. Parallax should not attack this head-on. |
| Sentry Seer | AI debugging agent using Sentry issue context, tracing, logs, profiles, and code; can root-cause, propose solution, generate code changes, and open PRs through an API. | The production-error version of Parallax directly overlaps with Sentry, so Parallax must win on self-hosting simplicity, open implementation, Rust ergonomics, and data ownership. |
| Grafana Assistant | AI observability agent over metrics, logs, traces, profiles, SQL data, investigations, queries, dashboards, MCP servers, API, Slack/Teams, and terminal CLI. | Grafana is turning dashboards into an agent platform. Parallax's CLI/API thesis is validated, but Grafana owns the broad LGTM path. |
| Dynatrace Intelligence / Davis | Causal AI RCA using topology, transaction, and code-level context; time correlation alone is explicitly not enough. | Topology-aware causality is table stakes for high-end RCA. Parallax must represent topology/evidence explicitly. |
| New Relic iRCA | Preview graph/causal RCA using topology graph, causal models, and path-based ranking. | The market is converging on causal graphs, not only anomaly correlation. |
| Splunk AI Assistant in Observability Cloud | Natural-language investigation across APM, infrastructure, DB, RUM, and log analytics; examples include trace analysis and upstream/downstream service RCA. | Existing observability suites will use AI to reduce query friction and accelerate investigations. |
| Robusta / HolmesGPT | Open-source AI SRE agent with read-only integrations into observability and infra systems, evidence links, audit logging, and recommendations. | Open-source AI SRE exists; Parallax needs a sharper runtime-evidence and product-state model. |
| Causely | Causal intelligence layer and MCP server positioned as grounding for production agents; recent benchmark claims lower time, tokens, and tool calls when agents get causal context. | The strongest version of Parallax is a context/causal layer, not a chat UI. |
| Meta DrP | Internal RCA platform at Meta with analyzer SDK, scalable execution, workflow integration, post-processing actions, 300+ teams, 2,000 analyzers, and 50,000 analyses/day. | Large companies codify investigations as reusable programs, not only prompts. Parallax should make analyzers/plugins first-class. |
| Microsoft RCACopilot | Research system matching incidents to handlers, collecting diagnostics, predicting root-cause category, and producing explanatory narratives over a year of Microsoft incidents; reported accuracy up to 0.766. | Even strong internal systems are imperfect. Parallax must expose confidence and missing evidence. |
| Google incident-debugging research | Google documents production debugging as a combination of tools, strategies, and low-level tasks, not a single dashboard action. | Debugging is an investigative workflow; Parallax should optimize the workflow state and evidence trail. |
| Amazon operational visibility guidance | Amazon emphasizes standardized instrumentation from aggregate operational metrics down to request-level troubleshooting data. | Application instrumentation remains essential; an agent context engine cannot recover facts the system never emitted. |

Sources:

- [Datadog Bits AI SRE](https://www.datadoghq.com/product/ai/bits-ai-sre/)
- [Datadog Bits AI SRE investigation docs](https://docs.datadoghq.com/bits_ai/bits_ai_sre/investigate_issues/)
- [Sentry Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer)
- [Sentry Seer Issue Fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/)
- [Grafana Assistant docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/get-started/)
- [Grafana Assistant MCP docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/configure/mcp-servers/)
- [Grafana Assistant CLI docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/guides/cli/)
- [Grafana Assistant everywhere announcement](https://grafana.com/blog/grafana-assistant-everywhere/)
- [Dynatrace RCA concepts](https://docs.dynatrace.com/docs/dynatrace-intelligence/root-cause-analysis/concepts)
- [New Relic iRCA announcement](https://newrelic.com/blog/ai/intelligent-rca-accurately-pinpoints-root-cause-in-seconds)
- [Splunk AI Assistant in Observability Cloud](https://help.splunk.com/en/splunk-observability-cloud/splunk-ai-assistant)
- [HolmesGPT docs](https://holmesgpt.dev/0.21.0/why-holmesgpt/)
- [Causely benchmark](https://www.causely.ai/product/benchmark)
- [Meta DrP RCA platform](https://engineering.fb.com/2025/12/19/data-infrastructure/drp-metas-root-cause-analysis-platform-at-scale/)
- [Microsoft RCACopilot](https://www.microsoft.com/en-us/research/publication/automatic-root-cause-analysis-via-large-language-models-for-cloud-incidents/)
- [Google: Debugging Incidents in Google's Distributed Systems](https://research.google/pubs/debugging-incidents-in-googles-distributed-systems/)
- [Amazon Builders' Library: instrumenting distributed systems](https://aws.amazon.com/builders-library/instrumenting-distributed-systems-for-operational-visibility/)

## What The Industry Is Really Building

The recurring architecture is not "LLM over logs." It is closer to:

```text
Signal collection
  -> normalized entities and topology
  -> known change/release/source context
  -> hypothesis generation
  -> iterative data access
  -> evidence-ranked conclusion
  -> workflow output
```

The important product shift is that vendors now expose the investigation itself
as a first-class object:

- Datadog has an Agent Trace view showing investigative steps and evidence
  evaluation.
- Grafana ships Assistant through UI, API, MCP, Slack/Teams, and CLI surfaces.
- Sentry has a Seer API where the stopping point can be root cause, solution,
  code changes, or open PR.
- Meta DrP codifies investigation playbooks as analyzers and executes them at
  scale.
- HolmesGPT emphasizes read-only tools, evidence links, and audit logging.

This validates Parallax's API-first and agent-first direction. It also means a
plain chat assistant is already obsolete.

## Do Agents Make Dashboards Less Important?

Yes, but not zero.

Dashboards remain useful for:

- broad situational awareness;
- operational reviews;
- human confidence building;
- long-running metrics and SLO tracking;
- visual anomaly exploration;
- teaching engineers what normal looks like.

Agents reduce the importance of dashboards during acute debugging because the
workflow changes from "human scans charts" to "system gathers evidence and
explains the investigation path." The agent needs machine-readable context:

- stable IDs;
- explicit time windows;
- trace/span/log joins;
- topology;
- release/change data;
- raw evidence references;
- permission boundaries;
- ranked hypotheses;
- missing-data warnings.

So the Parallax UI principle should be:

1. CLI/API/MCP first for investigation.
2. A small human UI for issue review, evidence inspection, and trust.
3. Object-centric logs and traces, not decorative dashboards.

The UI should answer "why does this evidence support this claim?" rather than
"how many chart panels can we render?"

## What Is Missing In Existing Products

The gaps are not evenly distributed. Incumbents have broad telemetry gravity,
but they often miss the exact shape Parallax wants.

### 1. Portable Evidence Bundles

Datadog, Sentry, Grafana, and New Relic keep investigations inside their cloud
or product context. Parallax can differentiate with portable bundles:

- JSON/Markdown/ZIP evidence dossier;
- deterministic fingerprints and raw evidence refs;
- redaction report;
- agent prompt/context file;
- reproducible query manifest;
- links back to source systems when available.

This is valuable for self-hosted teams, consulting/debugging handoff, and agent
workflows that span multiple tools.

### 2. Open Failure Schema

The open-source opportunity is not only code. It is an inspectable failure
schema:

- issue;
- event;
- trace;
- log excerpt;
- metric window;
- deploy;
- commit;
- CI run;
- test case;
- hypothesis;
- evidence edge;
- agent action.

If this schema is stable and useful, agents and external tools can build around
it.

### 3. Rust-First Runtime Evidence

Sentry supports Rust, and OTEL supports Rust, but neither is a Rust-first
debugging environment. Parallax can make the Rust path excellent:

- panic hook;
- `Backtrace`;
- `tracing` spans and fields;
- `tracing-error` span trace;
- `anyhow`/`eyre` source chain;
- debug ID / build ID;
- symbolication;
- release and commit mapping;
- agent-ready source references.

This is smaller than "all observability" and more defensible.

### 4. Repo Intent as Context

The user's monorepo assumption matters. Existing observability products know
telemetry and sometimes code. They rarely know:

- roadmap;
- design decisions;
- task history;
- architectural intent;
- why a feature exists;
- constraints chosen by the team.

Parallax can integrate runtime evidence with repo-held intent files, ADRs,
tasks, and plans. That is useful to coding agents because it gives the "why,"
not just the failing line.

### 5. Honest Causality

Many products market "root cause." Parallax should avoid overclaiming. The
stronger product language is:

- evidence-backed lifecycle;
- ranked hypotheses;
- deterministic edges;
- inferred edges;
- missing evidence;
- confidence level;
- recommended next checks.

This makes agent output safer and more auditable.

## Strategic Questions

### Is This Company-Sized?

Potentially, but not if the product is "open-source AI RCA."

The company-sized version is:

> the context and evidence layer for autonomous software maintenance.

That includes production errors, CI failures, flaky tests, deploy regressions,
issue ownership, and agent PR workflows. Observability is the data source; the
product is the runtime context engine between telemetry, repos, CI, issue
trackers, and coding agents.

### Is This Just A Feature?

For Datadog, Sentry, and Grafana customers, yes: generic AI investigation is a
feature inside the platform.

Parallax is only not-a-feature if it wins on:

- self-hosted simplicity;
- data ownership;
- open schema;
- low-resource Rust-first operation;
- Sentry-compatible migration path;
- OTLP-native ingestion;
- portable bundles;
- agent workflow across tools rather than inside one vendor.

### Is The Market Too Crowded?

The broad market is crowded. The narrow product can still exist.

Avoid:

- generic incident agent;
- generic observability assistant;
- generic dashboard replacement;
- generic flaky-test dashboard;
- LLM log summarizer.

Prefer:

- self-hosted error context engine;
- Rust-first capture path;
- Sentry-compatible ingestion;
- OTLP correlation;
- bundle/compiler model;
- typed evidence graph;
- MCP/API surface with strict safety boundaries.

### What Is Commodity?

Commodity or becoming commodity:

- collecting OTLP telemetry;
- generating natural-language summaries;
- querying logs with an LLM;
- basic anomaly detection;
- chat inside dashboards;
- generic SRE assistant wrappers over existing APIs.

Non-commodity:

- trustworthy failure grouping;
- cross-signal evidence stitching;
- issue-to-code-to-deploy linkage;
- causal edge modeling;
- data minimization and redaction for agents;
- reproducible investigation bundles;
- agent action policy;
- high-quality language/runtime-specific error capture.

### Where Could A Moat Emerge?

Possible moat order:

1. **Failure corpus and feedback loop.** Which evidence led to accepted fixes?
2. **Open evidence schema.** If tools and agents use the Parallax bundle format,
   the schema itself becomes leverage.
3. **Runtime plus repo intent.** Linking failures to docs, decisions, tasks, and
   roadmap is hard for telemetry-only products.
4. **Rust-first capture quality.** Better error chains, spans, backtraces, and
   symbolication produce better agent fixes.
5. **Trust layer.** Auditable confidence, redaction, permissions, and action
   boundaries matter more as agents become autonomous.

Telemetry storage alone is not a moat.

## Autonomy Reality

An agent can open useful PRs from telemetry context, but not for every incident.

| Failure type | Autonomous PR likelihood | Why |
| --- | --- | --- |
| Clear panic/exception in app code with stacktrace and simple invariant | High | The agent can map stacktrace to code and patch a local logic bug. |
| Regression after small deploy with failing test or obvious error event | Medium-high | Release/change context narrows the search. |
| Flaky test with deterministic repro evidence | Medium | Fix may be possible if the bundle captures timing, race, fixture, or external dependency clues. |
| Multi-service incident with partial traces | Medium-low | Agent can propose hypotheses, but root cause may need operator judgment. |
| Data corruption, privacy, billing, security, or production state mutation | Low | Human review and strict permissions are mandatory. |
| Infrastructure/provider outage | Low | Agent can summarize and mitigate, but cannot fix root provider cause. |

The right product behavior:

- PR directly for high-confidence, low-blast-radius application fixes.
- Proposal PR or issue for medium-confidence fixes.
- Investigation bundle only for high-risk or insufficient-evidence incidents.

## Hard Problems

### Data Quality

Agents fail when telemetry is missing, sampled, unstructured, or unjoined.
Parallax must make missing instrumentation visible, not silently invent context.

### Causal Confusion

Time correlation is not causality. Dynatrace, New Relic, Meta DrP, Causely, and
recent RCA papers all point in the same direction: topology and explicit
causal/evidence edges matter.

### Agent Reasoning Failure

Recent RCA-agent research found persistent pitfalls such as hallucinated data
interpretation and incomplete exploration across models. This implies Parallax
should reduce reasoning freedom by giving agents structured evidence, allowed
queries, and explicit confidence instead of raw unrestricted telemetry.

Source:

- [Why Do AI Agents Systematically Fail at Cloud Root Cause Analysis?](https://arxiv.org/abs/2602.09937)

### MCP Blast Radius

MCP is becoming a normal integration surface, but it also turns observability
data and operational APIs into callable agent tools. Parallax should expose
read-only context tools first and avoid generic `run_sql`, `run_shell`, or
production-mutation tools in the core product.

### Cost And Retention

The AI value depends on historical comparison: previous occurrences, first-seen
release, baseline metrics, prior incidents, retries, and fixed examples. That
requires cheap retention and compaction. This reinforces the GreptimeDB/object
storage direction and the need for retention math.

### Trust

The user does not need "the AI said so." They need:

- evidence refs;
- raw excerpts;
- query manifests;
- confidence;
- contradictions;
- missing evidence;
- reproduction or test status;
- PR diff and validation log.

## Product Implications For Parallax

### Keep

- Sentry-compatible error ingestion.
- OTLP-native traces/logs/metrics ingestion.
- Rust-first application capture.
- GreptimeDB default observability store.
- Turso metadata store for tiny/local deployments.
- Local WAL first, Iggy for durable profiles.
- Evidence graph.
- Read-only MCP/API first.
- Minimal UI, object-centric evidence inspection.

### Add

- A durable "investigation run" entity, similar in spirit to Datadog Agent Trace
  and Meta DrP analyzer executions.
- Analyzer/plugin model for deterministic steps before LLM reasoning.
- Portable evidence bundle export from day one.
- Explicit missing-data checks: "no trace ID," "logs sampled," "release
  unknown," "symbolication missing," "no deploy data."
- PR/proposal workflow that records evidence used, tests run, and confidence.

### Avoid

- Competing as a dashboard suite.
- Generic autonomous SRE for every alert.
- Claiming full root-cause proof.
- Letting the agent browse raw production data without scoped evidence bundles.
- Building around a single vendor's telemetry APIs.

## Revised Wedge

The best near-term wedge is not one thing for every market. It is a sequence:

1. **Rust production error context.** Replace the operationally heavy parts of
   self-hosted Sentry for Rust-heavy teams: Sentry-compatible ingest, stacktrace
   grouping, trace/log correlation, release context, and evidence bundles.
2. **CI failure bundles.** Add GitHub Actions/JUnit/Go test evidence bundles
   because they are deterministic, local, and agent-friendly.
3. **Flaky-test memory.** Build historical pass/fail/retry fingerprints once
   bundle data exists.
4. **Agent PR workflow.** Generate proposal or fix PRs only when evidence and
   test validation support it.
5. **Incident context graph.** Expand toward service-level RCA after the error
   and CI evidence model is proven.

This keeps the startup deployment small while preserving the long-term
trajectory toward the runtime context engine.

## Bottom Line

AI does change observability architecture, but not by deleting observability. It
changes the winning abstraction from dashboards to evidence.

Parallax is viable only if it becomes the open, self-hosted evidence layer that
agents can trust. The product should build deterministic context first, causal
edges second, summaries third, and autonomous PRs last.

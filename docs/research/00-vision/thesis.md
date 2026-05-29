# Project Parallax

<!-- markdownlint-disable -->

## AI-Native Debugging System

### Open Source, CLI-First Observability & Flaky Test Investigation

**Status:** idea validation document

**Purpose:** share with engineers, founders, investors, and observability/SRE practitioners to collect critical feedback.

**Current research status, 2026-05-25:** the maintained verdict is **GO only for
the narrow version**: a Rust-first, self-hostable runtime evidence/context
engine that accepts fixture-gated Sentry envelope error events, OTLP telemetry,
CLI invocation traces, and tested coding-agent session records, then serves
schema-valid, redacted evidence bundles. Do not read this thesis as a claim for
a generic AI RCA chatbot, a full dashboard suite, or autonomous production
mutation. The
current storage posture is also narrower than early drafts: the observability
store stays behind a ClickHouse/GreptimeDB adapter, the current proxy-lens lean
is ClickHouse, GreptimeDB remains the cost/cardinality/auto-rebalance branch,
and storage claims stay unproven until freshness, bundle-latency, object-cost,
and operational gates pass.

Maintained follow-ups:

- [Parallax Go / No-Go Verdict](verdict.md)
- [Technical implementation concept](technical-implementation-concept.md)
- [Storage benchmark prototype](storage-benchmark-prototype.md)
- [OTLP conformance ledger](otlp-conformance-ledger.md)
- [Evidence bundle and open schema](evidence-bundle-and-schema.md)

---

# 1. Executive Summary

Project Parallax is an early product thesis:

> **Build an open-source, AI-native debugging system that reconstructs the best
> available evidence around failures, using unified observability and execution
> data — starting with a CLI/API evidence-bundle workflow and expanding only
> after the bundle, redaction, and storage gates prove out.**
> 

The core insight is simple:

Modern engineering teams already collect huge amounts of data:

- errors
- logs
- metrics
- traces
- CI/test results
- deploy history
- issue tracker context

But during real debugging, engineers still manually jump between Sentry, Grafana, Kibana, Jaeger, CI logs, GitHub, and Linear/Jira.

Project Parallax aims to turn these fragmented signals into structured,
schema-bound investigation context that both humans and AI agents can use.

The initial validation wedge is an evidence-bundle evaluation: prove that a
bounded bundle helps a human or coding agent investigate better than raw
telemetry dumps. Flaky-test investigation remains a useful low-risk wedge, but
the maintained build direction now also includes fixture-gated Sentry envelope
error migration, OTLP telemetry, CLI traces, and tested agent-session adapters.

---

# 2. The Problem

## 2.1 Observability is fragmented

A typical modern stack looks like this:

| Need | Common tools |
| --- | --- |
| Error tracking | Sentry |
| Metrics | Prometheus / Grafana |
| Logs | ELK / Loki / Kibana |
| Traces | Jaeger / Tempo |
| Incidents | PagerDuty / incidents.io / Opsgenie |
| Issues | Linear / Jira / GitHub Issues |
| CI failures | GitHub Actions / GitLab CI / Buildkite / Jenkins |

Each tool is useful, but the debugging workflow is still mostly manual.

A real production debugging flow often looks like:

1. Error appears in Sentry.
2. Engineer checks logs.
3. Engineer opens Grafana.
4. Engineer inspects traces.
5. Engineer checks deploys.
6. Engineer checks recent PRs.
7. Engineer guesses possible root cause.
8. Engineer writes a ticket or hotfix.

The problem is not a lack of data.

The problem is:

> **The data is not assembled into an explanation.**
> 

Google’s SRE material emphasizes that monitoring must cover signals such as latency, traffic, errors, and saturation, and its workbook notes that monitoring can include metrics, logs, structured events, tracing, and event introspection. This confirms the industry direction: debugging needs multiple types of signals, not one isolated dashboard.

Sources: [Google SRE Book — Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/), [Google SRE Workbook — Monitoring](https://sre.google/workbook/monitoring/)

Google has also published research on how engineers debug incidents in distributed systems, describing the different tools, strategies, and investigation tasks engineers combine during production debugging. This supports the idea that debugging is an investigative workflow, not just a dashboard-viewing workflow.

Source: [ACM Queue / Google — Debugging Incidents in Google’s Distributed Systems](https://queue.acm.org/detail.cfm?id=3404974)

---

## 2.2 Self-hosted Sentry is powerful but operationally heavy

Sentry is a strong product. The issue is not that Sentry is bad.

The issue is that self-hosting Sentry can become a serious operational burden.

Sentry’s self-hosted data flow includes multiple components such as Relay, Kafka, ingest consumers, Snuba, ClickHouse, Postgres, Redis, and workers. Sentry’s own developer documentation shows this as a multi-component distributed system.

Source: [Sentry Developer Docs — Self-hosted Data Flow](https://develop.sentry.dev/self-hosted/data-flow/)

Snuba, Sentry’s event storage/query layer, is backed by ClickHouse and ingests through Kafka topics.

Source: [Sentry Snuba Architecture Overview](https://getsentry.github.io/snuba/architecture/overview.html)

Independent market evidence also points to this pain. Bugsink’s founder wrote “Why I gave up on self-hosted Sentry,” explicitly arguing that modern self-hosted Sentry became too complex and resource-heavy for many teams.

Source: [Bugsink — Why I gave up on self-hosted Sentry](https://www.bugsink.com/blog/why-i-gave-up-on-self-hosted-sentry/)

A Hacker News discussion around the same topic includes users saying they prefer hosted Sentry because it is less to manage, and that self-hosting is mainly justified for legal/compliance reasons.

Source: [Hacker News — I gave up on self-hosted Sentry](https://news.ycombinator.com/item?id=43725815)

Another writeup about self-hosting Sentry describes an expected short setup turning into real debugging of Relay logs, worker logs, Snuba migrations, and ClickHouse schema state.

Source: [Self-Hosting Sentry: What I Thought Would Take an Hour and Why It Didn’t](https://medium.com/%40manohays/self-hosting-sentry-what-i-thought-would-take-an-hour-and-why-it-didnt-e3355eaa8129)

Conclusion:

> **There is proven demand for Sentry-like value, but the operational model of self-hosted Sentry is painful for many teams.**
> 

---

## 2.3 Flaky tests are a large and well-known productivity problem

Flaky tests are not a niche problem. They are a recurring engineering productivity problem at large companies.

Google defines a flaky test as a test that both passes and fails on the same code, and reported that around 1.5% of all test runs across its corpus produced flaky results. Google also lists common causes such as concurrency, nondeterministic behavior, undefined behavior, third-party code, and infrastructure problems.

Source: [Google Testing Blog — Flaky Tests at Google and How We Mitigate Them](https://testing.googleblog.com/2016/05/flaky-tests-at-google-and-how-we.html)

Google later emphasized that flaky tests slow down the entire development process because automated tests stop providing a consistent signal.

Source: [Google Testing Blog — Test Flakiness: One of the Main Challenges of Automated Testing](https://testing.googleblog.com/2020/12/test-flakiness-one-of-main-challenges.html)

Uber has published multiple articles about flaky tests. In 2021, Uber described flaky tests as tests that return different pass/fail results without source code changes, and pointed to causes such as thread ordering, concurrency, and nondeterminism.

Source: [Uber Engineering — Handling Flaky Unit Tests in Java](https://www.uber.com/us/en/blog/handling-flaky-tests-java/)

In 2024, Uber published “Flaky Tests Overhaul at Uber,” describing Testopedia, a language/repo-agnostic service centered around a “test entity” with a fully qualified name. This shows that large companies build internal systems to track, classify, and manage flaky tests as first-class engineering objects.

Source: [Uber Engineering — Flaky Tests Overhaul at Uber](https://www.uber.com/us/en/blog/flaky-tests-overhaul/)

Google Research has also published work on automatically locating root causes of flaky tests across 428 Google projects. This is strong evidence that “explain why this test is flaky” is a serious research and engineering problem.

Source: [Google Research — DeFlake Your Tests: Automatically Locating Root Causes of Flaky Tests in Code at Google](https://research.google/pubs/de-flake-your-tests-automatically-locating-root-causes-of-flaky-tests-in-code-at-google/)

Conclusion:

> **Flaky test investigation is a strong initial wedge because the pain is frequent, measurable, and easier to validate than full production incident debugging.**
> 

---

# 3. Market Direction

## 3.1 OpenTelemetry is becoming the common telemetry standard

OpenTelemetry defines and standardizes telemetry signals such as traces, metrics, and logs.

Source: [OpenTelemetry — Signals](https://opentelemetry.io/docs/concepts/signals/)

OpenTelemetry’s documentation describes it as a vendor-neutral open-source observability framework for generating, collecting, and exporting telemetry data such as traces, metrics, and logs.

Source: [OpenTelemetry Docs](https://opentelemetry.io/docs/)

OpenTelemetry logs also explicitly include correlation concepts: logs can be correlated by timestamp, execution context, and resource context. This is important because it supports the idea that logs, metrics, traces, and errors should not be treated as isolated systems.

Source: [OpenTelemetry Logs Specification](https://opentelemetry.io/docs/specs/otel/logs/)

Conclusion:

> **The industry is standardizing the data collection layer, but the debugging/explanation layer is still immature.**
> 

---

## 3.2 GreptimeDB and similar systems point toward unified observability storage

GreptimeDB positions itself as an open-source observability database for “Observability 2.0,” treating metrics, logs, and traces as one unified data model rather than three separate pillars.

Source: [GreptimeDB GitHub](https://github.com/GreptimeTeam/greptimedb)

GreptimeDB supports OpenTelemetry Protocol ingestion for traces, metrics, and logs.

Source: [GreptimeDB Docs — OpenTelemetry Protocol](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/)

Greptime has also written about unified observability with native OpenTelemetry traces support alongside metrics and logs, enabling cross-correlation in one backend.

Source: [Greptime — OpenTelemetry Traces Integration with GreptimeDB](https://greptime.com/tech-content/2025-07-17-opentelemetry-traces-integration-with-greptimedb)

GreptimeDB reached `v1.0.2` as the latest stable release checked on
2026-05-25. Its GA status reduces unreleased-database risk, but its trace
read/write docs still mark trace support as experimental, and vendor-published
benchmarks do not prove Parallax's evidence-bundle workload. The current
research position is: ClickHouse is the pragmatic proxy-lens lean for retrieval
speed and build-on-top ecosystem, while GreptimeDB stays alive for PromQL,
object-storage economics, high-cardinality metrics, and auto-rebalance until
the benchmark gates settle speed, cost, and operations.

Sources: [GreptimeDB v1.0.2 release](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.0.2),
[GreptimeDB trace read/write docs](https://docs.greptime.com/user-guide/traces/read-write/),
[Storage benchmark prototype](storage-benchmark-prototype.md),
[Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md),
[Storage size and object cost gate](storage-size-and-object-cost-gate.md)

Conclusion:

> **Storage is converging, but storage fit is not the product. The opportunity is
> the evidence layer that turns unified data into schema-valid, redacted,
> citable context.**
> 

---

# 4. Proposed Solution

Project Parallax should not start as “another dashboard.”

It should start as:

> **A CLI-first, AI-native investigation engine.**
> 

The core workflow:

```
Failure happens
    ↓
Project Parallax collects related context
    ↓
It groups and summarizes relevant logs, traces, metrics, CI data, and deploy changes
    ↓
It builds a bounded evidence bundle and ranked hypotheses with citations
    ↓
It gives the engineer or AI agent actionable next steps, or says evidence is inconclusive
```

Example CLI commands:

```bash
parallax issue explain ISSUE-123
parallax test explain checkout_should_apply_discount
parallax trace explain <trace_id>
parallax context build --issue ISSUE-123 --window 30m
```

Example output:

```
Root cause hypothesis:
- Redis latency spike caused checkout timeout.

Evidence:
- Error rate increased after release 2026.04.30-7.
- Trace span payment.authorize increased from 120ms p95 to 2.8s p95.
- Logs show retry exhaustion from payment worker.
- Metric redis_p95_latency increased 4x during the same window.

Suggested actions:
1. Check release diff for payment retry logic.
2. Inspect Redis latency and connection pool saturation.
3. Consider rolling back feature flag new_payment_routing.
```

The CLI is important because it works naturally with:

- engineers
- CI
- scripts
- AI agents
- GitHub Actions
- Linear/Jira automation
- incident bots

A UI can come later, but the initial differentiator is not a better dashboard.

The differentiator is:

> **structured debugging context for humans and agents.**
> 

---

# 5. Initial Wedge: Flaky Test Investigation

The best starting point may be flaky tests rather than production incidents.

Why:

- faster feedback loop
- lower production-risk
- easier data access
- easier to prove value
- frequent pain in engineering teams
- less direct competition with Sentry/Datadog/Grafana

Initial feature set:

```
- collect CI test runs
- identify flaky tests
- group failures by test entity + failure signature
- correlate failures with logs, timing, environment, retries, and recent code changes
- produce explanation + suggested fix
```

Example:

```
Test: checkout_should_apply_discount

Flakiness:
- Fails in 17% of runs
- Passes on retry in 82% of failed runs

Likely causes:
1. Order dependency
   - Usually fails after user_cleanup_test
2. Race condition
   - Logs show cache not warmed before assertion

Suggested fix:
- isolate test data
- avoid shared cache state
- add deterministic setup phase
```

This aligns strongly with Uber’s Testopedia work and Google’s research on flaky test root-cause localization.

---

# 6. Long-Term Vision

The long-term vision is not “replace Sentry” as a first-order goal.

The long-term vision is:

> **Build the intelligence layer for software reliability.**
> 

That means:

```
CI failure → explanation
Flaky test → explanation
Production error → explanation
Incident → explanation
Regression → explanation
```

Project Parallax can become the system that sits between telemetry/CI/issue trackers and AI agents.

Potential integrations:

- OpenTelemetry
- GreptimeDB
- Postgres
- GitHub Actions
- GitHub Issues / Pull Requests
- Linear
- Sentry envelope error-event ingestion
- Slack
- incident management systems

Example agentic flow:

```
Failure detected
    ↓
Project Parallax builds investigation context
    ↓
Linear/GitHub issue created
    ↓
AI agent receives structured context
    ↓
Agent proposes or creates PR
    ↓
Engineer reviews
```

This aligns with the broader shift toward agentic software development.

---

# 7. Open Source Strategy

The recommended strategy is open source first.

Reasons:

1. Trust matters in observability and debugging.
2. Engineers want to inspect and extend tools that touch production data.
3. AI makes software cheaper to build, so closed-source-only defensibility is weaker.
4. Open source can become a distribution advantage.
5. Teams can self-host initially, then pay for convenience later.

The model is similar to:

```
Open source core
    +
Hosted cloud / managed service
    +
Enterprise support / compliance / integrations
```

This does not mean “no business.”

It means:

> **Adoption first, monetization second.**
> 

Many startups will still prefer hosted cloud because maintaining observability infrastructure is time-consuming and not their core business.

The business thesis:

```
Even if self-hosting is free, many teams will pay to avoid operating it.
```

---

# 8. Monetization Hypothesis

Possible monetization paths:

## 8.1 Hosted Cloud

For teams that want zero maintenance.

Charge by:

- seats
- repositories
- test runs
- events
- AI investigations

## 8.2 Enterprise Self-Hosted Support

For companies that must keep data inside their own environment.

Paid features:

- SSO/SAML
- RBAC
- audit logs
- compliance
- private deployments
- support SLAs

## 8.3 AI Investigation Credits

Charge for high-value AI workflows:

- root cause explanation
- PR suggestion
- failure clustering
- incident summary
- postmortem draft

---

# 9. Competitive Landscape

## 9.1 Sentry

Strengths:

- mature error tracking
- strong UX
- broad SDK support
- issue workflow

Weakness/opportunity:

- self-hosted complexity
- not primarily an AI investigation engine
- telemetry correlation still requires multiple workflows

## 9.2 Grafana / Prometheus / Loki / Tempo

Strengths:

- great dashboards
- open ecosystem
- strong metrics/logs/traces story

Weakness/opportunity:

- dashboard-first
- human must still interpret signals
- not focused on automatic explanation

## 9.3 Bugsink / GlitchTip

Strengths:

- simpler Sentry alternatives
- self-hosting focus

Weakness/opportunity:

- mostly error tracking
- not unified observability + AI investigation

## 9.4 Datadog / New Relic / commercial APM

Strengths:

- broad platform
- mature integrations

Weakness/opportunity:

- cost
- vendor lock-in
- less open-source/community-driven
- may not fit teams wanting ownership and extensibility

## 9.5 Internal tools at large companies

Examples:

- Uber Testopedia
- Google flaky test research/internal systems

Opportunity:

> Smaller companies cannot afford to build these systems internally.
> 

---

# 10. Why This Might Work

Strong supporting signals:

1. Google and Uber have both publicly documented flaky test pain and mitigation systems.
2. Sentry self-hosted complexity is a known and publicly discussed pain.
3. OpenTelemetry is standardizing telemetry collection.
4. GreptimeDB-like systems are pushing toward unified observability storage.
5. AI agents need structured context, not dashboard screenshots.
6. Engineering teams increasingly want tools that can connect issue trackers, CI, telemetry, and code changes.

Core thesis:

> **The market is moving from “show me the data” to “explain the failure.”**
> 

---

# 11. Why This Might Be Naive

This section is intentionally critical.

Possible weaknesses:

## 11.1 AI explanations may not be trusted

Engineers may not trust AI-generated root cause hypotheses unless evidence is very clear.

Mitigation:

- always show evidence
- provide confidence
- make conclusions reproducible
- never hide raw data

## 11.2 Data quality may be poor

Many teams do not have clean logs, traces, metrics, or deploy metadata.

Mitigation:

- start with CI/flaky tests where data is more structured
- provide instrumentation helpers
- degrade gracefully

## 11.3 Existing tools may add similar features

Sentry, Datadog, Grafana, GitHub, or Linear could build AI explanations.

Mitigation:

- open source distribution
- CLI-first workflow
- agent-first design
- focus on composability rather than closed platform

## 11.4 The wedge may be too narrow or too broad

Flaky tests may be a strong entry point, but production incidents are a larger market.

Mitigation:

- validate flaky-test demand quickly
- keep architecture general enough for production errors later

## 11.5 Selling open source to investors is harder

Investors may worry about monetization.

Mitigation:

- show adoption-first strategy
- sell hosted convenience
- sell enterprise support
- emphasize that open source is a distribution moat

---

# 12. Validation Questions

Please be direct and critical.

## Problem validation

1. Have you personally lost time debugging flaky tests or production errors?
2. How often do you jump between Sentry, Grafana, logs, traces, CI, and GitHub?
3. Which part of debugging is most painful today?

## Product validation

1. Would a CLI-first tool fit your workflow?
2. Would you use a tool that produces an evidence-backed failure explanation?
3. Would you trust AI explanations if they showed supporting logs/traces/metrics?

## Market validation

1. Would your team self-host this if it were open source?
2. Would your team pay for a managed version?
3. Is flaky test investigation a strong enough entry point?
4. What would make this idea fail?

## Strategic feedback

1. Is this a product, feature, or open-source project?
2. Who would be the first 100 users?
3. What existing tool is closest to this?
4. What am I missing?

---

# 13. Recommended MVP

## MVP 1: Flaky Test Intelligence

```
Input:
- CI logs
- JUnit/test reports
- test history
- retry history
- commit metadata

Output:
- flaky test detection
- failure grouping
- likely cause explanation
- suggested fix
```

CLI:

```bash
parallax test ingest ./junit.xml
parallax test list --flaky
parallax test explain com.company.CheckoutTest.shouldApplyDiscount
```

## MVP 2: Production Error Intelligence

```
Input:
- Sentry envelope error events
- OpenTelemetry logs/traces/metrics
- deploy metadata

Output:
- issue grouping
- investigation context
- likely root cause
- suggested action
```

CLI:

```bash
parallax issue list
parallax issue explain ISSUE-123
parallax context build ISSUE-123 --window 30m
```

## MVP 3: Agent Integration

```
Input:
- issue/test failure

Output:
- structured context for AI agent
- Linear/GitHub issue
- suggested PR prompt
```

---

# 14. Final Thesis

Project Parallax is based on three observations:

```
1. Observability data is increasingly unified.
2. Debugging workflows are still fragmented and manual.
3. AI agents need structured context to be useful.
```

Therefore:

> **The next valuable layer is not another dashboard. It is an intelligence layer that turns telemetry and CI data into evidence-backed explanations.**
> 

Recommended starting point:

> **Flaky test investigation first. Production incident debugging second.**
> 

Recommended distribution:

> **Open source core + hosted cloud + enterprise support.**
> 

Recommended interface:

> **CLI first. UI later.**
> 

---

# 15. One-Sentence Pitch

> **Project Parallax is an open-source, CLI-first AI debugging system that explains flaky tests and production failures by correlating errors, logs, traces, metrics, CI data, and code changes.**
> 

---

# 16. Short Investor-Oriented Pitch

Engineering teams already collect massive observability and CI data, but debugging still requires manual investigation across fragmented tools. Sentry shows errors, Grafana shows metrics, Kibana shows logs, Jaeger shows traces, and CI shows failures — but no system reliably explains why the failure happened.

Project Parallax is an open-source, AI-native debugging engine that converts fragmented engineering signals into evidence-backed explanations. It starts with flaky test investigation, a frequent and measurable pain validated by companies like Google and Uber, then expands into production error and incident investigation.

The long-term opportunity is to become the intelligence layer between telemetry, CI, issue trackers, and AI coding agents.

---

# 17. Sources

## Observability and incident debugging

- [Google SRE Book — Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/)
- [Google SRE Workbook — Monitoring](https://sre.google/workbook/monitoring/)
- [ACM Queue / Google — Debugging Incidents in Google’s Distributed Systems](https://queue.acm.org/detail.cfm?id=3404974)

## Self-hosted Sentry complexity

- [Sentry Developer Docs — Self-hosted Data Flow](https://develop.sentry.dev/self-hosted/data-flow/)
- [Sentry Snuba Architecture Overview](https://getsentry.github.io/snuba/architecture/overview.html)
- [Bugsink — Why I gave up on self-hosted Sentry](https://www.bugsink.com/blog/why-i-gave-up-on-self-hosted-sentry/)
- [Hacker News — I gave up on self-hosted Sentry](https://news.ycombinator.com/item?id=43725815)
- [Self-Hosting Sentry: What I Thought Would Take an Hour and Why It Didn’t](https://medium.com/%40manohays/self-hosting-sentry-what-i-thought-would-take-an-hour-and-why-it-didnt-e3355eaa8129)

## Flaky tests

- [Google Testing Blog — Flaky Tests at Google and How We Mitigate Them](https://testing.googleblog.com/2016/05/flaky-tests-at-google-and-how-we.html)
- [Google Testing Blog — Test Flakiness: One of the Main Challenges of Automated Testing](https://testing.googleblog.com/2020/12/test-flakiness-one-of-main-challenges.html)
- [Google Research — DeFlake Your Tests](https://research.google/pubs/de-flake-your-tests-automatically-locating-root-causes-of-flaky-tests-in-code-at-google/)
- [Uber Engineering — Handling Flaky Unit Tests in Java](https://www.uber.com/us/en/blog/handling-flaky-tests-java/)
- [Uber Engineering — Flaky Tests Overhaul at Uber](https://www.uber.com/us/en/blog/flaky-tests-overhaul/)

## OpenTelemetry and unified observability

- [OpenTelemetry — Signals](https://opentelemetry.io/docs/concepts/signals/)
- [OpenTelemetry Docs](https://opentelemetry.io/docs/)
- [OpenTelemetry Logs Specification](https://opentelemetry.io/docs/specs/otel/logs/)
- [GreptimeDB GitHub](https://github.com/GreptimeTeam/greptimedb)
- [GreptimeDB Docs — OpenTelemetry Protocol](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/)
- [Greptime — OpenTelemetry Traces Integration with GreptimeDB](https://greptime.com/tech-content/2025-07-17-opentelemetry-traces-integration-with-greptimedb)

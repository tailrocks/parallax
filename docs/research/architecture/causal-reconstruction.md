# Causal Reconstruction and Agent Safety

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Executive Summary

Parallax should not promise that telemetry can always prove "the root cause."
That is too strong. The defensible claim is narrower and more useful:

> Parallax can reconstruct an evidence-backed lifecycle around a failure and rank
> causal hypotheses with explicit confidence, raw evidence, and missing-data
> caveats.

Exact lifecycle reconstruction is achievable for a single request or workflow
when trace context, logs, spans, errors, release metadata, and key business
attributes are present. Full causal reconstruction across a distributed system
is only partial unless the system has dependency topology, change history,
complete instrumentation, known async links, and enough counterfactual evidence
to separate causes from symptoms.

The proof gate for whether those strong edges are common enough in real telemetry
is
[Correlation reliability on real telemetry gate](../capture/correlation.md).

The product implication:

- build deterministic context assembly first;
- represent evidence and causal claims as a graph with typed edges and
  confidence;
- use AI to summarize, plan, and propose fixes only after deterministic evidence
  retrieval;
- gate autonomous actions by evidence strength and blast radius;
- expose read-only MCP/API context first, then PR creation, and never direct
  production mutation as an early feature.

## The Core Answer

### Is "How Did We Get Here?" Achievable?

Yes, but only in layers.

| Reconstruction target | Achievability | What proves it |
| --- | --- | --- |
| Single traced request lifecycle | High | Complete parent/child spans, span links, error event, correlated logs, service/resource attributes. |
| Async workflow lifecycle | Medium-high | Span links or explicit message IDs across queue boundaries; otherwise only temporal inference. |
| Error grouping lifecycle across releases | High | Stable fingerprint, release/version, deploy timestamps, first-seen/last-seen, regression window. |
| CI failure lifecycle | High | Run/job/step/test IDs, JUnit or test JSON, logs, retries, commit metadata. |
| Service dependency blast radius | Medium | Topology graph, trace edges, metrics by service, downstream error propagation. |
| True incident root cause | Medium to low | Requires topology, change data, historical baselines, and enough evidence to rule out alternatives. |
| Business-level cause | Low without custom events | Telemetry often lacks product intent, user journey, data invariants, and business rules. |

The hard boundary: logs, metrics, traces, and errors show what was observed.
They do not automatically prove why it happened. Causality needs structure:
dependency topology, happened-before relations, change events, known ownership,
and sometimes direct inspection of code, database state, config, or infrastructure
state.

### What Is Naive?

The naive version of the thesis is:

> If we store all telemetry, an AI can infer the full root cause.

That fails for several reasons:

- telemetry is sampled, incomplete, and biased toward what engineers chose to
  instrument;
- clocks drift and temporal order is not enough for causality;
- symptoms often appear before the root cause is visible;
- async systems break parent/child request trees unless message links are
  propagated;
- retries, queues, caches, load balancers, and circuit breakers hide causal
  paths;
- metrics identify abnormal dimensions but often lack the code-level or
  data-level reason;
- logs may contain secrets or user data and cannot be dumped wholesale into an
  agent;
- production data access can be necessary, but it creates the largest privacy and
  blast-radius risk.

The realistic thesis is:

> The value is not omniscient RCA. The value is fast, bounded, reproducible
> evidence assembly plus hypothesis validation.

That is enough to be product-grade if the system is honest about confidence and
missing evidence.

## Evidence From Existing Systems

The industry pattern is converging around the same architecture:

1. collect telemetry and topology;
2. build a dependency or evidence graph;
3. form hypotheses;
4. query more data;
5. return a conclusion only when enough evidence exists;
6. show an investigation trail.

Examples:

- Datadog Bits AI SRE describes a loop of observation, reasoning, and action:
  form hypotheses, query telemetry, validate or invalidate them, and return an
  evidence-backed conclusion or mark the investigation inconclusive.
- Dynatrace explicitly says time correlation alone is insufficient and relies on
  topology, transaction, and code-level information across dependent components.
- New Relic iRCA positions topology graphs, causal models, and path-based
  ranking as the way to distinguish symptoms from causes.
- Azure SRE Agent describes hypothesis-driven investigation with a full evidence
  chain rather than random log searching.
- Microsoft RCACopilot reports up to 0.766 RCA accuracy on a real cloud-incident
  dataset, which is promising but also a warning: even a purpose-built system
  over a year of internal incident data is not perfect.

Sources:

- [Datadog Bits AI SRE investigation docs](https://docs.datadoghq.com/bits_ai/bits_ai_sre/investigate_issues/)
- [Dynatrace root cause analysis concepts](https://docs.dynatrace.com/docs/dynatrace-intelligence/root-cause-analysis/concepts)
- [New Relic iRCA announcement](https://newrelic.com/blog/ai/intelligent-rca-accurately-pinpoints-root-cause-in-seconds)
- [Azure SRE Agent RCA docs](https://learn.microsoft.com/en-us/azure/sre-agent/root-cause-analysis)
- [Microsoft Research RCACopilot](https://www.microsoft.com/en-us/research/publication/automatic-root-cause-analysis-via-large-language-models-for-cloud-incidents/)

This validates the direction but not the moat. Incumbents are already building
agentic RCA over their own telemetry data gravity. Parallax needs a sharper
differentiator: open, self-hosted, Rust-first evidence graph and agent context,
not a generic RCA chatbot.

## What OpenTelemetry Gives Us

OpenTelemetry is the right base protocol because it already gives Parallax the
raw ingredients for lifecycle reconstruction:

- traces as directed acyclic graphs of spans;
- parent/child span relationships;
- span links for causally related spans, including async and batch work;
- trace IDs and span IDs;
- logs with trace context fields;
- resource and service attributes such as service name, version, deployment
  environment, host, container, and Kubernetes metadata;
- semantic conventions for common operations.

Sources:

- [OpenTelemetry specification overview](https://opentelemetry.io/docs/specs/otel/overview/)
- [OpenTelemetry traces](https://opentelemetry.io/docs/concepts/signals/traces/)
- [OpenTelemetry trace context in logs](https://opentelemetry.io/docs/specs/otel/compatibility/logging_trace_context/)
- [OpenTelemetry resource semantic conventions](https://opentelemetry.io/docs/specs/semconv/resource/)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)

The crucial OpenTelemetry point: trace structure is not just a convenient join
key. It is the strongest deterministic evidence Parallax has. A parent/child
span edge is much stronger than "same time window." A span link across queue
processing is much stronger than "same message text." Logs with `trace_id` and
`span_id` are much stronger than free-text log search.

### OpenTelemetry Is Not Enough

OpenTelemetry does not automatically provide:

- Sentry-grade issue grouping;
- stacktrace normalization and symbolication;
- deploy/change context;
- code ownership;
- business invariants;
- database state;
- incident memory;
- safe redaction policy;
- agent-ready evidence ranking;
- a durable explanation of which facts support which conclusion.

Those are the layers above OTEL where Parallax can still create value.

## Evidence Graph Design

Parallax should store an explicit evidence graph rather than only rows in
observability tables. The graph can be materialized on demand at first; it does
not need a graph database in the MVP.

### Node Types

| Node | Examples |
| --- | --- |
| Error event | Sentry-style event, panic, exception, normalized stacktrace. |
| Span | OTLP span with trace ID, span ID, parent ID, links, duration, status. |
| Log record | Structured log with trace/span IDs and redacted attributes. |
| Metric window | Error rate, p95 latency, saturation, queue depth, DB connections. |
| Release/deploy | Version, commit SHA, environment, rollout time, deploy actor. |
| Code change | Commit, PR, changed file, owner, diff summary. |
| Runtime resource | Service, host, pod, container, region, dependency, queue, database. |
| CI/test event | Run, job, step, test case, retry, failure signature. |
| Agent action | Tool call, evidence query, PR draft, test command, reasoning summary. |

### Edge Types

| Edge | Strength | Meaning |
| --- | --- | --- |
| `span_parent` | Strong | Child operation happened inside parent operation. |
| `span_link` | Strong | Async/batch operation is causally related to another span. |
| `log_in_span` | Strong | Log carries matching trace/span context. |
| `error_in_span` | Strong | Error event carries trace/span context or exact request context. |
| `same_fingerprint` | Strong | Events share deterministic grouping key. |
| `same_release_regression` | Medium | Failure first appears after release/change boundary. |
| `depends_on` | Medium | Topology or traces show one service calls another. |
| `metric_anomaly_on_path` | Medium | Metric anomaly appears on a dependency path relevant to the failure. |
| `same_time_window` | Weak | Events are temporally nearby only. |
| `semantic_similarity` | Weak | LLM or text match suggests relatedness without structural evidence. |
| `human_confirmed` | Strong after review | Human accepted the causal relationship or fix. |

Each edge must carry:

- source system;
- source record IDs;
- timestamp range;
- confidence;
- whether the relationship is deterministic, inferred, or human-confirmed;
- redaction status;
- raw evidence references.

### Why This Matters

An agent should not receive one flattened blob of logs. It should receive:

1. the anchor failure;
2. the strongest deterministic neighbors first;
3. weaker hypotheses separately labeled;
4. missing-data warnings;
5. direct links to raw evidence.

This prevents a model from confusing correlation with causation and makes its
answer auditable.

## Reconstruction Pipeline

The causal/lifecycle reconstruction layer should run after ingestion,
normalization, grouping, and storage:

```text
error/test/alert anchor
  -> deterministic neighborhood query
  -> trace/log/span/error stitching
  -> deploy/change window lookup
  -> metric anomaly and baseline comparison
  -> topology path expansion
  -> candidate cause generation
  -> evidence scoring and contradiction search
  -> bounded agent context bundle
  -> PR proposal or investigation report
```

### Step 1: Anchor The Investigation

Every investigation needs a specific anchor:

- error event ID;
- issue/group ID;
- trace ID;
- failing test case;
- CI run/job/step;
- alert/monitor transition.

Without an anchor, the system becomes a generic anomaly search and the agent will
overfit on noise.

### Step 2: Build The Deterministic Neighborhood

Fetch:

- same trace;
- same span and child spans;
- logs with matching trace/span IDs;
- same error fingerprint in a bounded recent window;
- release/deploy covering the timestamp;
- service/resource metadata;
- nearby CI/deploy/change events.

This is the high-confidence context.

### Step 3: Expand Along Topology

Only after deterministic evidence is assembled should Parallax expand to:

- upstream callers;
- downstream dependencies;
- database/queue/cache dependencies;
- service owners;
- host/pod/container neighbors;
- region/zone;
- error and latency changes on the dependency path.

The topology path should constrain the search. If every metric in the company is
eligible evidence, the agent will find false correlations.

### Step 4: Score Candidate Causes

Candidate causes should be ranked by evidence class:

| Evidence | Weight |
| --- | --- |
| Error stack points directly to changed code | Very high |
| Failing span/log has exact trace correlation | Very high |
| Failure starts immediately after deploy and rollback fixes it | High |
| Dependency on abnormal downstream service is on trace path | High |
| Metric anomaly on dependent resource during trace window | Medium |
| Same error seen before with known fix | Medium |
| Time-only correlation | Low |
| LLM semantic similarity only | Very low |

Contradictions should be first-class:

- deployment happened after the first failure;
- same failure occurs on old release;
- downstream service is not on the trace path;
- metric anomaly affects unrelated region;
- logs show retry success before user-visible failure;
- missing trace coverage prevents confident conclusion.

### Step 5: Produce A Bounded Bundle

The agent-facing bundle should include:

```json
{
  "anchor": {},
  "confidence": "medium",
  "hypotheses": [
    {
      "claim": "Recent deploy likely introduced panic in checkout discount path",
      "confidence": "high",
      "supporting_edges": ["error_in_span", "same_release_regression"],
      "contradictions": [],
      "missing_evidence": ["no database query plan captured"]
    }
  ],
  "timeline": [],
  "strong_evidence": [],
  "weak_evidence": [],
  "raw_refs": [],
  "redaction": {}
}
```

Do not hide uncertainty. The difference between a trusted Parallax report and a
bad AI RCA report is that Parallax says "inconclusive" when the graph does not
support a conclusion.

## Agent Autonomy Model

Parallax should separate investigation from action. The safer, more useful
sequence is:

| Level | Capability | Default |
| --- | --- | --- |
| 0 | Read-only evidence bundle | Always allowed. |
| 1 | Suggested next queries/checks | Allowed. |
| 2 | Draft diagnosis with confidence and citations | Allowed. |
| 3 | Draft PR proposal without pushing branch | Allowed for connected repos. |
| 4 | Open PR with code changes and tests | Allowed only with repo-scoped permissions and policy checks. |
| 5 | Execute production actions | Not an MVP feature. Human gate required. |
| 6 | Direct database mutation or deploy rollback | Explicitly out of scope until a separate safety system exists. |

The original vision that an agent can open a correct PR is realistic in some
cases:

| Failure class | Autonomous PR realism |
| --- | --- |
| Panic/exception with clear stack, changed code, and test reproduction | High |
| CI/test failure with deterministic failure and local reproducer | High |
| Missing validation, obvious nil/None/null path, type or compile error | High |
| Config mismatch visible in code/repo | Medium |
| Dependency timeout, DB saturation, infra capacity | Medium-low |
| Data corruption, privacy issue, multi-service incident | Low |
| Unknown production-only race | Low unless traces and logs capture the invariant break. |

The agent should open either:

- a direct fix PR when evidence is strong and tests validate it; or
- a proposal PR/issue with candidate fixes when evidence is incomplete.

That aligns with the prompt's end state without pretending telemetry alone can
solve every incident.

## Database and Production Data Access

Giving an agent database access is the most dangerous part of the vision. It may
also be necessary for hard bugs. The design should make database access a
controlled evidence source, not a general shell.

Recommended policy:

| Control | Requirement |
| --- | --- |
| Read-only by default | No writes, migrations, deletes, or DDL through the agent context path. |
| Query templates first | Prefer approved diagnostic queries over free-form SQL. |
| Row/column redaction | Mask secrets, tokens, emails, IPs, payment data, and user content by policy. |
| Sampling and limits | Hard row count, byte count, time range, and query duration limits. |
| Environment boundaries | Production access requires explicit project/environment scope. |
| Just-in-time grants | Short-lived credentials tied to one investigation. |
| Audit log | Record who/what requested which query, why, and what redaction occurred. |
| Explainable output | Store query purpose and result summary, not only raw rows. |

The agent should see summaries and bounded result sets unless a human explicitly
opens a wider view. The focused gate for this boundary is
[Production database evidence access gate](../capture/production-db-evidence.md),
and the measurable claim contract is the
[Production database evidence ledger](../capture/production-db-evidence.md).

## MCP and API Safety

MCP is the right agent integration surface, but it expands blast radius if it
becomes a generic production-control plane. The MCP spec now has authorization
and security guidance, and OpenTelemetry has MCP semantic conventions, which
means Parallax can make the agent surface observable from day one. The focused
CLI/API/MCP decision is captured in
[Agent access surface: CLI, HTTP API, and MCP](../decisions/agent-access-surface.md).

Sources:

- [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)
- [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)
- [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/)

Recommended MCP tools:

| Tool | Permission | Notes |
| --- | --- | --- |
| `parallax_issue_list` | Read | Scoped by project/environment. |
| `parallax_issue_show` | Read | Normalized issue and latest events. |
| `parallax_issue_context` | Read | Bounded evidence bundle. |
| `parallax_event_raw` | Read-sensitive | Requires raw-event scope; redacted by default. |
| `parallax_trace_context` | Read | Trace, span waterfall, logs, metric deltas. |
| `parallax_hypothesis_check` | Read | Runs deterministic checks for one hypothesis. |
| `parallax_pr_proposal` | Write-repo-proposal | Creates patch text or draft branch only. |

Do not expose generic tools like:

- `run_sql`;
- `kubectl`;
- `ssh`;
- `run_shell`;
- `deploy`;
- `rollback`;
- `update_secret`;
- `delete_data`.

Those can exist later behind a different policy engine, but they should not be
part of the core Parallax context server.

### Security Rules

| Risk | Control |
| --- | --- |
| Prompt injection from logs/issues | Treat all telemetry and issue text as untrusted data; never let it redefine tool policy. |
| Sensitive data leakage | Redact before storage where possible and before agent output always. |
| Excessive agency | Least-privilege scopes per tool; no omnibus token. |
| Confused deputy | Per-client consent and scoped authorization for MCP clients. |
| Audit gaps | Every tool call emits an audit event and OpenTelemetry span. |
| Tool result poisoning | Label source trust and evidence strength in every bundle. |
| Cost explosion | Bound context size, query windows, token budgets, and repeated tool loops. |

The OWASP LLM Top 10 reinforces these controls: prompt injection, sensitive
information disclosure, excessive agency, and overreliance are direct risks for
this product.

Source:

- [OWASP Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/)

## Product Moat Implications

The moat is not "AI explains logs." That is already commodity.

The possible moat is:

1. a high-quality evidence graph;
2. deterministic grouping and correlation;
3. code/repo/deploy/runtime context in one bundle;
4. local/self-hosted trust for sensitive debugging data;
5. agent integrations that are safe, auditable, and useful;
6. historical debugging datasets created by accepted/rejected hypotheses and
   confirmed fixes.

The evidence graph is the compounding asset. Every investigation can teach:

- which edges mattered;
- which hypothesis was accepted;
- which fix worked;
- which signals were missing;
- which queries were useful;
- which agent actions were safe.

That becomes a debugging dataset. Incumbents have broader telemetry data gravity,
but Parallax can have better open evidence packaging and agent workflow if it is
deliberate.

## Implementation Concept

### Component Placement

```text
Ingest gateway
  -> raw append-only log
  -> normalizers
  -> grouping worker
  -> symbolication worker
  -> storage writer
  -> evidence graph builder
  -> hypothesis engine
  -> context API / MCP server
  -> agent PR workflow
```

### Storage

- Store raw telemetry and normalized observability records in GreptimeDB or
  ClickHouse, per the storage benchmark.
- Store low-volume issue/project metadata in Turso.
- Store graph edges either as tables first or as materialized JSON bundles.
- Avoid a graph database until query patterns prove it is necessary.

### First Evidence Graph Tables

```text
evidence_nodes(
  node_id,
  project_id,
  node_type,
  source,
  source_id,
  timestamp,
  summary,
  raw_ref,
  redaction_level
)

evidence_edges(
  edge_id,
  project_id,
  from_node_id,
  to_node_id,
  edge_type,
  confidence,
  inference_method,
  evidence_refs,
  created_at
)

hypotheses(
  hypothesis_id,
  project_id,
  anchor_node_id,
  claim,
  confidence,
  supporting_edge_ids,
  contradicting_edge_ids,
  missing_evidence,
  status
)
```

This is enough to build context bundles and audit agent reasoning without
choosing a specialized graph engine too early.

## Bottom Line

Parallax's causal reconstruction goal is valid if stated precisely:

- **Strong claim:** reconstruct the request/workflow/error lifecycle when trace
  context and instrumentation are present.
- **Medium claim:** identify likely causal paths using topology, metrics,
  deploys, and historical examples.
- **Weak claim:** automatically prove true root cause for arbitrary production
  incidents.

The product should optimize for the strong and medium claims. That means
deterministic evidence first, AI second, safe autonomy third. The architecture
should be built around a typed evidence graph and bounded agent context, not
around a dashboard and not around an untrusted LLM reading unbounded telemetry.

Related proof gate:

- [Correlation reliability on real telemetry gate](../capture/correlation.md)
  — A4 measurement plan for strong edges, false strong edges, frontend
  continuation, async links, and missing-evidence reporting.

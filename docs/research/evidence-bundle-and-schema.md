# Evidence Bundle and Open Schema Specification

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This document specifies the **Parallax evidence bundle** — the portable,
versioned, machine-readable object that Parallax serves to humans and coding
agents — and the **open evidence schema** behind it. The
[Go / No-Go verdict](verdict.md) names the open evidence schema and the portable
bundle format as the primary defensible moat: a feature can be copied in a
release, but a schema that other tools and agents build against becomes leverage
that compounds. This is the concrete spec that turns that claim into something
buildable.

It elevates the data-model tables scattered across
[Technical implementation concept](technical-implementation-concept.md) (error
event model, CLI invocation row, agent session row, audit graph edges,
correlation layers) into one coherent, externally consumable contract. The
implementation concept says *what Parallax stores*; this document says *what
Parallax hands back* and *how that contract stays stable as the product grows*.

Version freshness rule: this is a `v0` schema draft as of the research date. It
is intended to be implemented, benchmarked, and revised — not frozen. Every field
here is a proposal to validate against real Sentry/OTLP/CI/agent data.

## Why The Bundle Is The Unit, Not The Query

Incumbents (Datadog, Sentry, Grafana, New Relic) keep investigations inside their
cloud and expose query APIs. Open competitors (SigNoz, OpenObserve) expose query
tools and MCP servers over raw telemetry. Both make the agent do the assembly.

Parallax's bet is that the valuable unit is not a query result but a **bounded,
self-contained dossier**: everything connected to one anchor (an issue, an event,
a trace, a CI failure, a CLI invocation, or an agent session), pre-correlated,
redacted, with evidence strengths and missing-data flags already computed. The
bundle is:

- **portable** — a single JSON (or Markdown rendering, or ZIP with attachments)
  that can be attached to a GitHub issue, pasted into Claude Code / Codex /
  Cursor, archived, or diffed across time;
- **deterministic** — assembled by typed correlation logic before any LLM
  touches it, so the same anchor and window produce the same bundle;
- **citable** — every claim carries a reference to raw evidence the agent or a
  human can pull;
- **safe** — redaction happens at bundle-build time, not at the agent's
  discretion;
- **honest** — it states what evidence is missing and how confident each edge is,
  instead of presenting a smooth narrative.

This is what existing agent-observability tools (LangSmith, Langfuse, Phoenix)
stop short of: they trace the LLM application, not the full chain from production
error to deploy to CLI side effect to coding-agent patch to outcome.

## Design Principles

1. **Stable IDs everywhere.** Every node has a globally stable ID so bundles can
   reference each other and agents can re-fetch. IDs are opaque strings.
2. **References over dumps.** The bundle carries bounded excerpts plus a `ref`
   to the full raw evidence behind Parallax's access policy. It never inlines
   unbounded logs or full production payloads.
3. **Redaction is mandatory and reported.** Every bundle includes a
   `redaction_report`. A bundle with no redaction report is invalid.
4. **Deterministic before probabilistic.** Nodes, edges, and edge strengths are
   computed by typed logic. The `hypotheses` block is the only place model-ranked
   or inferred content appears, and it must cite deterministic evidence.
5. **Confidence and absence are first-class.** `missing_evidence` and per-edge
   `strength` are required structural fields, not optional metadata.
6. **Versioned and additive.** `schema_version` is mandatory. The schema evolves
   by adding optional fields and new node/edge types, never by silently changing
   the meaning of an existing field. See [Versioning](#versioning-and-compatibility).
7. **Signal-agnostic envelope, typed nodes.** The envelope is the same whether the
   anchor is an error, a trace, a CI run, or an agent session. Specialization
   lives in node `type` and `data`.

## Bundle Envelope

The top-level object:

```json
{
  "schema_version": "0.1.0",
  "bundle_id": "bndl_01J9...",
  "generated_at": "2026-05-25T14:03:11.482Z",
  "generator": { "name": "parallax", "version": "0.1.0", "grouping_algo": "rust-stack-v1" },
  "project": { "id": "proj_checkout", "environment": "production" },
  "anchor": { "type": "issue", "id": "iss_8b21" },
  "window": { "from": "2026-05-25T13:58:00Z", "to": "2026-05-25T14:03:00Z", "rationale": "anchor event +/- 5m" },
  "nodes": [ /* typed evidence nodes */ ],
  "edges": [ /* typed evidence edges */ ],
  "hypotheses": [ /* ranked, evidence-cited */ ],
  "missing_evidence": [ /* explicit gaps */ ],
  "redaction_report": { /* what was removed and why */ },
  "query_manifest": [ /* reproducible queries that built this bundle */ ],
  "access": { "raw_access_policy": "scoped-read", "expires_at": "2026-05-25T15:03:11Z" }
}
```

Required top-level fields: `schema_version`, `bundle_id`, `generated_at`,
`generator`, `project`, `anchor`, `nodes`, `edges`, `missing_evidence`,
`redaction_report`. `hypotheses`, `window`, `query_manifest`, and `access` are
recommended but may be empty/absent for the smallest bundles.

## Node Type Catalog

Every node: `{ "id", "type", "ts" (when applicable), "summary", "data", "refs" }`.
`summary` is a short human/agent-readable line; `data` is the typed payload;
`refs` point to raw evidence.

| Node `type` | Purpose | Key `data` fields (v0) |
| --- | --- | --- |
| `issue` | Grouped error identity. | `fingerprint`, `grouping_algo`, `title`, `first_seen`, `last_seen`, `event_count`, `status`, `first_seen_release` |
| `error_event` | One Sentry-style error occurrence. | `error_type`, `message`, `level`, `handled`, `mechanism`, `panic_location`, `stack` (frames), `error_chain`, `trace_id`, `span_id`, `release`, `environment`, `sdk`, `runtime` |
| `span` | One OTLP span. | `name`, `trace_id`, `span_id`, `parent_span_id`, `start`, `end`, `duration_ms`, `status`, `attributes`, `service` |
| `log_window` | Bounded log slice. | `trace_id?`, `span_id?`, `count`, `levels`, `excerpts[]`, `time_range` |
| `metric_window` | Metric series slice / anomaly. | `metric`, `labels`, `baseline`, `observed`, `delta`, `anomaly`, `time_range` |
| `release` | Release/version marker. | `version`, `commit_sha`, `published_at`, `predecessor_version` |
| `deploy` | Deploy/change event. | `deploy_id`, `release`, `started_at`, `finished_at`, `actor`, `targets` |
| `code_change` | Commit / PR / diff region. | `commit_sha`, `pr_url?`, `files[]`, `authors`, `changed_symbols?` |
| `ci_run` | CI pipeline execution. | `provider`, `run_id`, `workflow`, `status`, `commit_sha`, `branch`, `started_at`, `jobs[]` |
| `test_case` | One test and its history. | `suite`, `name`, `status`, `retries`, `pass_fail_history`, `first_failed_commit?`, `flaky_score?` |
| `cli_invocation` | First-class CLI execution. | `command`, `subcommand`, `args_sanitized`, `cwd`, `repo`, `branch`, `commit`, `exit_code`, `duration_ms`, `stdout_ref`, `stderr_ref`, `child_processes[]`, `side_effects[]` |
| `agent_session` | Coding-agent run. | `agent_product`, `started_at`, `ended_at`, `status`, `prompt_refs`, `bundles_used[]`, `outcome` |
| `agent_action` | One action inside a session. | `kind` (model_call/tool_call/shell/file_edit/test/pr/approval), `target`, `result_ref`, `ts`, `confidence?` |
| `hypothesis` | A candidate cause (see below). | `statement`, `confidence`, `supporting[]`, `contradicting[]`, `checks[]` |

Frame shape inside `error_event.stack` (oldest→newest), matching the Rust-first
capture story in
[Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md):

```json
{ "crate": "checkout", "module": "discount", "function": "apply",
  "file": "src/discount.rs", "line": 118, "in_app": true, "build_id": "a1b2..." }
```

## Edge Type Catalog

Every edge: `{ "from", "to", "type", "strength", "evidence?" }`. `strength` is one
of `strong | medium | weak | inferred`, mapping to the correlation layers in the
implementation concept. Deterministic joins are `strong`; topology/regression
windows are `medium`; time/semantic proximity is `weak`; model-proposed links are
`inferred` and must appear only via a `hypothesis` node.

Correlation edges:

| Edge `type` | Meaning | Typical strength |
| --- | --- | --- |
| `error_in_span` | Error event occurred inside a span. | strong (shared `span_id`) |
| `log_in_span` | Log line belongs to a span. | strong |
| `span_child_of` | Span parent/child. | strong |
| `same_fingerprint` | Event belongs to issue. | strong |
| `same_release_regression` | Issue appeared/spiked at a release boundary. | medium |
| `metric_anomaly_on_path` | Metric anomaly on the trace's service path. | medium |
| `deploy_preceded_issue` | Deploy in window before first occurrence. | medium |
| `code_change_touched_frame` | Changed file/symbol matches a top in-app frame. | medium |
| `temporal_proximity` | Co-occurred in window, no stronger link. | weak |

Audit edges (the agent/CLI accountability layer):

| Edge `type` | Meaning |
| --- | --- |
| `agent_used_bundle` | Session consumed a specific bundle. |
| `agent_ran_command` | Session invoked a CLI/shell command. |
| `command_spawned_process` | CLI command spawned a child / CI step. |
| `command_touched_resource` | Command touched a file, DB object, queue, deploy target, or API. |
| `agent_changed_file` | Agent produced a patch touching a file. |
| `agent_opened_pr` | Agent/fixer opened a PR from a patch. |
| `validation_checked_patch` | Test/build/lint validated or rejected a patch. |
| `fix_addressed_issue` | Outcome/recurrence linked a fix to an issue. |
| `fix_worsened_issue` | Revert/recurrence linked a bad outcome to prior action. |

These edges are what let Parallax answer the prompt's audit questions — "which
command touched this database object?", "what did the agent do before this
deploy?" — as a graph traversal, not a log scan.

## Hypothesis And Confidence Model

`hypotheses` is the only place ranked/inferred reasoning lives, and it is
structurally forced to cite deterministic evidence:

```json
{
  "id": "hyp_1",
  "statement": "Empty discount rule set causes unwrap panic in apply().",
  "confidence": 0.72,
  "supporting": ["span_db_lookup", "evt_panic", "code_change_discount"],
  "contradicting": [],
  "checks": [
    { "name": "empty_ruleset_before_panic", "result": "pass", "ref": "span_db_lookup" },
    { "name": "regression_in_prior_release", "result": "absent" }
  ],
  "recommended_next": "Guard empty rule set; add regression test for empty discount config."
}
```

Rules:

- `confidence` is advisory, never a gate for autonomous action.
- A hypothesis with empty `supporting` is invalid — no uncited claims.
- `checks` are deterministic verifications Parallax ran or can run; `absent`
  means the check could not be evaluated (feeds `missing_evidence`).
- The bundle must be able to carry **zero hypotheses** and say so. "Inconclusive
  with evidence" is a valid, first-class output.

## Redaction Report

Mandatory. Shape:

```json
{
  "policy_version": "redact-v1",
  "rules_applied": ["secret-detector", "pii-email", "auth-header-strip"],
  "removed": [
    { "node": "cli_invocation_1", "field": "args_sanitized", "rule": "secret-detector", "count": 1 },
    { "node": "log_window_1", "field": "excerpts", "rule": "pii-email", "count": 3 }
  ],
  "raw_access_policy": "scoped-read",
  "residual_risk": "low"
}
```

Redaction at build time is what makes the bundle safe to hand to an agent or
paste into a third-party model. CLI invocation args/env and log excerpts are the
highest-risk fields (see
[Agent and CLI execution tracing](agent-and-cli-execution-tracing.md)). The
trust model and red-team gate for this object are specified in
[Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md).

## Worked Example (Abbreviated)

The deterministic-context proof point from the implementation concept, as a real
bundle fragment:

```json
{
  "schema_version": "0.1.0",
  "bundle_id": "bndl_checkout_panic_01",
  "anchor": { "type": "issue", "id": "iss_8b21" },
  "nodes": [
    { "id": "iss_8b21", "type": "issue",
      "summary": "panic: called Option::unwrap() on a None value in checkout::discount::apply",
      "data": { "fingerprint": "rust:checkout::discount::apply:src/discount.rs:118",
                "first_seen_release": "2026.05.25-4", "event_count": 14, "status": "unresolved" } },
    { "id": "evt_panic", "type": "error_event", "ts": "2026-05-25T14:02:58.114Z",
      "summary": "unwrap on None at src/discount.rs:118",
      "data": { "error_type": "panic", "handled": false, "trace_id": "4f9c...",
                "span_id": "a17e...", "release": "2026.05.25-4",
                "panic_location": "src/discount.rs:118" },
      "refs": { "raw_event": "evt://proj_checkout/evt_panic" } },
    { "id": "span_apply", "type": "span", "summary": "checkout.apply_discount (33ms)",
      "data": { "trace_id": "4f9c...", "span_id": "a17e...", "duration_ms": 33, "status": "error" } },
    { "id": "span_db_lookup", "type": "span", "summary": "db.lookup discount_rules -> 0 rows",
      "data": { "trace_id": "4f9c...", "parent_span_id": "a17e...", "duration_ms": 4,
                "attributes": { "db.rows_returned": 0 } } },
    { "id": "rel_4", "type": "release", "data": { "version": "2026.05.25-4", "commit_sha": "9d1f..." } }
  ],
  "edges": [
    { "from": "evt_panic", "to": "span_apply", "type": "error_in_span", "strength": "strong" },
    { "from": "span_db_lookup", "to": "span_apply", "type": "span_child_of", "strength": "strong" },
    { "from": "iss_8b21", "to": "rel_4", "type": "same_release_regression", "strength": "medium",
      "evidence": "no occurrences in 2026.05.25-3 window" }
  ],
  "hypotheses": [
    { "id": "hyp_1", "statement": "Empty discount rule set triggers unwrap panic.",
      "confidence": 0.72, "supporting": ["span_db_lookup", "evt_panic"], "contradicting": [],
      "recommended_next": "Guard empty rule set; add regression test." }
  ],
  "missing_evidence": [
    { "what": "metric_window for checkout error rate", "reason": "metrics not ingested for this service" }
  ],
  "redaction_report": { "policy_version": "redact-v1", "rules_applied": ["secret-detector"], "removed": [], "residual_risk": "low" }
}
```

An agent receiving this can map the panic to `src/discount.rs:118`, see the empty
rule set 4 ms earlier in the same trace, see the regression began at release
`2026.05.25-4`, and notice metrics are missing — without scanning raw telemetry.

## Surfaces That Emit The Bundle

The same bundle object is returned by all three agent surfaces (one contract,
three transports), per the CLI-vs-MCP decision in the implementation concept:

- **CLI:** `parallax issue context iss_8b21 --window 10m --format json|markdown`
- **HTTP:** `GET /api/projects/:project/issues/:issue_id/context?window=10m`
- **MCP:** tool `parallax_issue_context` returns the same JSON; the Markdown
  rendering is a deterministic projection of the JSON, never a separate source of
  truth.

## Versioning And Compatibility

The schema is the moat only if external tools can depend on it. Rules:

- `schema_version` is semver. Agents should pin a major version.
- **Additive within a major:** new optional fields, new node `type`s, new edge
  `type`s, new hypothesis `checks`. Consumers must ignore unknown node/edge types
  rather than fail.
- **Breaking only on major bump:** renaming/removing a field, changing a field's
  meaning, or changing `strength` semantics.
- `grouping_algo` and `policy_version` are carried in-band so a bundle is
  self-describing across algorithm changes — a re-grouped issue is auditable.
- Markdown and ZIP renderings are projections of the JSON; the JSON is canonical.

## What Must Be Validated

This is a draft contract. The benchmark and prototype gates that decide whether
it survives contact with real data:

1. Does the bundle improve agent fix/diagnosis quality versus raw Sentry/CI
   context? (The core moat claim — see kill criterion 3 in [verdict](verdict.md).)
2. Can bundles stay bounded (size/token budget) while still carrying enough
   evidence for high-confidence application-error fixes?
3. Is `redaction_report` trustworthy across logs, CLI args/env, attachments, and
   agent prompt material? See the
   [redaction pipeline](redaction-pipeline-and-secret-safety.md) for the
   required default-deny policy and red-team gate.
4. Do `strength` tiers correspond to real predictive value, or are `medium`
   edges noise?
5. Is the schema stable enough that an external tool built on `v0.1` keeps
   working across two minor revisions?

## Relationship To Other Research

- [Technical implementation concept](technical-implementation-concept.md) — the
  storage-side data model this contract is projected from, and the data-flow that
  builds it.
- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  — how edges, confidence, and missing-data feed safe agent behavior.
- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) — the
  source detail for `cli_invocation`, `agent_session`, `agent_action`, and audit
  edges.
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  — the additive frontend node types (`frontend_session`, `user_step`,
  `frontend_error`, `route_view`, `frontend_release`) and cross-tier edges.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  — the source-specific policy and eval gate that makes `redaction_report`
  enforceable rather than decorative.
- [Verdict](verdict.md) — why the open schema and portable bundle are the moat.

## Bottom Line

The evidence bundle is Parallax's product surface and its moat in one artifact: a
bounded, deterministic, redacted, citable, versioned dossier with explicit
confidence and gaps. Build the schema as a public, stable, additive contract from
day one, emit the identical object over CLI/HTTP/MCP, and let adoption of the
format — not any single feature — become the thing competitors cannot copy in a
release.

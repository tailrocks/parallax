# A3 Schema Adoption And Corpus Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md)
defines what A3 must prove:

> The open evidence schema plus failure/fix corpus becomes a compounding moat.

This note defines the missing public ledger for that proof. A3 is not validated
by publishing Markdown, JSON Schema files, or a few friendly comments. It is
validated by durable events: schema releases, conformance runs, external review,
external bundle producers/consumers, compatibility decisions, and labeled
failure/fix outcomes.

No A3 pass, schema-gravity, corpus-growth, or moat claim should count unless the
supporting events are recorded in a committed, source-linked ledger.

## Current Source Posture

Outside sources checked for this pass:

- OpenTelemetry treats stability and versioning as a user-facing contract:
  stable clients should be upgradable across minor versions without breaking
  users, semantic-convention changes should avoid breaking analysis tooling, and
  additive telemetry changes are treated differently from breaking changes
  ([OpenTelemetry versioning and stability](https://opentelemetry.io/docs/specs/otel/versioning-and-stability/)).
- OpenTelemetry's telemetry-stability page is still marked Development and says
  stable instrumentation must clearly label produced telemetry; this is a useful
  warning that Parallax must label schema status and not overstate stability
  before it has consumers
  ([OpenTelemetry telemetry stability](https://opentelemetry.io/docs/specs/otel/telemetry-stability/)).
- JSON Schema Draft 2020-12 gives the machine-validation substrate: the draft
  page names the 2020-12 metaschema, and the core spec defines schema resources,
  core keywords, identifiers, references, annotations, and output structure
  ([JSON Schema Draft 2020-12](https://json-schema.org/draft/2020-12),
  [JSON Schema Core](https://json-schema.org/draft/2020-12/json-schema-core)).
- MCP's current specification says MCP uses JSON Schema for validation, defaults
  to JSON Schema 2020-12 when `$schema` is omitted, and exposes tool
  input/output schemas and structured tool results; this makes Parallax bundle
  schema compatibility relevant to agent tool surfaces
  ([MCP overview](https://modelcontextprotocol.io/specification/2025-11-25/basic),
  [MCP schema reference](https://modelcontextprotocol.io/specification/2025-11-25/schema)).

Internal sources:

- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  current `v0` bundle draft.
- [Risks and the bear case](risks-and-bear-case.md) says A3 is existential
  because the schema/corpus moat fails without adoption.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  starts the schema-adoption clock in Phase 2 and feeds outcome data in Phase 4.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  defines the first outcome-style result ledger that can seed early corpus
  events.
- [A2 interview evidence ledger](a2-interview-evidence-ledger.md) defines the
  design-partner signal that can seed external schema review.

## Why A3 Needs A Ledger

The schema/corpus moat has several false-positive paths:

| Failure mode | How it misleads Parallax | Ledger control |
| --- | --- | --- |
| Schema published, no consumer pain | The repo has a format, not a standard. | Count only validated producer/consumer events and compatibility decisions. |
| Operator-controlled demos counted as adoption | n=1 activity masquerades as ecosystem pull. | Track actor relationship and control level for every event. |
| Friendly feedback counted as dependency | Review is useful, but not adoption. | Separate review events from integration/dependency events. |
| Corpus rows without outcomes | A pile of bundles is not a failure/fix corpus. | Count only rows with action, outcome, and recurrence/review status. |
| Private pilots summarized too vaguely | The public repo cannot audit why A3 passed. | Allow anonymized actors, but require evidence hashes, reviewer, and count rules. |
| Compatibility breakage hidden | A schema that repeatedly breaks consumers is anti-moat. | Record breakage reports, migration decisions, and deprecation windows. |

## Ledger Artifacts

When the schema artifacts exist and the A3 clock starts, add:

```text
docs/research/schema-adoption-results.md
docs/research/corpus-outcome-results.md
```

`schema-adoption-results.md` should contain:

- current schema-release snapshot;
- adoption-clock start date and checkpoint status;
- external review ledger;
- integration attempt ledger;
- conformance run ledger;
- compatibility decision ledger;
- breakage and migration ledger;
- current A3 schema-claim level.

`corpus-outcome-results.md` should contain:

- corpus snapshot by bundle source and outcome type;
- one redacted corpus row per bundle/action/outcome event;
- recurrence/rollback/review status;
- privacy/redaction status;
- current A3 corpus-claim level.

Private evidence can stay outside the repo, but every private event counted in
public totals needs a public hash, reviewer, and anonymized actor class.

## Schema Adoption Event Types

Use these event types in `schema-adoption-results.md`:

| Event type | Counts toward | Required evidence |
| --- | --- | --- |
| `schema_release` | Starts or advances clock | Git tag or commit, schema version, changelog, conformance command, compatibility policy. |
| `external_review` | 30-day feedback target | Link to issue/comment/PR, or private evidence hash; reviewer is non-operator. |
| `producer_integration` | 90/180/365-day adoption targets | External tool/team emits a valid bundle fixture against canonical schema. |
| `consumer_integration` | 90/180/365-day adoption targets | External tool/team consumes a valid bundle and preserves required safety fields. |
| `workflow_dependency` | 180/365-day dependency target | A non-operator workflow would break or lose value if schema compatibility broke. |
| `conformance_run` | Adoption quality | Validator version, schema version, fixture set hash, pass/fail summary. |
| `compatibility_decision` | Compatibility health | Change proposal, affected consumers, decision, migration/deprecation plan. |
| `breakage_report` | Negative A3 signal | Consumer impact, schema version, severity, fix/migration status. |

Review events are not adoption. Integration events are not dependencies until the
external actor uses the schema in a real workflow outside the operator's demo.

## Adoption Event Row Schema

Each schema-adoption event should be recorded as a structured row:

```yaml
event_id: A3-SCHEMA-001
event_date: 2026-05-25
event_type: schema_release | external_review | producer_integration | consumer_integration | workflow_dependency | conformance_run | compatibility_decision | breakage_report
schema_version: 0.1.0
actor_class: operator | design_partner | unrelated_tool | oss_maintainer | customer_team | agent_wrapper | observability_tool | private_partner
actor_relationship: operator_controlled | warm_design_partner | external_non_operator | unrelated_public | private_non_operator
public_evidence:
  url: https://example.invalid/link-or-null
  hash: sha256:...
private_evidence_available: false
validator_version: parallax-schema 0.1.0
fixture_hash: sha256:...
counts_toward:
  day_30_review: false
  day_90_integration: false
  day_180_dependency: false
  day_365_dependency: false
summary: "Short public summary with no secrets."
blockers:
  - "Missing redaction_report in fixture."
decision: count | do_not_count | negative_signal
reviewer: operator | second_reviewer | external_reviewer
```

An event must be marked `do_not_count` when evidence is missing, the actor is
operator-controlled, the fixture cannot validate, or the integration is only a
demo scripted entirely by Parallax.

## Corpus Outcome Event Types

The corpus only matters when bundles are tied to actions and outcomes:

| Event type | Required evidence | Counts toward corpus? |
| --- | --- | --- |
| `bundle_presented` | Bundle id/version, anchor, actor, redaction policy, token/size budget. | No by itself. |
| `action_taken` | Agent/human/tool action, evidence refs cited, unsupported claims. | No by itself. |
| `patch_or_proposal` | Patch/PR/proposal ref, files touched, tests run, risk class. | No by itself. |
| `outcome_labeled` | Accepted/rejected/inconclusive/needs-human, reviewer or verifier, rationale. | Yes. |
| `recurrence_checked` | No recurrence/recurrence/rollback/revert/worsened, time window. | Strengthens corpus quality. |
| `regression_feedback` | Follow-up issue, incident, revert, or review comment. | Negative or positive corpus signal. |

The minimum counted corpus unit is `outcome_labeled`. A bundle without an
outcome is useful operational evidence but not failure/fix corpus data.

## Corpus Row Schema

Use this row shape in `corpus-outcome-results.md`:

```yaml
corpus_event_id: A3-CORPUS-001
bundle_id: bndl_...
schema_version: 0.1.0
bundle_source: phase0_eval | tiny_mvp_pilot | production_incident | ci_failure | agent_session | synthetic_fault | private_partner
telemetry_provenance:
  - observed_from_sdk
  - observed_from_harness
redaction_policy_version: redact-v1
actor_type: human | coding_agent | fixer_component | external_tool
model_or_tool: "exact model/tool id if applicable"
action_type: diagnosis | patch | pr | rollback | no_action | escalation
evidence_refs_cited: 0
unsupported_claim_count: 0
patch_or_proposal_ref:
  url: null
  hash: sha256:...
outcome: accepted | rejected | inconclusive | needs_human | unsafe | no_action
verifier: tests | reviewer | recurrence_window | manual_spot_check | unknown
recurrence_status: not_checked | no_recurrence | recurred | reverted | worsened
recurrence_window_days: 0
counts_toward:
  phase0_manual_eval: false
  tiny_mvp_25: false
  public_100: false
  moat_1000: false
privacy_level: public | anonymized | private_hash_only
reviewer: operator | second_reviewer | external_reviewer
summary: "Short public summary."
```

Corpus rows with `verifier: unknown`, missing `redaction_policy_version`, or
`privacy_level: private_hash_only` can support internal learning but should not
support a public corpus-improves-agents claim.

## Claim Levels

Use explicit claim labels:

| Claim level | Required evidence | Allowed wording |
| --- | --- | --- |
| `schema_draft` | Markdown or JSON Schema exists, no clock. | "Schema draft exists." |
| `clock_started` | Canonical schema, fixtures, validator, changelog, and release/tag exist. | "A3 adoption clock has started." |
| `review_signal` | 3 non-operator reviews by 30 days. | "External review is underway." |
| `weak_schema_gravity` | 2 non-operator valid producer/consumer integrations by 90 days. | "Early schema integration exists." |
| `schema_dependency` | 1 unrelated workflow depends on compatibility by 180 days. | "A3 partially holds for schema gravity." |
| `corpus_signal` | 100 labeled bundles across 3 repos/teams, with outcomes. | "Corpus is useful enough to study." |
| `corpus_moat_candidate` | 1,000 labeled bundles with accepted/rejected outcomes and recurrence tracking. | "A corpus moat is plausible." |

Do not use "standard," "ecosystem," "moat," or "agent-improving corpus" language
before the matching claim level is reached.

## Counting Rules

Count an event only if all applicable checks pass:

- The actor is not controlled by the operator, unless the event is explicitly
  marked as operator-controlled and excluded from adoption totals.
- The bundle validates against the canonical JSON Schema version claimed.
- `redaction_report`, `missing_evidence`, evidence refs, and cited hypotheses
  remain present through projection or round-trip.
- The event has a public URL or a private evidence hash plus reviewer.
- The event is dated and tied to a schema version.
- Compatibility breakage is recorded as negative evidence, not omitted.
- The same organization cannot count as multiple unrelated integrations unless
  there are separate tools/workflows with separate owners.

Do not count:

- stars, likes, follows, newsletter signups, or social praise;
- forks with no validated bundle generation or consumption;
- fixtures generated only by Parallax's own test harness;
- integrations that strip safety fields;
- agent demos where the model sees a Markdown projection but no consumer depends
  on the canonical JSON schema.

## Refresh Cadence

Update `schema-adoption-results.md`:

- at every schema release;
- after every external review, integration attempt, breakage report, or
  compatibility decision;
- at 30/90/180/365-day clock checkpoints.

Update `corpus-outcome-results.md`:

- after every batch of 10 outcome-labeled bundles during Phase 0/MVP;
- after any unsafe/worsened outcome;
- before making any public corpus usefulness claim.

If a ledger is stale at a checkpoint, A3 remains unproven.

## Relationship To Other Research

- [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md)
  defines the thresholds; this note defines the event ledger.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  bundle contract events validate against.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  can seed early `outcome_labeled` rows.
- [A2 interview evidence ledger](a2-interview-evidence-ledger.md) can seed early
  `external_review`, `producer_integration`, and `consumer_integration` rows.
- [Fixer component and outcome loop](fixer-component-and-outcome-loop.md)
  supplies later accepted/rejected fix outcomes.

## Bottom Line

A3 only becomes real when outsiders depend on the schema and when bundles accrue
outcomes. The public ledger is the proof boundary: if an adoption or corpus event
is not recorded with schema version, actor relationship, validation status,
evidence hash, and count decision, it is learning signal at most, not moat
evidence.

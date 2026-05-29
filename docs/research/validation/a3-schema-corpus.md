# A3 — Schema Adoption and Corpus Moat

> A3 (one of the A1–A7 bear-case assumptions) holds that the open evidence schema plus the failure/fixer-outcome corpus becomes a compounding moat — and it is treated as existential and currently unproven. The gate is explicit that a schema is not a moat merely because it exists in Markdown: the adoption clock starts only once machine-readable artifacts (canonical JSON Schema at `schemas/evidence-bundle/v0.1.0/schema.json`, positive/negative fixtures, a validator command, a compatibility/semver policy, and a changelog) exist and a public release/tag lands, and even then only validated, non-operator producer/consumer integrations, workflow dependencies, and compatibility decisions count — never stars, forks, one-off operator demos, or product-language claims like SigNoz's "open investigation format" (the 2026-05-25 check found no published schema there). The corpus is a separate gate that only begins when bundles carry labeled outcomes, with thresholds of 5–10 hand-built (Phase 0), 25 fault-injected (tiny MVP), 100 across ≥3 repos/teams before any corpus-improves-agents claim, and 1,000 with recurrence tracking before any data-moat claim. The current Parallax claim level is `schema_draft`: there is no released JSON Schema artifact, validator, canonicalization command, fixture corpus, projection-equivalence harness, or MCP `outputSchema` fixture, so the A3 adoption clock has not started and the next durable boundary is a first `schema_release` event. Both proofs are bound to a public, source-linked ledger (`schema-adoption-results.md`, `corpus-outcome-results.md`): if an adoption or corpus event is not recorded with schema version, actor relationship, validation status, evidence hash, and count decision, it is learning signal at most, not moat evidence.

This note consolidates the following previously-separate research files, each preserved in full below:

- `schema-adoption-and-corpus-moat-gate.md`
- `a3-schema-adoption-corpus-ledger.md`

## Schema Adoption and Corpus Moat Gate

_Provenance: merged verbatim from `schema-adoption-and-corpus-moat-gate.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

This operationalizes bear-case assumption A3:

> The open evidence schema plus failure/fixer-outcome corpus becomes a
> compounding moat.

The repo already defines a `v0` [Evidence bundle and open schema](../architecture/evidence-bundle-schema.md).
That is necessary but not sufficient. A schema is not a moat because it exists in
Markdown; it becomes leverage only when external tools can validate against it,
agents can consume it consistently, contributors can extend it without breaking
older consumers, and real accepted/rejected/reverted fixer outcomes accumulate
into a corpus.

This gate defines the adoption clock, conformance artifacts, compatibility
rules, corpus events, and NO-GO conditions for A3.
The companion
[A3 schema adoption and corpus ledger](a3-schema-corpus.md)
defines the public event ledger that decides what actually counts.

### Source Posture

Current primary references reinforce three rules:

- OpenTelemetry treats stability as a contract: stable APIs must avoid backward
  incompatible minor-version changes, semantic conventions have explicit
  stability status, and additive telemetry changes are handled differently from
  breaking changes
  ([OpenTelemetry versioning and stability](https://opentelemetry.io/docs/specs/otel/versioning-and-stability/),
  [OpenTelemetry telemetry stability](https://opentelemetry.io/docs/specs/otel/telemetry-stability/)).
- JSON Schema draft 2020-12 gives Parallax a standard machine-validation layer:
  `$schema`, `$id`, vocabularies, references, and dialect declaration are part of
  the core spec
  ([JSON Schema 2020-12](https://json-schema.org/draft/2020-12),
  [JSON Schema Core](https://json-schema.org/draft/2020-12/json-schema-core)).
- MCP's latest specification is `2025-11-25`. It uses JSON Schema 2020-12 as the
  default schema dialect, exposes tool input/output schemas and structured tool
  results, reserves `_meta` for protocol/extension metadata, and warns that tool
  descriptions/annotations are untrusted unless obtained from a trusted server.
  A Parallax bundle schema can therefore be reused by MCP tools and clients, but
  only the canonical structured JSON counts for conformance; Markdown/text is a
  projection
  ([MCP specification](https://modelcontextprotocol.io/specification/2025-11-25),
  [MCP schema reference](https://modelcontextprotocol.io/specification/2025-11-25/schema)).
- SigNoz's official agent-native landing page now claims an "open investigation
  format," but the 2026-05-25 focused check found no published schema or
  portable artifact in the checked landing page, MCP docs, MCP README, or
  release metadata
  ([SigNoz open investigation format check](../market/competitor-watch.md)).
  Product-language claims therefore do not start the A3 clock. Only
  machine-readable schemas, fixtures, validators, compatibility rules, and
  artifact exports count.

Updated implication from the A1/A6 source-field pass: JSON Schema and MCP output
schemas can require the presence and shape of `source_field_policy`, but they
cannot prove that no forbidden value was copied into a free-text field. The
conformance suite therefore needs both schema validation and semantic negative
fixtures that intentionally leak runner-private, grader-private, and default
triage-private fields.

Internal sources:

- [Risks and the bear case](../decisions/risks-and-bear-case.md) makes A3 existential: without
  adoption there is no schema gravity, and without usage there is no failure/fix
  corpus.
- [Future platform direction](../00-vision/platform-direction.md) says the platform
  outcome is earned only if the open schema is adopted and the corpus compounds.
- [Business model and economics](business-model.md) says adoption,
  not early revenue, is the first metric because schema/corpus gravity is the
  moat-building mechanism.
- [A3 schema adoption and corpus ledger](a3-schema-corpus.md)
  defines the public schema-adoption and corpus-outcome event rows, count rules,
  claim levels, and refresh cadence.

### What Must Exist Before The Clock Starts

Do not declare "schema published" until all of these exist in the repo:

| Artifact | Required content |
| --- | --- |
| Canonical JSON Schema | `schemas/evidence-bundle/v0.1.0/schema.json` with `$schema`, `$id`, semver `schema_version`, required top-level fields, node/edge/hypothesis/redaction/source-field-policy definitions, and extension rules. |
| Example fixtures | At least one valid fixture for issue, trace, CI failure, CLI invocation, agent session anchors, and an MCP structured tool-result wrapper. |
| Negative fixtures | Missing `redaction_report`, missing source-field policy status for eval/corpus bundles, source-field policy violation, missing `missing_evidence`, uncited hypothesis, invalid edge target, unknown required field, oversized inline dump, MCP text-only result counted as canonical, and safety fields hidden only in `_meta`. |
| Validator | A small CLI or test command that validates all positive/negative fixtures. |
| Compatibility policy | A short `schemas/evidence-bundle/README.md` that states major/minor/patch rules, extension namespacing, deprecation window, and consumer expectations. |
| Changelog | Schema changes recorded by version, with migration notes and breaking-change labels. |

The current Markdown spec is a design draft. The adoption clock starts only when
these machine-readable artifacts exist and are usable without reading the
implementation.

### Compatibility Rules

Parallax should copy the discipline of stable telemetry ecosystems:

| Rule | Policy |
| --- | --- |
| Canonical format | JSON bundle is canonical. Markdown, ZIP, and MCP responses are projections. |
| Semver | `schema_version` uses semver. `0.x` may change faster, but every change is still recorded. `1.0.0` starts the external-compatibility commitment. |
| Additive minor | New optional fields, new node types, new edge types, and new hypothesis checks are minor-version changes. |
| Breaking major | Removing/renaming required fields, changing field meaning, changing `strength` semantics, or changing redaction-report requirements requires a major version. |
| Consumer behavior | Consumers must ignore unknown optional node/edge types and preserve unknown extension fields when round-tripping. |
| Extension namespace | External extensions use reverse-DNS or URI-like namespaces, such as `com.example.foo`, to avoid collisions. |
| MCP projection | MCP tools must declare an output schema and return the canonical bundle as structured JSON. Text/Markdown content is a deterministic projection and cannot be the source of truth. |
| Required extension | If an extension is required to interpret safety or meaning, it must be listed in a `required_extensions` field. Consumers that do not understand it must fail closed. |
| Deprecation | Fields can be deprecated in one minor, retained for at least one more minor, then removed only in the next major. |
| Safety invariants | `redaction_report`, source-field policy status for eval/corpus bundles, `missing_evidence`, evidence refs, and cited hypotheses are never optional for agent-visible bundles. |

### A3 Adoption Clock

Start the clock when the artifacts above land and a public release/tag exists.
Track these checkpoints:

| Time from schema release | Target | Interpretation |
| --- | --- | --- |
| 30 days | 3 non-operator teams or tools have reviewed the schema and opened issues, comments, or patches. | Early design feedback; no adoption claim yet. |
| 90 days | 2 non-operator integrations generate or consume a valid bundle fixture, even if private/pilot. | Weak but real schema gravity. |
| 180 days | 1 unrelated tool or team depends on the schema for a workflow Parallax does not directly control. | A3 begins to hold. |
| 365 days | 3 unrelated integrations or recurring workflows depend on the schema, and compatibility breaks would create external pain. | Schema moat is plausible. |

Do not count:

- stars, likes, or newsletter interest;
- forks with no bundle generation/consumption;
- one-off demos controlled entirely by the operator;
- integrations that cannot validate against the canonical schema.

Count:

- independent tools that emit Parallax-compatible bundles;
- coding-agent wrappers that consume bundles;
- CI/observability systems that export bundle fixtures;
- design partners using the schema in their own pipelines;
- external PRs/issues that force compatibility decisions.

### Corpus Gate

The failure/fixer-outcome corpus is separate from schema adoption. It starts only when
Parallax records outcomes for real or realistic tasks:

| Corpus event | Required fields |
| --- | --- |
| `bundle_presented` | bundle id/version, anchor type/id, redaction policy, source-field policy status when eval/corpus-derived, consumer type, token/size budget. |
| `agent_or_human_action` | actor type, tool/model if agent, action summary, evidence refs cited, unsupported claims. |
| `patch_or_proposal` | repo/ref, files touched, tests run, PR/proposal link or local patch ref. |
| `outcome` | accepted/rejected/needs-human/inconclusive, tests passed, recurrence status, reviewer notes. |
| `regression_feedback` | whether the issue recurred, whether the fix worsened another issue, rollback/revert refs. |

Initial corpus thresholds:

| Stage | Threshold |
| --- | --- |
| Phase 0 manual eval | 5-10 hand-built bundles with known outcomes, as specified in the bundle-value runbook. |
| Tiny MVP pilot | 25 real or fault-injected bundles with outcome labels. |
| First public usefulness claim | 100 labeled bundles across at least 3 repos or teams. |
| Corpus moat claim | 1,000 labeled bundles with accepted/rejected outcomes and recurrence tracking. |

Until the 100-labeled-bundle threshold, do not claim the corpus improves agents.
Until the 1,000-labeled-bundle threshold, do not claim a data moat.

### Conformance Suite

The conformance suite should test five things:

1. **Schema validity:** every positive fixture validates against JSON Schema
   draft 2020-12; every negative fixture fails for the expected reason.
2. **Projection equivalence:** CLI, HTTP, and MCP return the same canonical JSON
   for the same anchor; Markdown is a deterministic projection.
3. **Compatibility:** a v0.1 consumer can ignore v0.2 optional fields and still
   preserve safety fields.
4. **MCP structured output:** the MCP tool wrapper declares an `outputSchema`,
   returns canonical bundle JSON as structured content, and keeps Markdown/text
   as a projection.
5. **Safety invariants:** bundles without `redaction_report`,
   required source-field policy status, `missing_evidence`, valid refs, or cited
   hypotheses are invalid for agent exposure; the same fields must not live only
   in optional `_meta` or tool annotations.

Minimum command shape:

```bash
parallax-schema validate schemas/evidence-bundle/v0.1.0/examples/*.json
parallax-schema test schemas/evidence-bundle/conformance/
```

This can start as a simple Rust or Node-based validator, but the test outputs
must be deterministic and easy for external contributors to run.

### Pass, Narrow, Kill

| Result | Interpretation | Action |
| --- | --- | --- |
| External review plus 2 integrations by 90 days | Weak positive A3 | Keep schema public, continue pilots, avoid premature moat language. |
| One unrelated workflow depends on the schema by 180 days | A3 partially holds | Invest in compatibility, versioning, examples, and corpus capture. |
| No non-operator users by 180 days | A3 fails for schema gravity | Reframe moat around product execution, not standardization. |
| Bundles help agents in A1 but no external schema adoption | Product value exists, schema moat weak | Keep schema open but stop presenting it as the primary moat. |
| Schema adoption exists but corpus does not grow | Interchange format useful, data moat absent | Focus on integrations and hosted/fixer loops that generate outcomes. |
| Corpus grows but schema breaks consumers repeatedly | Execution failure | Freeze `1.0`, strengthen conformance, and stop adding surface until compatibility stabilizes. |

### Repo Changes This Gate Implies

When implementation begins, add:

```text
schemas/
  evidence-bundle/
    README.md
    CHANGELOG.md
    v0.1.0/
      schema.json
      examples/
      negative/
    conformance/
```

Also add a `docs/research/schema-adoption-results.md` once external feedback or
integration attempts exist. That file should record dates, external actors,
integration type, compatibility issues, and whether each event counts toward the
A3 clock, following the event schema in the
[A3 schema adoption and corpus ledger](a3-schema-corpus.md).
Add `docs/research/corpus-outcome-results.md` once bundles produce
accepted/rejected/inconclusive outcome labels.

### Relationship To Other Research

- [Evidence bundle and open schema](../architecture/evidence-bundle-schema.md) defines the
  current `v0` contract this gate makes testable.
- [A3 schema adoption and corpus ledger](a3-schema-corpus.md)
  defines the event ledger and claim labels for schema gravity and corpus growth.
- [Risks and the bear case](../decisions/risks-and-bear-case.md) names A3 as existential.
- [Build roadmap](../architecture/build-roadmap.md) starts the adoption
  clock in Phase 2 and records accepted/rejected fix outcomes in Phase 4.
- [Business model and economics](business-model.md) relies on
  schema/corpus adoption as the open-source moat-building path.
- [Bundle-value evaluation](a1-bundle-value/bundle-value-evaluation.md) defines the A1 result
  that makes the schema worth adopting.
- [User interview and deployment intent gate](a2-user-demand.md)
  can recruit the first design partners who review or pilot the schema.

### Bottom Line

A3 is not proven by writing a schema. It is proven by external dependency and
outcome data: machine-readable schema artifacts, conformance tests, independent
integrations, and a growing corpus of bundles tied to accepted or rejected fixes.
Until those exist, the honest moat is only a hypothesis.

## A3 Schema Adoption and Corpus Ledger

_Provenance: merged verbatim from `a3-schema-adoption-corpus-ledger.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

The [Schema adoption and corpus moat gate](a3-schema-corpus.md)
defines what A3 must prove:

> The open evidence schema plus failure/fixer-outcome corpus becomes a
> compounding moat.

This note defines the missing public ledger for that proof. A3 is not validated
by publishing Markdown, JSON Schema files, or a few friendly comments. It is
validated by durable events: schema releases, conformance runs, external review,
external bundle producers/consumers, compatibility decisions, and labeled
failure/fixer-outcome rows.

No A3 pass, schema-gravity, corpus-growth, or moat claim should count unless the
supporting events are recorded in a committed, source-linked ledger.

### Current Source Posture

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
- RFC 8785 JSON Canonicalization Scheme is the current candidate for canonical
  bundle hashing because it specifies deterministic JSON serialization and
  UTF-8 output for repeatable hashing
  ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html)).
- MCP's current specification is version `2025-11-25` and says MCP uses JSON
  Schema for validation, defaults to JSON Schema 2020-12 when `$schema` is
  omitted, and exposes tool input/output schemas plus structured tool results.
  It also says tool annotations/descriptions should be treated as untrusted
  unless they come from a trusted server, and that security/consent controls are
  implementor responsibilities rather than protocol guarantees. This makes
  Parallax bundle schema compatibility relevant to agent tool surfaces, but only
  if the canonical bundle is returned as structured content and safety fields are
  preserved
  ([MCP overview](https://modelcontextprotocol.io/specification/2025-11-25/basic),
  [MCP specification](https://modelcontextprotocol.io/specification/2025-11-25),
  [MCP schema reference](https://modelcontextprotocol.io/specification/2025-11-25/schema)).

Internal sources:

- [Evidence bundle and open schema](../architecture/evidence-bundle-schema.md) defines the
  current `v0` bundle draft.
- [Risks and the bear case](../decisions/risks-and-bear-case.md) says A3 is existential
  because the schema/corpus moat fails without adoption.
- [Build roadmap and validation sequence](../architecture/build-roadmap.md)
  starts the schema-adoption clock in Phase 2 and feeds outcome data in Phase 4.
- [A1 eval result ledger and model refresh](a1-bundle-value/a1-eval-result-ledger-and-model-refresh.md)
  defines the first outcome-style result ledger that can seed early corpus
  events.
- [A6 redaction red-team ledger](../capture/redaction.md) now requires
  source-field policy audit rows for agent-visible eval/corpus projections.
- [A2 interview evidence ledger](a2-user-demand.md) defines the
  design-partner signal that can seed external schema review.

### Current Claim Boundary

Current Parallax claim level: `schema_draft`.

This is intentionally narrow. The repository has a Markdown evidence-bundle
draft, but no released JSON Schema artifact, schema changelog, canonicalization
command, validator command, committed fixture corpus, projection-equivalence
harness, or MCP `outputSchema` fixture. The A3 adoption clock has not started.

The next boundary is not another prose refinement. It is a first
`schema_release` event that records the exact schema URI/hash, validator
version, fixture hash, canonicalization method, changelog entry, release/tag or
commit, and initial conformance command. Until that event exists, every
integration, agent-demo, or bundle example must be marked `do_not_count` for
schema adoption unless it is explicitly only an internal learning signal.

### Why A3 Needs A Ledger

The schema/corpus moat has several false-positive paths:

| Failure mode | How it misleads Parallax | Ledger control |
| --- | --- | --- |
| Schema published, no consumer pain | The repo has a format, not a standard. | Count only validated producer/consumer events and compatibility decisions. |
| Operator-controlled demos counted as adoption | n=1 activity masquerades as ecosystem pull. | Track actor relationship and control level for every event. |
| Friendly feedback counted as dependency | Review is useful, but not adoption. | Separate review events from integration/dependency events. |
| Corpus rows without outcomes | A pile of bundles is not a failure/fixer-outcome corpus. | Count only rows with action, outcome, and recurrence/review status. |
| Private pilots summarized too vaguely | The public repo cannot audit why A3 passed. | Allow anonymized actors, but require evidence hashes, reviewer, and count rules. |
| Compatibility breakage hidden | A schema that repeatedly breaks consumers is anti-moat. | Record breakage reports, migration decisions, and deprecation windows. |

### Ledger Artifacts

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

### Schema Adoption Event Types

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

### Adoption Event Row Schema

Each schema-adoption event should be recorded as a structured row:

```yaml
event_id: A3-SCHEMA-001
event_date: 2026-05-25
event_type: schema_release | external_review | producer_integration | consumer_integration | workflow_dependency | conformance_run | compatibility_decision | breakage_report
schema_version: 0.1.0
actor_class: operator | design_partner | unrelated_tool | oss_maintainer | customer_team | agent_wrapper | observability_tool | private_partner
actor_relationship: operator_controlled | warm_design_partner | external_non_operator | unrelated_public | private_non_operator
transport_surface: file | cli | http | mcp | other
canonical_json_present: true
schema_uri: https://schemas.parallax.dev/evidence-bundle/0.1.0/schema.json
schema_hash: sha256:...
canonicalization_method: jcs-rfc8785 | other
canonical_bundle_hash: sha256:...
projection_equivalence_status: pass | fail | not_checked
safety_fields_preserved: true
mcp_output_schema_id: https://schemas.parallax.dev/evidence-bundle/0.1.0/schema.json | null
structured_content_hash: sha256:... | null
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
  - "Missing source_field_policy status in eval-derived fixture."
decision: count | do_not_count | negative_signal
reviewer: operator | second_reviewer | external_reviewer
```

An event must be marked `do_not_count` when evidence is missing, the actor is
operator-controlled, the fixture cannot validate, or the integration is only a
demo scripted entirely by Parallax.

For MCP events, a text-only or Markdown-only tool response is a projection, not
a schema-consumer event. Count MCP producer/consumer events only when the tool
declares an `outputSchema`, returns the canonical bundle in `structuredContent`,
and preserves required safety fields outside untrusted descriptions,
annotations, or optional `_meta` fields.

### Corpus Outcome Event Types

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
outcome is useful operational evidence but not failure/fixer-outcome corpus data.

### Corpus Row Schema

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
source_field_policy_status: pass | fail | not_applicable
source_field_policy_hash: sha256:... | null
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

Corpus rows with `verifier: unknown`, missing `redaction_policy_version`,
missing required source-field policy status, or
`privacy_level: private_hash_only` can support internal learning but should not
support a public corpus-improves-agents claim.

### Claim Levels

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

### Counting Rules

Count an event only if all applicable checks pass:

- The actor is not controlled by the operator, unless the event is explicitly
  marked as operator-controlled and excluded from adoption totals.
- The bundle validates against the canonical JSON Schema version claimed.
- The event records the schema URI, schema hash, canonicalization method, and
  canonical bundle hash. File, CLI, HTTP, and MCP variants that claim to be the
  same bundle must share the same canonical hash.
- For MCP integrations, `structuredContent` validates against the declared
  `outputSchema`; unstructured text is treated only as a deterministic
  projection. Safety fields hidden only in `_meta`, tool descriptions, or
  annotations do not count as preserved.
- `redaction_report`, required `source_field_policy` status, `missing_evidence`,
  evidence refs, and cited hypotheses remain present through projection or
  round-trip.
- Eval/corpus-derived bundles have `source_field_policy_status: pass` and a
  policy hash; `not_applicable` is allowed only for direct production telemetry
  that did not originate from a mixed source row.
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
- MCP tool demos that expose only text/Markdown, hide safety fields in `_meta`,
  or rely on tool descriptions/annotations as trusted behavior guarantees.

### Refresh Cadence

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

### Relationship To Other Research

- [Schema adoption and corpus moat gate](a3-schema-corpus.md)
  defines the thresholds; this note defines the event ledger.
- [Evidence bundle and open schema](../architecture/evidence-bundle-schema.md) defines the
  bundle contract events validate against.
- [A1 eval result ledger and model refresh](a1-bundle-value/a1-eval-result-ledger-and-model-refresh.md)
  can seed early `outcome_labeled` rows.
- [A6 redaction red-team ledger](../capture/redaction.md) defines the
  source-field and redaction proof that corpus rows must preserve.
- [A2 interview evidence ledger](a2-user-demand.md) can seed early
  `external_review`, `producer_integration`, and `consumer_integration` rows.
- [Fixer component and outcome loop](../decisions/fixer-boundary.md)
  supplies later accepted/rejected fix outcomes.
- [Fixer outcome ledger](../decisions/fixer-boundary.md) controls when those
  accepted/rejected/edited/reverted/recurrence rows are measured enough to feed
  the corpus.

### Bottom Line

A3 only becomes real when outsiders depend on the schema and when bundles accrue
outcomes. The public ledger is the proof boundary: if an adoption or corpus event
is not recorded with schema version, actor relationship, validation status,
evidence hash, and count decision, it is learning signal at most, not moat
evidence.

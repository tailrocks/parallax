# Schema Adoption and Corpus Moat Gate

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This operationalizes bear-case assumption A3:

> The open evidence schema plus failure/fixer-outcome corpus becomes a
> compounding moat.

The repo already defines a `v0` [Evidence bundle and open schema](evidence-bundle-and-schema.md).
That is necessary but not sufficient. A schema is not a moat because it exists in
Markdown; it becomes leverage only when external tools can validate against it,
agents can consume it consistently, contributors can extend it without breaking
older consumers, and real accepted/rejected/reverted fixer outcomes accumulate
into a corpus.

This gate defines the adoption clock, conformance artifacts, compatibility
rules, corpus events, and NO-GO conditions for A3.
The companion
[A3 schema adoption and corpus ledger](a3-schema-adoption-corpus-ledger.md)
defines the public event ledger that decides what actually counts.

## Source Posture

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
  ([SigNoz open investigation format check](signoz-open-investigation-format-check.md)).
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

- [Risks and the bear case](risks-and-bear-case.md) makes A3 existential: without
  adoption there is no schema gravity, and without usage there is no failure/fix
  corpus.
- [Future platform direction](future-platform-direction.md) says the platform
  outcome is earned only if the open schema is adopted and the corpus compounds.
- [Business model and economics](business-model-and-economics.md) says adoption,
  not early revenue, is the first metric because schema/corpus gravity is the
  moat-building mechanism.
- [A3 schema adoption and corpus ledger](a3-schema-adoption-corpus-ledger.md)
  defines the public schema-adoption and corpus-outcome event rows, count rules,
  claim levels, and refresh cadence.

## What Must Exist Before The Clock Starts

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

## Compatibility Rules

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

## A3 Adoption Clock

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

## Corpus Gate

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

## Conformance Suite

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

## Pass, Narrow, Kill

| Result | Interpretation | Action |
| --- | --- | --- |
| External review plus 2 integrations by 90 days | Weak positive A3 | Keep schema public, continue pilots, avoid premature moat language. |
| One unrelated workflow depends on the schema by 180 days | A3 partially holds | Invest in compatibility, versioning, examples, and corpus capture. |
| No non-operator users by 180 days | A3 fails for schema gravity | Reframe moat around product execution, not standardization. |
| Bundles help agents in A1 but no external schema adoption | Product value exists, schema moat weak | Keep schema open but stop presenting it as the primary moat. |
| Schema adoption exists but corpus does not grow | Interchange format useful, data moat absent | Focus on integrations and hosted/fixer loops that generate outcomes. |
| Corpus grows but schema breaks consumers repeatedly | Execution failure | Freeze `1.0`, strengthen conformance, and stop adding surface until compatibility stabilizes. |

## Repo Changes This Gate Implies

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
[A3 schema adoption and corpus ledger](a3-schema-adoption-corpus-ledger.md).
Add `docs/research/corpus-outcome-results.md` once bundles produce
accepted/rejected/inconclusive outcome labels.

## Relationship To Other Research

- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  current `v0` contract this gate makes testable.
- [A3 schema adoption and corpus ledger](a3-schema-adoption-corpus-ledger.md)
  defines the event ledger and claim labels for schema gravity and corpus growth.
- [Risks and the bear case](risks-and-bear-case.md) names A3 as existential.
- [Build roadmap](build-roadmap-and-validation-sequence.md) starts the adoption
  clock in Phase 2 and records accepted/rejected fix outcomes in Phase 4.
- [Business model and economics](business-model-and-economics.md) relies on
  schema/corpus adoption as the open-source moat-building path.
- [Bundle-value evaluation](bundle-value-evaluation.md) defines the A1 result
  that makes the schema worth adopting.
- [User interview and deployment intent gate](user-interview-and-deployment-intent-gate.md)
  can recruit the first design partners who review or pilot the schema.

## Bottom Line

A3 is not proven by writing a schema. It is proven by external dependency and
outcome data: machine-readable schema artifacts, conformance tests, independent
integrations, and a growing corpus of bundles tied to accepted or rejected fixes.
Until those exist, the honest moat is only a hypothesis.

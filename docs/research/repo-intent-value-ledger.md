# Repo-Intent Value Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

[Repo-intent dependence and the degraded mode](repo-intent-dependence.md)
defines the strategic claim:

> Runtime evidence is the product floor; repo-held intent is an opt-in
> multiplier.

This ledger turns that claim into an auditable evaluation. It defines the run
artifacts, row schemas, counting rules, claim levels, and product wording needed
before Parallax can say that docs, decisions, tasks, roadmap, or agent
instruction files improve agent diagnosis or patch quality.

Current status: `not_measured`.

Central rule:

> No repo-intent moat or "why layer" claim until paired runs show that adding
> repo-held intent to the same runtime bundle improves constraint-aware diagnosis
> or patch quality without hiding degraded-mode weakness, leaking private docs,
> or increasing unsupported claims.

## Current Source Snapshot

| Source | Ledger consequence |
| --- | --- |
| [GitHub Copilot repository custom instructions](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/add-custom-instructions/add-repository-instructions) | Repository-wide, path-specific, and agent instructions are now a mainstream agent interface. Parallax should treat repo intent as structured, versioned context rather than a chat-only convention. |
| [Claude Code memory and CLAUDE.md docs](https://code.claude.com/docs/en/memory) | Claude Code loads project instructions such as `CLAUDE.md`, can import `AGENTS.md`, supports path-scoped rules, and warns that instructions are context rather than hard enforcement. Stale, vague, or conflicting intent must be tested as a risk. |
| [On the Impact of AGENTS.md Files on the Efficiency of AI Coding Agents](https://arxiv.org/abs/2601.20404) | Early empirical evidence associates AGENTS.md files with lower median runtime and output-token use, but not with a proven fix-correctness lift. Parallax should measure outcome quality separately from efficiency. |
| [On the Use of Agentic Coding Manifests](https://arxiv.org/abs/2509.14744) | Agent manifests commonly contain operational commands, implementation notes, and architecture. That maps to Parallax intent nodes, but also shows why content quality and structure matter. |
| [Bundle-value evaluation](bundle-value-evaluation.md) | The A1 eval already measures bundle value. Repo-intent should be a paired sub-study, not a replacement for the raw-telemetry control. |
| [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md) | Model drift and task contamination also apply to repo-intent claims; every claim needs expiry and rerun triggers. |

## Claim Levels

| Level | Meaning | Minimum evidence |
| --- | --- | --- |
| `not_measured` | No repo-intent eval exists. | Default state. |
| `intent_manifest_indexed` | Repo-held intent sources can be discovered and normalized. | Source inventory covers README, docs, ADRs/decision records, roadmap/tasks, issue refs, and agent instruction files when present. |
| `intent_edges_projected` | Intent nodes and edges can appear in bundles without breaking schema or redaction. | Bundle projection rows include intent nodes, source refs, edge strengths, and missing-intent flags. |
| `intent_safety_pass` | Intent context does not leak private material or override policy. | Redaction, prompt-injection, stale/conflict, and gold-patch leakage fixtures pass. |
| `degraded_mode_pass` | Runtime-only bundles remain valuable without repo intent. | Runtime-only arm still meets the A1 decision rule or stays within the allowed drop from the intent arm. |
| `intent_value_signal` | Repo intent shows a useful directional lift. | Paired run shows better constraint adherence, fewer unsupported claims, or lower investigation time without a resolved-rate drop. |
| `intent_value_pass` | Repo intent is a claimable multiplier. | Mixed task set, at least two model families, pre-registered rule passes, degraded mode still acceptable, and no safety failures. |
| `intent_overfit_risk` | Intent helps only operator-like repos or leaks task-specific hints. | Result narrows ICP and blocks broad market wording. |
| `claim_expired` | Prior result is stale. | Refresh trigger fired or max age elapsed. |
| `claim_failed` | Repo intent harms quality, leaks, or makes degraded mode too weak. | Any required fixture or pre-registered outcome fails. |

Initial claim level: `not_measured`.

## Evaluation Arms

Run this as a paired sub-study under A1. Keep the existing A/B/C/D A1 arms, then
split the Parallax-bundle arm into runtime-only and runtime-plus-intent variants:

| Arm | Context | What it isolates |
| --- | --- | --- |
| `C0_runtime_bundle` | Repo + runtime evidence bundle with no repo-intent nodes. | Degraded mode: the market case for teams without curated intent. |
| `C1_runtime_plus_intent` | Same runtime bundle plus repo-intent nodes and edges. | Incremental value of docs, decisions, tasks, roadmap, and agent instructions. |
| `C2_intent_conflict` | Same as C1, but with seeded stale/conflicting intent fixtures when safe. | Whether agents over-trust bad intent or report conflicts. |

The primary comparison is `C1_runtime_plus_intent` versus
`C0_runtime_bundle`. A useful result does not rescue Parallax if C0 is weak; the
product must still work when intent is absent.

## Result Artifacts

The durable result index lives at:

```text
docs/research/repo-intent-value-results.md
```

Each run stores immutable artifacts under:

```text
docs/research/repo-intent-value-runs/<run_id>/manifest.json
docs/research/repo-intent-value-runs/<run_id>/intent-source-inventory.jsonl
docs/research/repo-intent-value-runs/<run_id>/intent-node-ledger.jsonl
docs/research/repo-intent-value-runs/<run_id>/intent-edge-ledger.jsonl
docs/research/repo-intent-value-runs/<run_id>/context-arm-manifest.jsonl
docs/research/repo-intent-value-runs/<run_id>/task-arm-results.jsonl
docs/research/repo-intent-value-runs/<run_id>/constraint-audit.jsonl
docs/research/repo-intent-value-runs/<run_id>/unsupported-claim-audit.jsonl
docs/research/repo-intent-value-runs/<run_id>/intent-conflict-results.jsonl
docs/research/repo-intent-value-runs/<run_id>/redaction-results.jsonl
docs/research/repo-intent-value-runs/<run_id>/claim-ledger.jsonl
docs/research/repo-intent-value-runs/<run_id>/hashes.sha256
```

Do not commit private design docs, task descriptions, issue comments, internal
roadmap text, or full agent transcripts unless redacted and approved. Commit
hashes, normalized intent rows, bounded excerpts, and source-class labels.

## Run Manifest

```json
{
  "schema_version": "repo-intent-value-v1",
  "run_id": "repo-intent-2026-05-25-r001",
  "research_date": "2026-05-25",
  "a1_run_id": "a1-phase0-2026-05-25-r001",
  "repo_commit": "<task_repo_commit>",
  "parallax_commit": "<parallax_commit_sha>",
  "task_set_hash": "sha256:<hex>",
  "intent_corpus_hash": "sha256:<hex>",
  "bundle_schema_version": "0.1.0",
  "redaction_policy": "a6-default-deny-vN",
  "intent_schema_version": "repo-intent-v1",
  "arms": ["C0_runtime_bundle", "C1_runtime_plus_intent", "C2_intent_conflict"],
  "models": [
    {
      "provider": "provider-name",
      "model_id": "exact-api-model-id",
      "model_family": "frontier-a"
    }
  ],
  "pre_registered_rule": "C1 improves constraint adherence by >=10pp or unsupported claims by >=15% without lowering resolved rate; C0 remains within 80% of C1 resolved rate",
  "result": "not_measured"
}
```

## Minimum Row Schemas

Intent source row:

```json
{
  "source_id": "intent_src_001",
  "source_type": "adr|readme|docs|roadmap|task|issue|agent_instruction|commit_message",
  "path_or_provider": "docs/decisions/001-auth.md",
  "commit_or_updated_at": "2026-05-20T00:00:00Z",
  "available_before_failure": true,
  "private": false,
  "redaction_policy": "a6-default-deny-vN",
  "content_hash": "sha256:<hex>",
  "included": true
}
```

Intent node row:

```json
{
  "node_id": "intent_001",
  "source_id": "intent_src_001",
  "node_type": "decision|constraint|goal|non_goal|task_intent|operational_rule",
  "summary": "Use passwordless auth; do not add password login.",
  "confidence": "source_stated",
  "staleness": "current|possibly_stale|stale",
  "redacted": false,
  "bundle_visible": true
}
```

Intent edge row:

```json
{
  "edge_id": "intent_edge_001",
  "from": "intent_001",
  "to": "code_change_checkout_auth",
  "edge_type": "constrains_fix|explains_design|contradicts_hypothesis|supports_hypothesis|missing_intent",
  "strength": "strong|medium|weak",
  "evidence_ref": "intent_src_001",
  "manual_audit_required": false
}
```

Context arm row:

```json
{
  "task_id": "task_001",
  "arm": "C1_runtime_plus_intent",
  "runtime_bundle_hash": "sha256:<hex>",
  "intent_corpus_hash": "sha256:<hex>",
  "context_hash": "sha256:<hex>",
  "gold_patch_leak_check": "pass",
  "redaction_check": "pass",
  "intent_available": true
}
```

Task arm result row:

```json
{
  "task_id": "task_001",
  "arm": "C1_runtime_plus_intent",
  "model_id": "exact-api-model-id",
  "seed": 1,
  "resolved": true,
  "root_cause_grade": "correct|partial|wrong|unsupported",
  "constraint_adherence_grade": "pass|partial|fail|not_applicable",
  "intent_refs_used": 2,
  "unsupported_claim_count": 0,
  "stale_intent_overtrusted": false,
  "input_tokens": 0,
  "output_tokens": 0,
  "wall_clock_seconds": 0,
  "patch_hash": "sha256:<hex>"
}
```

Claim ledger row:

```json
{
  "claim_level": "intent_value_signal",
  "run_id": "repo-intent-2026-05-25-r001",
  "scope": "paired C1-vs-C0 sub-study, mixed public/fault-injected tasks",
  "granted_at": "2026-05-25T14:00:00Z",
  "expires_at": "2026-08-23T14:00:00Z",
  "result": "pass"
}
```

## Counting Rules

- Intent files must predate the failure, issue, or task prompt unless the row is
  explicitly marked as post-hoc and excluded from product claims.
- Do not include gold patches, hidden tests, solution notes, or task-specific
  hints in the intent corpus.
- C0 and C1 must use the same runtime evidence, task, model, seed, scaffold,
  tool permissions, and budgets. Only intent nodes differ.
- C1 wins only if it improves quality or efficiency without increasing
  unsupported claims, leaking private content, or lowering resolved rate.
- Degraded mode fails if C0 needs repo intent to clear A1. That narrows the
  market and blocks broad product wording even when C1 performs well.
- Count an intent edge as strong only when it cites a specific source and
  constrains a specific code path, test, task, or hypothesis.
- Stale or conflicting intent must be represented as `possibly_stale`, `stale`,
  or `contradicts_hypothesis`; agents should not silently obey it.
- Agent instruction files such as `AGENTS.md`, `CLAUDE.md`, and Copilot
  instructions count as operational intent, not as source-of-truth product
  requirements unless corroborated by docs/tasks/decisions.

## Pass Targets

Initial thresholds:

| Gate | Target |
| --- | --- |
| Intent projection | 100 percent of included intent nodes carry source refs, staleness, and redaction status. |
| Degraded mode | C0 resolved rate is at least 80 percent of C1 or independently passes A1. |
| Intent value | C1 improves constraint adherence by >=10 percentage points or reduces unsupported claims by >=15 percent. |
| No quality regression | C1 does not lower resolved rate or root-cause accuracy versus C0. |
| Token budget | C1 median input tokens increase by <=15 percent unless quality lift is large enough to justify a narrower high-context tier. |
| Conflict handling | Seeded stale/conflicting intent is reported as a conflict in 100 percent of audited cases. |
| Safety | Zero private intent leaks and zero gold-patch leaks. |

## Refresh Triggers

Mark the claim `claim_expired` and rerun when any of these changes:

- A new model family, coding-agent scaffold, or repo-instruction mechanism is
  used in product claims.
- GitHub Copilot, Claude Code, Cursor, Codex, or another target agent changes
  how repository instructions or memory files load.
- Parallax bundle schema, intent node schema, redaction policy, truncation
  policy, or source-ingest logic changes.
- A2 interviews show target users have materially different repo-intent hygiene
  than the tested corpus.
- Ninety days pass for public claims or a new A1 run supersedes the paired
  tasks.

## Product Wording

Allowed before measurement:

> Repo-held intent is an optional Parallax enrichment. The current value lift is
> unmeasured.

Allowed after `degraded_mode_pass`:

> Parallax remains useful for teams with code and telemetry even when curated
> docs or decisions are absent.

Allowed after `intent_value_pass`:

> In the tested tasks, adding repo-held intent improved constraint-aware agent
> work over the same runtime bundle without weakening degraded mode.

Avoid:

- "Parallax requires a monorepo."
- "Understands why the code exists" without cited intent refs.
- "Repo docs make agents autonomous."
- "AGENTS.md/CLAUDE.md improves fix correctness" unless the run measured
  correctness, not only token/time.
- "Intent is source of truth" when code/tests/runtime evidence contradict it.

## Relationship To Other Research

- [Repo-intent dependence and the degraded mode](repo-intent-dependence.md)
  defines the strategic floor/multiplier split this ledger measures.
- [Bundle-value evaluation](bundle-value-evaluation.md) owns the main A1 arms;
  this ledger is a paired sub-study under the Parallax-bundle arm.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  supplies model snapshot, contamination, and expiry rules.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) should carry
  intent nodes and edges only as additive, source-cited context.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  controls private docs, issue text, and agent instruction excerpts.
- [A2 interview evidence ledger](a2-interview-evidence-ledger.md) should record
  whether target users actually maintain docs, decisions, tasks, and roadmap in
  forms Parallax can use.

## Bottom Line

Repo intent is a promising multiplier and possible moat, but it must not become
an untested dependency. Parallax earns broad-market wording only if runtime-only
bundles work well; it earns repo-intent wording only if adding cited, current,
redacted intent improves agent work in paired runs.

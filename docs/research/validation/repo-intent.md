# Repo-Intent Dependence

> Strategic question 13 asks how much Parallax depends on the operator's context-rich monorepo (code plus docs, design decisions, tasks, roadmap). The decided answer is that value splits into two separable layers: a **runtime-evidence floor** (error grouping, correlation, evidence bundles, traces, retention) that needs only telemetry plus source code and no monorepo, and an opt-in **repo-intent multiplier** (linking failures to *why the code is the way it is*) that is additive and never a prerequisite. The strategic rule is fixed — the product must be fully valuable on the runtime floor alone, with degraded mode (teams with code + telemetry but no curated intent) treated as the common case the product is designed for, and repo-intent offered as opt-in enrichment that also seeds a moat telemetry-only competitors cannot match. What remains an open gate is the empirical size of the multiplier: the repo-intent value ledger is at status `not_measured`, and no "why layer" or moat claim is allowed until paired `C1_runtime_plus_intent` vs `C0_runtime_bundle` runs show repo-held intent improves constraint-aware diagnosis or patch quality without hiding degraded-mode weakness, leaking private docs, increasing unsupported claims, or treating agent instruction files (`AGENTS.md`, `CLAUDE.md`, Copilot instructions) as enforced policy rather than source-cited, scoped, possibly-stale context. Pre-registered thresholds (e.g. C1 lifts constraint adherence by ≥10pp or cuts unsupported claims by ≥15% without lowering resolved rate; C0 stays within 80% of C1) and stale/conflict, redaction, and projection fixtures gate every claim level and the allowed product wording.

This note consolidates the following previously-separate research files, each preserved in full below:

- `repo-intent-dependence.md`
- `repo-intent-value-ledger.md`

## Repo-Intent Dependence and the Degraded Mode

_Provenance: merged verbatim from `repo-intent-dependence.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

Answer strategic question 13 directly, because the corpus has only handled it in
one-line table cells: the vision assumes a context-rich monorepo (code **plus**
docs, design decisions, tasks, roadmap). How much of Parallax's value actually
depends on that, and what happens for the large majority of teams that do not work
that way? This matters because the answer sizes the addressable market and is the
sharpest edge of the founder-market-fit risk in the [bear case](risks-and-bear-case.md)
(A2 / n=1).

### The Value Decomposes Into Two Separable Layers

Parallax's value is not one thing that needs the monorepo. It is two layers, and
only the second needs repo-intent:

1. **Runtime evidence layer (the floor).** Error grouping, cross-signal
   correlation, evidence bundles, CLI/agent/CI traces, retention. This needs
   **telemetry + the source code**, nothing more. Any team that emits Sentry/OTLP
   data and has a repo gets the full floor. No monorepo, no ADRs, no task tracker
   required.
2. **Repo-intent layer (the multiplier).** Linking a failure to *why the code is
   the way it is* — design decisions, ADRs, tasks, roadmap. This needs the
   context-rich repo the vision assumes. It makes proposals better and avoids
   "why is this here" mistakes, but it is **additive**, not required.

The strategic rule that follows: **the product must be fully valuable on the
runtime-evidence floor alone. Repo-intent is an optional multiplier, never a
prerequisite.** If Parallax's pitch requires the monorepo+intent setup, its
market collapses to teams that work exactly like the operator — which is the
n=1 founder-market-fit trap.
The result contract for proving or narrowing this claim is the
[Repo-intent value ledger](repo-intent-value-ledger.md).

### Degraded Mode (Teams Without Repo-Intent)

Most teams have code + telemetry but not curated docs/decisions/tasks. Their
experience must still be excellent:

- They get grouping, correlation, trace/log/metric joins, release-regression
  detection, and bounded bundles.
- A coding agent fixes from **code + runtime evidence** — and SWE-bench shows
  agents already fix real bugs from code alone, so code + telemetry is a strong
  floor (see [bundle-value evaluation](bundle-value-evaluation.md)).
- What they lose is only the "why" layer: the agent may not know a piece of code
  exists to satisfy a constraint it cannot see. That is a real but bounded loss,
  and it degrades gracefully — the bundle simply omits intent edges rather than
  breaking.

Degraded mode is the **common case**, so it is the case the product must be
designed for. The context-rich monorepo is the power-user case.

### What Repo-Intent Adds When Present (The Upside)

When a team *does* keep decisions/tasks/roadmap in the repo (like the operator):

- proposals can cite the decision a fix must not violate;
- the agent can align a fix with stated intent, not just make tests pass;
- "this code is intentional, do not "simplify" it" is knowable.
- agent instruction files such as `AGENTS.md`, `CLAUDE.md`, and Copilot
  instructions can expose repo-local operating intent, but they are still
  context. They do not become policy enforcement unless hooks, settings, CI, or
  another control enforces them outside the prose file.

This is also a **moat seed**: telemetry-only competitors (Datadog, Sentry,
OpenObserve, SigNoz) do not link runtime evidence to repo-held intent. So
repo-intent is simultaneously something to *not depend on* (for market size) and
something that *differentiates* (for high-context teams). Offer it as an
opt-in enrichment: point Parallax at `docs/`, ADRs, tasks, and approved
instruction surfaces; it adds source-cited intent nodes/edges to the bundle when
available.

### Implication For The Bundle-Value Eval

This question is empirically testable and should be an arm in the
[bundle-value evaluation](bundle-value-evaluation.md): run the agent with
**bundle + repo-intent** vs **bundle + code only (no intent)**. The delta
measures how much repo-intent actually buys. Two outcomes:

- Small delta → degraded mode is nearly as good; market is broad; repo-intent is
  a nice-to-have. (Good for A2.)
- Large delta → the product leans on repo-intent; market narrows toward
  operator-like teams; flag the founder-market-fit risk and either invest in
  making intent capture trivial or accept the narrower wedge.

Either way, do not assume; measure.

### Bottom Line

Parallax's runtime-evidence value does not depend on the monorepo, and it must
not be allowed to. Build for the team that has code + telemetry and nothing else;
treat repo-intent (docs, decisions, tasks, roadmap) as an opt-in multiplier that
differentiates for high-context teams and seeds a moat telemetry-only tools
cannot match. The monorepo is the operator's advantage, not the product's
requirement — and the bundle-value eval should measure the size of that
advantage rather than presume it.

### Relationship To Other Research

- [Verdict](verdict.md) — Q13 row this expands; the GO depends on broad
  addressable market, i.e. on degraded mode being good.
- [Risks and the bear case](risks-and-bear-case.md) — A2 / founder-market-fit;
  repo-intent dependence is the sharpest form of that risk.
- [Bundle-value evaluation](bundle-value-evaluation.md) — add the
  intent-vs-no-intent arm proposed here.
- [Repo-intent value ledger](repo-intent-value-ledger.md) — defines the paired
  runtime-only versus runtime-plus-intent result rows, claim levels, and
  stale/conflicting-intent fixtures.
- [AI-native observability](ai-native-observability-and-incident-intelligence.md)
  — repo-intent linkage as a differentiator vs telemetry-only products.

## Repo-Intent Value Ledger

_Provenance: merged verbatim from `repo-intent-value-ledger.md` (2026-05-29 restructure)._

_(Shared note — see the Repo-Intent Dependence and the Degraded Mode section above.)_

Research date: 2026-05-25

### Purpose

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
> increasing unsupported claims, or treating agent instruction files as enforced
> policy rather than source-cited, scoped, and potentially stale context.

### Current Source Snapshot

| Source | Current evidence | Ledger consequence |
| --- | --- | --- |
| [GitHub Copilot repository custom instructions](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/add-custom-instructions/add-repository-instructions) | Copilot supports repository-wide `.github/copilot-instructions.md`, path-specific `.github/instructions/*.instructions.md`, and agent instructions. Current docs also say agents can use one or more `AGENTS.md` files, with the nearest file taking precedence, or a root `CLAUDE.md`/`GEMINI.md`. | Repo intent inventory must capture file type, path scope, nearest-file precedence, and target agent. A generic "repo instructions exist" flag is too weak. |
| [Claude Code memory and CLAUDE.md docs](https://code.claude.com/docs/en/memory) | Claude Code treats `CLAUDE.md` and auto memory as context, not enforced configuration; supports imports such as `@AGENTS.md`, `CLAUDE.local.md`, path-scoped `.claude/rules/`, managed policy files, and local exclusions. It warns about stale/conflicting instructions and says enforced behavior belongs in hooks/settings, not prose memory. | Repo intent rows must distinguish committed project intent from local/user/managed memory, imported files, context-only guidance, and externally enforced policy. |
| [OpenAI Codex introduction](https://openai.com/index/introducing-codex/) | OpenAI says Codex can be guided by `AGENTS.md` files placed in a repository, and that configured dev environments, reliable tests, and clear documentation improve agent performance. | `AGENTS.md` should be indexed as a cross-agent instruction source, but any lift claim still needs paired outcome rows; instruction presence alone is not fix-correctness evidence. |
| [On the Impact of AGENTS.md Files on the Efficiency of AI Coding Agents](https://arxiv.org/abs/2601.20404) | Early empirical evidence associates AGENTS.md files with lower median runtime and output-token use, but not with a proven fix-correctness lift. Parallax should measure outcome quality separately from efficiency. |
| [On the Use of Agentic Coding Manifests](https://arxiv.org/abs/2509.14744) | Agent manifests commonly contain operational commands, implementation notes, and architecture. That maps to Parallax intent nodes, but also shows why content quality and structure matter. |
| [Bundle-value evaluation](bundle-value-evaluation.md) | The A1 eval already measures bundle value. Repo-intent should be a paired sub-study, not a replacement for the raw-telemetry control. |
| [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md) | Model drift and task contamination also apply to repo-intent claims; every claim needs expiry and rerun triggers. |

### Claim Levels

| Level | Meaning | Minimum evidence |
| --- | --- | --- |
| `not_measured` | No repo-intent eval exists. | Default state. |
| `intent_manifest_indexed` | Repo-held intent sources can be discovered and normalized. | Source inventory covers README, docs, ADRs/decision records, roadmap/tasks, issue refs, and agent instruction files when present. |
| `instruction_surface_indexed` | Tool-specific instruction surfaces are discovered with load scope, precedence, target agents, import/symlink refs, and local/private status. | AGENTS/Copilot/Claude/Gemini/Cursor-style instruction surfaces are inventoried for tested repos. |
| `intent_edges_projected` | Intent nodes and edges can appear in bundles without breaking schema or redaction. | Bundle projection rows include intent nodes, source refs, edge strengths, and missing-intent flags. |
| `intent_safety_pass` | Intent context does not leak private material, dereference raw refs, or override policy. | Redaction, source-field, projection, prompt-injection, stale/conflict, and gold-patch leakage fixtures pass. |
| `degraded_mode_pass` | Runtime-only bundles remain valuable without repo intent. | Runtime-only arm still meets the A1 decision rule or stays within the allowed drop from the intent arm. |
| `intent_value_signal` | Repo intent shows a useful directional lift. | Paired run shows better constraint adherence, fewer unsupported claims, or lower investigation time without a resolved-rate drop. |
| `intent_value_pass` | Repo intent is a claimable multiplier. | Mixed task set, at least two model families, pre-registered rule passes, degraded mode still acceptable, and no safety failures. |
| `intent_overfit_risk` | Intent helps only operator-like repos or leaks task-specific hints. | Result narrows ICP and blocks broad market wording. |
| `claim_expired` | Prior result is stale. | Refresh trigger fired or max age elapsed. |
| `claim_failed` | Repo intent harms quality, leaks, or makes degraded mode too weak. | Any required fixture or pre-registered outcome fails. |

Initial claim level: `not_measured`.

### Evaluation Arms

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

### Result Artifacts

The durable result index lives at:

```text
docs/research/repo-intent-value-results.md
```

Each run stores immutable artifacts under:

```text
docs/research/repo-intent-value-runs/<run_id>/manifest.json
docs/research/repo-intent-value-runs/<run_id>/intent-source-inventory.jsonl
docs/research/repo-intent-value-runs/<run_id>/instruction-surface-inventory.jsonl
docs/research/repo-intent-value-runs/<run_id>/intent-node-ledger.jsonl
docs/research/repo-intent-value-runs/<run_id>/intent-edge-ledger.jsonl
docs/research/repo-intent-value-runs/<run_id>/context-arm-manifest.jsonl
docs/research/repo-intent-value-runs/<run_id>/task-arm-results.jsonl
docs/research/repo-intent-value-runs/<run_id>/constraint-audit.jsonl
docs/research/repo-intent-value-runs/<run_id>/unsupported-claim-audit.jsonl
docs/research/repo-intent-value-runs/<run_id>/intent-conflict-results.jsonl
docs/research/repo-intent-value-runs/<run_id>/source-field-policy-results.jsonl
docs/research/repo-intent-value-runs/<run_id>/projection-results.jsonl
docs/research/repo-intent-value-runs/<run_id>/redaction-results.jsonl
docs/research/repo-intent-value-runs/<run_id>/claim-ledger.jsonl
docs/research/repo-intent-value-runs/<run_id>/hashes.sha256
```

Do not commit private design docs, task descriptions, issue comments, internal
roadmap text, or full agent transcripts unless redacted and approved. Commit
hashes, normalized intent rows, bounded excerpts, and source-class labels.

### Run Manifest

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
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "projection_schema_version": "repo-intent-projection-vN",
  "instruction_surface_policy_version": "repo-intent-instruction-surface-vN",
  "intent_schema_version": "repo-intent-v1",
  "instruction_surface_count": 0,
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

### Minimum Row Schemas

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
  "source_field_policy_status": "pass|fail|not_applicable",
  "instruction_surface": "none|agents_md|claude_md|claude_local|claude_rules|copilot_repository|copilot_path|gemini_md|cursor_rules|other",
  "agent_products_targeted": ["codex", "copilot", "claude_code"],
  "load_scope": "repo|path|directory_subtree|local_user|managed_policy|unknown",
  "load_precedence": "nearest|root|path_match|imported|local_override|unknown",
  "local_or_private": false,
  "import_or_symlink_refs": [],
  "enforcement_mode": "context_only|external_hook_or_setting|unknown",
  "content_hash": "sha256:<hex>",
  "included": true
}
```

Instruction surface row:

```json
{
  "surface_id": "instr_001",
  "source_id": "intent_src_001",
  "agent_product": "codex|copilot|claude_code|gemini|cursor|other",
  "file_pattern": "AGENTS.md|CLAUDE.md|.github/copilot-instructions.md|.github/instructions/*.instructions.md|.claude/rules/*",
  "resolved_path": "AGENTS.md",
  "load_scope": "repo|path|directory_subtree|local_user|managed_policy|unknown",
  "load_precedence": "nearest|root|path_match|imported|local_override|unknown",
  "resolved_import_hashes": ["sha256:<hex>"],
  "local_or_private": false,
  "committed_to_repo": true,
  "available_to_agent": true,
  "context_only": true,
  "policy_enforced_elsewhere": false,
  "source_field_policy_status": "pass|fail|not_applicable",
  "redaction_check": "pass|fail",
  "staleness": "current|possibly_stale|stale"
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
  "instruction_only": false,
  "enforcement_mode": "context_only|external_hook_or_setting|unknown",
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
  "source_field_policy_status": "pass|fail|not_applicable",
  "instruction_surface_policy_hash": "sha256:<hex>",
  "raw_ref_dereferenced": false,
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

### Counting Rules

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
- Instruction files are context evidence, not enforced policy, unless a separate
  hook, setting, managed policy, or CI control proves enforcement.
- Source-field policy and projection policy must pass before any instruction
  file is bundled. Raw file refs, imports, or symlink targets must not be
  dereferenced by downstream agents.
- Local/private/user memory such as `CLAUDE.local.md`, local `.claude/` files,
  and user-level memory may be hashed for run provenance, but cannot support
  public product claims unless the tested agent actually loaded it and privacy
  approval allowed it.
- Imports and symlinks must resolve to committed content hashes when they affect
  the tested context. Unresolved imports fail `instruction_surface_indexed`.
- Path or nearest-file precedence must be recorded. Ambiguous or conflicting
  instructions create an `intent_conflict` row and cannot be silently collapsed.
- If an instruction conflicts with code, tests, telemetry, or a newer decision,
  the passing behavior is to surface the conflict; obeying stale prose is a
  failure.

### Pass Targets

Initial thresholds:

| Gate | Target |
| --- | --- |
| Intent projection | 100 percent of included intent nodes carry source refs, staleness, and redaction status. |
| Instruction inventory | 100 percent of discovered instruction surfaces carry load scope, precedence, target agent, import/symlink status, locality/privacy status, and enforcement mode. |
| Source/projection policy | 100 percent of bundled instruction surfaces pass source-field and projection-policy checks before C1/C2 context construction. |
| Degraded mode | C0 resolved rate is at least 80 percent of C1 or independently passes A1. |
| Intent value | C1 improves constraint adherence by >=10 percentage points or reduces unsupported claims by >=15 percent. |
| No quality regression | C1 does not lower resolved rate or root-cause accuracy versus C0. |
| Token budget | C1 median input tokens increase by <=15 percent unless quality lift is large enough to justify a narrower high-context tier. |
| Conflict handling | Seeded stale/conflicting intent is reported as a conflict in 100 percent of audited cases. |
| Safety | Zero private intent leaks and zero gold-patch leaks. |

### Refresh Triggers

Mark the claim `claim_expired` and rerun when any of these changes:

- A new model family, coding-agent scaffold, or repo-instruction mechanism is
  used in product claims.
- GitHub Copilot, Claude Code, Gemini, Cursor, Codex, or another target agent
  changes how repository instructions, memory files, imports, local overrides,
  or path precedence load.
- A target agent changes whether instruction-like content is merely context or
  externally enforced through hooks, settings, managed policy, CI, or sandbox
  controls.
- Parallax bundle schema, intent node schema, redaction policy, truncation
  policy, or source-ingest logic changes.
- A2 interviews show target users have materially different repo-intent hygiene
  than the tested corpus.
- Ninety days pass for public claims or a new A1 run supersedes the paired
  tasks.

### Product Wording

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
- "AGENTS.md enforces policy", "repo instructions are guardrails", or similar
  wording unless enforcement is proven outside the prose instruction file.

### Relationship To Other Research

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

### Bottom Line

Repo intent is a promising multiplier and possible moat, but it must not become
an untested dependency. Agent instruction files are useful context, not policy
enforcement by themselves. Parallax earns broad-market wording only if
runtime-only bundles work well; it earns repo-intent wording only if adding
cited, current, redacted intent improves agent work in paired runs.

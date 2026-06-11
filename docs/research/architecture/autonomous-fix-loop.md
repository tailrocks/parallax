# The Autonomous Fix Loop (Concept)

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-11. Status: **design, not measured** — every stage below carries the
claim-wording rules of its governing ledger. This note closes the loop-level gaps between
existing designs; it does not replace them.

> **One sentence.** A failure becomes an issue (Detect), the issue becomes a bounded evidence
> bundle (Context), the bundle wakes a fixer under an earned autonomy budget (Dispatch), the
> fixer proposes or lands a change (Fix — outside Parallax core), Parallax follows the change
> through CI, merge, deploy, and a recurrence window (Validate), and the recorded outcome
> adjusts evidence selection and the autonomy budget for that failure class (Learn).

```text
            ┌──────────────────────────────────────────────────────────────┐
            ▼                                                              │
  [Detect] ──► [Context] ──► [Dispatch] ──► [Fix] ──► [Validate] ──► [Learn]
  Detector     Bundler        Dispatcher    Fixer      Reconciler     Learner
  (Parallax)   (Parallax)     (Parallax)    (external) (Parallax)     (Parallax)
```

Ownership is unchanged from the [fixer boundary](../decisions/fixer-boundary.md): Parallax never
checks out repos, generates patches, pushes branches, merges, or mutates production. The fixer is
a separate component consuming the documented contracts.

## Stage 1 — Detect (new design surface)

Today every design assumes a human or agent *asks* for a bundle. The loop needs issue birth and
escalation to be event-driven. The Detector is a Parallax worker evaluating cheap streaming
predicates over already-normalized data (it reads the same rollups the cost layer produces — no
second pipeline). Note: GreptimeDB's trigger/alerting surface is Enterprise-only, so detection
logic lives in Parallax regardless of engine — which is where it belongs anyway, since triggers
must be engine-portable across the `StorageAdapter`.

Trigger taxonomy (v0):

| Trigger | Predicate (sketch) | Default action |
| --- | --- | --- |
| `new_fingerprint` | First sighting of a fingerprint in (project, environment) | Create issue; dispatch eligible |
| `frequency_spike` | Fingerprint rate > k× trailing baseline (EWMA) for sustained window | Escalate issue; dispatch eligible |
| `deploy_adjacent_regression` | `new_fingerprint` or `frequency_spike` within T of a deploy event whose release differs | Escalate + attach deploy edge; dispatch eligible, higher priority |
| `fix_regression` | Recurrence of a fingerprint inside an active post-merge watch window | Append `fix_worsened_issue`/recurrence outcome row; dispatch eligible at reduced autonomy |
| `slo_breach` | Error-rate/latency SLO predicate over rollups | Later; requires SLO config surface |

Policy per project: which triggers create issues (cheap, default all) versus which **dispatch**
(guarded, default only `new_fingerprint` + `deploy_adjacent_regression` at L1 diagnose). Every
trigger evaluation is itself recorded (trigger id, predicate inputs, decision) so detection is
auditable evidence, not folklore.

## Stage 2 — Context (already designed)

Unchanged: anchored bundle assembly per
[evidence-bundle-schema.md](evidence-bundle-schema.md),
[causal-reconstruction.md](causal-reconstruction.md), and the redaction pipeline
([capture/redaction.md](../capture/redaction.md)). The only loop-specific addition: a bundle
created by a trigger records its `trigger_ref`, so "why did the system look at this at all" is
part of the evidence chain.

## Stage 3 — Dispatch (new design surface)

The wake mechanism — how a fixer learns there is work. Four transports, one contract:

1. **Outbound webhook** (primary for automation): Parallax POSTs a `fix_candidate` event to a
   registered endpoint.
2. **Work-item creation** (primary for human-in-the-loop teams): create a GitHub/Linear issue
   containing the bundle reference; agents assigned to the tracker pick it up (the
   Copilot-for-Jira / OpenHands-label pattern).
3. **MCP/CLI pull** (interactive agents): `parallax issue list --dispatchable` /
   read-only MCP tool; no push at all.
4. **Queue** (fleet fixers, later): same payload on a durable stream.

`fix_candidate` payload (sketch — final schema rides the evidence-bundle versioning rules):

```json
{
  "event_type": "parallax.fix_candidate.v0",
  "project_id": "proj_checkout",
  "issue_id": "iss_8b21",
  "trigger": "deploy_adjacent_regression",
  "bundle_ref": "bndl_01J...",
  "canonical_bundle_hash": "sha256:...",
  "autonomy_budget": {
    "failure_class": "backend_error",
    "max_level": "L2_propose_patch",
    "basis": { "window_days": 30, "runs": 14, "accepted_rate": 0.71, "revert_rate": 0.0 }
  },
  "required_validation": ["cargo test -p checkout"],
  "expires_at": "2026-06-12T00:00:00Z",
  "idempotency_key": "iss_8b21:bndl_01J"
}
```

Rules: idempotent per (issue, bundle); rate-limited per project; the payload carries the
**autonomy budget** (computed, never configured upward past its earned level — see
[north-star-autonomous-fix-loop.md](../00-vision/north-star-autonomous-fix-loop.md) §3); the
bundle itself is fetched through the normal read-only surface, never inlined into the webhook
(keeps redaction/projection on one audited path).

## Stage 4 — Fix (external; contracts already designed)

The fixer consumes the bundle and writes back an append-only outcome record — both contracts are
fully specified in the [fixer boundary](../decisions/fixer-boundary.md) (fixer request contract,
`fixer_context` block, outcome record, gates). Nothing new here by design: the loop must not
grow private side-channels between Parallax and any one fixer.

## Stage 5 — Validate (the missing actor, now named: the Reconciler)

[deploy-change-context.md](../capture/deploy-change-context.md) already designs ingestion of
deploys, workflow runs, check runs, PR reviews, and merges. What no document owned was the actor
that *connects* those facts to fixer runs over time. The Reconciler is that actor — a Parallax
worker that maintains, per outcome record:

1. **CI linkage:** attach check/workflow conclusions for the exact fixer head SHA (statuses per
   the fixer outcome ledger counting rules; skipped/stale ≠ passing).
2. **Review/merge linkage:** append review decisions, merge or close, edit-before-merge.
3. **Deploy watch:** when a deploy whose release contains the fix commit reaches an environment
   where the issue occurred, open a **recurrence window** (default 7d, per outcome schema).
4. **Recurrence verdict:** fingerprint silent through the window → append
   `fix_addressed_issue` (strong only with merge + clean window, per existing edge semantics);
   fingerprint recurs → append recurrence row, emit the `fix_regression` trigger (Stage 1), and
   downgrade the prior success.

Everything appended, nothing overwritten — the outcome ledger's append-only rule is what makes
the Learner trustworthy.

## Stage 6 — Learn (new design surface)

Outcome rows must change future behavior, or the loop is theater. Three consumers, all
deterministic (no model retraining implied at this stage):

| Consumer | Input | Effect |
| --- | --- | --- |
| **Autonomy budget** | Trailing per-(project, failure_class) outcome stats | Promote/demote the max dispatch level (formula in the north-star note §3) |
| **Evidence selection** | Which bundle edges/nodes were cited by accepted vs rejected fixes | Re-weight neighborhood expansion and bundle inclusion priorities in the Bundler |
| **Hypothesis priors** | Accepted hypothesis classes per fingerprint family | Order ranked hypotheses for recurring families |

Each adjustment writes a dated row referencing the outcome rows that caused it — satisfying the
ledger rule that "fixer learns from outcomes" may only be claimed when an outcome row demonstrably
altered a policy, retrieval, or scoring decision.

## The cost architecture (the triangle's second vertex)

Loop completeness must not bankrupt retention. Three mechanisms, designed together:

1. **Tiering.** Hot (engine memtable/NVMe, hours–days) → warm (engine-native object storage,
   weeks–months) → cold (rollups + pinned slices only). Per-signal TTLs; raw traces age fastest,
   error events and rollups live longest. Engine-side mechanics per
   [storage-cost-and-tiering.md](../storage/greptimedb-vs-clickhouse/storage-cost-and-tiering.md)
   and [retention-and-ttl.md](../storage/greptimedb-vs-clickhouse/retention-and-ttl.md).
2. **Pre-aggregation on ingest.** Fingerprint counters, per-(service, env, release) error rates,
   and metric downsamples are computed at write time (engine continuous-aggregation/flow where
   portable, Parallax worker otherwise). The Detector and the UI read aggregates; raw data is for
   bundles. This is what keeps the interactive vertex cheap.
3. **Evidence pinning (new rule).** At bundle creation, every raw slice the bundle cites (spans,
   log windows, metric windows) is copied into the bundle's durable storage alongside the bundle
   JSON. TTL eviction then never breaks an audit trail: the fix that merged in March is still
   explainable in November from its pinned evidence, even though the raw firehose is long gone.
   Pinned bytes are tiny (bounded bundles) — completeness survives cheapness.

## A loop walk-through (concept fixture)

```text
T+0     deploy d_456 (release 1.42.0, commit abc123) → deploy event ingested
T+9m    OTLP: span payment.authorize ERROR + exception event; ERROR logs share trace_id
T+9m    error events derived, fingerprint fp_77 born → Detector: new_fingerprint AND
        deploy_adjacent_regression(d_456) → issue iss_91 created
T+10m   Bundler: bundle bndl_3f (anchor iss_91, deploy edge strong, trigger_ref recorded)
T+10m   Dispatcher: fix_candidate → fixer webhook; budget says L2 (class earned 71% accept)
T+22m   fixer returns outcome row: patch proposed, validation cargo test passed (L2 — no PR;
        human promotes to PR per policy)
T+1d    PR merged → Reconciler attaches review+merge rows
T+2d    deploy d_457 contains fix commit → recurrence window opens (7d)
T+9d    fp_77 silent → fix_addressed_issue appended (strong)
T+9d    Learner: backend_error accept-rate updates; deploy-edge weight reinforced;
        next fp_77-family dispatch may carry L3 budget
```

## Gate mapping (what must pass before any stage is claimable)

| Stage | Governing gate/ledger | Current level |
| --- | --- | --- |
| Detect | New: needs its own fixture ledger (trigger precision/recall on replayed telemetry) | not designed → this note is the design |
| Context | A1 bundle value, A4 correlation, A6 redaction | not_measured |
| Dispatch | Agent access surface gates (read-only, projection equivalence) | not_measured |
| Fix | Fixer outcome ledger (all rows) | not_measured |
| Validate | Deploy/change ingestion ledger + outcome counting rules | not_measured |
| Learn | `outcome_feedback_loop_pass` in the fixer outcome ledger | not_measured |

## Relationship to other research

- [North star and impossible triangle](../00-vision/north-star-autonomous-fix-loop.md) — why the
  loop is the destination and how autonomy is earned.
- [Fixer boundary](../decisions/fixer-boundary.md) — Stage 4's contracts and the L0–L5 ladder.
- [Deploy and change context](../capture/deploy-change-context.md) — Stage 5's raw facts.
- [Evidence bundle schema](evidence-bundle-schema.md) — Stage 2's artifact.
- [Integration contract](integration-contract.md) — how apps, CI, deploys, browsers, agents, and
  fixers physically attach to the loop.
- [Build roadmap](build-roadmap.md) — unchanged phases; Detect/Dispatch/Reconcile/Learn land as
  Phase-3/4 work, but their schemas are versioned with the core so V1 does not paint them out.
- PoC: [`poc/evidence-loop/`](../../../poc/evidence-loop/) proves the offline data plane of
  every loop stage except Fix (external by ADR): derivation/fingerprint/bundle/redaction with
  canonical hashes (Stage 2), the `deploy_adjacent_regression` trigger with SHA-match strength
  upgrade (Stage 1), the earned autonomy budget + `parallax.fix_candidate.v0` dispatch payload
  (Stage 3), the Reconciler's recurrence kernel (Stage 5), and the Learner: outcome-citation
  edge weights plus the loop-closure property that appending a reverted outcome row demotes the
  class budget L2→L1 through the same public API (Stage 6). Learned weights are wired back into
  bundle edge ordering, and both outward payloads ship draft JSON Schemas (`poc/evidence-loop/schema/`)
  that every emitted artifact validates against in tests — the first machine-checkable A3 artifact.

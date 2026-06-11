# evidence-loop-poc

Concept proof for the offline data plane of the
[autonomous fix loop](../../docs/research/architecture/autonomous-fix-loop.md)
(Detect→Context stages). Apache-2.0, Tailrocks. **Not product code; supports no
product claims** (see the PoC rule in [`AGENTS.md`](../../AGENTS.md)).

What it proves, end to end, with no network, database, or wall clock:

1. **Error derivation without a fourth signal** — Parallax `error_event` rows
   derived from OTLP JSON: span ERROR status + `exception` span events, plain
   ERROR log records, and `exception.*` attributes on log records (the encoding
   OTel moves exceptions toward after the 2026-03 Span Events deprecation).
2. **Encoding convergence** — the same exception arriving as a span event and
   as a log record produces the same fingerprint and one issue group.
3. **Deterministic grouping** — normalize volatile tokens (numbers, hex, UUIDs),
   fingerprint over (type, normalized message, top frame).
4. **Bounded evidence bundle** — anchor, typed nodes (error events, spans, log
   window), typed edges with strength (`error_in_span` strong, `log_in_trace`
   strong, `same_fingerprint`), explicit `missing_evidence`, a trigger record
   (`new_fingerprint`, dispatch-eligible).
5. **Redaction with a report** — seeded canaries (AWS key, bearer token, email)
   never reach the serialized bundle; the bundle carries machine-readable
   redaction counts.
6. **Canonical hashing** — sorted-key compact JSON (JCS-lite; no float values
   in this PoC) hashed with SHA-256, computed with the hash field absent;
   identical fixtures yield identical hashes.
7. **Deploy-adjacency trigger escalation** — a `parallax.deploy.v0` deploy
   event within the 30-minute window escalates the trigger from
   `new_fingerprint` to `deploy_adjacent_regression`, adds the deploy node,
   and upgrades the `deploy_preceded_issue` edge to **strong** when the
   deployed `vcs_sha` matches the service's `vcs.ref.head.revision`; an
   out-of-window deploy does not escalate and the gap is listed in
   `missing_evidence`.
8. **Reconciler recurrence kernel** — `reconcile_recurrence(fix_deploy,
   event_times, window, horizon)` returns `Recurred` / `Silent` / `WindowOpen`
   with an explicit observation horizon instead of a wall clock, covering the
   fix-held, fix-failed, and verdict-pending cases of the Validate stage.
9. **Earned autonomy budget** — `compute_budget(outcome_rows, failure_class)`
   implements the v0 promotion policy (L2: n≥5, accept≥0.6, zero reverts;
   L3: n≥10, accept≥0.7, revert+recurrence ≤0.05; edited = half credit; any
   redaction failure caps at L1; L4/L5 never emitted). The fixture history
   earns L2 and is blocked from L3 by one recurrence — autonomy comes from
   outcomes, not configuration.
10. **Dispatch payload** — `parallax.fix_candidate.v0` wake event per bundle:
    references the bundle (never inlines it), carries the computed budget,
    canonical bundle hash, required validation, a stable idempotency key, and
    a telemetry-anchored expiry. Byte-deterministic.
11. **Learner kernel** — `compute_edge_weights(outcome_rows)` turns evidence
    citations from outcome rows into Laplace-smoothed accept-rate lifts per
    edge type (fixture: `deploy_preceded_issue` lift 1.19, `temporal_proximity`
    0.36), with a report referencing the exact basis outcome IDs (dated-row
    rule). Loop closure is asserted directly: appending one reverted outcome
    row demotes the class budget from L2 back to L1 through the same public
    API — outcome rows demonstrably alter a policy decision, which is the
    fixer-outcome-ledger definition of "learning".

12. **Weights→retrieval wiring** — `apply_edge_weights(bundle, report)`
    reorders bundle edges by learned lift (highest first, unknown types 1.0,
    stable tie-break) and recomputes the canonical hash, so the learner's
    output demonstrably changes what an agent reads first.
13. **Published machine-readable schemas** — [`schema/`](schema/) holds JSON
    Schema (draft 2020-12) for `parallax.fix_candidate.v0` and
    `bundle-v0-poc`; tests validate every emitted artifact against them and
    assert a negative control (dropping `redaction_report` fails validation).
    These are draft artifacts for the A3 schema-adoption gate
    ([a3-schema-corpus.md](../../docs/research/validation/a3-schema-corpus.md));
    the production schema versioning policy lives there.

14. **Token-budget bounding** — `bound_bundle(bundle, 10_000)` enforces the
    bounded-bundle product rule (agent-context-integration's ~10K default):
    oldest log lines drop first, then stacktrace tails; every trim is recorded
    in `missing_evidence`; anchor-critical nodes (error events, spans, deploy)
    are never dropped; the canonical hash is recomputed; bounding is
    idempotent and under-budget bundles pass through untouched.
15. **Frequency-spike trigger kernel** — `frequency_spike(counts, …)` is the
    EWMA predicate from the trigger taxonomy: spike when the latest bucket
    exceeds k× the trailing baseline AND an absolute min-count floor (so
    near-zero baselines don't dispatch on noise); cold starts return
    insufficient-baseline (that's `new_fingerprint` territory).
16. **Pre-aggregation rollup (the cost vertex in miniature)** — `RollupStore`
    buckets error events into (fingerprint, minute) counters at write time;
    `spike_check` runs the full Detect chain raw events → rollup →
    zero-filled dense series → spike verdict. The test suite asserts the
    triangle's cost argument directly: 5,000 raw events compress >100× into
    the rollup the Detector actually reads.
17. **Cross-tier reconstruction** — the `fixtures/crosstier/` scenario: a
    browser span (`web-frontend`) and the backend span it caused (`checkout`)
    share one W3C trace; the bundle anchored on the backend exception contains
    span nodes from **both services**, a strong `span_child_of` edge from the
    backend SERVER span to its browser CLIENT parent, and one service-tagged
    log window interleaving browser breadcrumb-style logs with backend logs —
    "how did the user reach this error" answered across the tier boundary,
    schema-valid. Run it: `cargo run -- fixtures/crosstier`.
18. **CLI runs as first-class evidence + the run-anchored bundle** — the
    `fixtures/clirun/` scenario is the operator's Jackin multiplexer panic: a
    root span carrying `process.command_line`/`process.exit_code` becomes a
    `cli_invocation` evidence node with a strong `error_in_invocation` edge
    from the panic; `build_run_bundle` assembles the local-first
    `parallax run inspect` shape — anchored on `parallax.run_id` (anchor type
    `run`, trigger `manual`, never dispatch-eligible), DEBUG logs explaining
    the dirty-pane state included. Unknown run ids return nothing;
    run-anchored bundles validate against the published schema.
19. **Agent sessions as evidence** — `attach_agent_evidence` joins normalized
    agent-session records (tool, repo, produced commit, ordered action log)
    into a bundle when the session ended inside the relevance window. The
    fixture proves the causality chain in edges: the agent's
    `file_edit src/payment.rs` action → its session → the deploy carrying the
    session's commit (`agent_session_produced_change`, strong via SHA
    equality) → the error — plus the sharpest edge,
    `agent_edited_failing_file` (strong: the edited path appears in the
    failing stacktrace). Out-of-window/other-repo sessions stay out; action
    details pass redaction; "what did the agent do before this incident" is
    now answerable from the bundle.

With this, **all six loop stages have an executable kernel**: Detect (triggers),
Context (bounded bundle), Dispatch (budget + payload), Fix (external by ADR —
contracts only), Validate (recurrence), Learn (weights + budget feedback), and
the outward contract is machine-checkable.

Run:

```bash
cd poc/evidence-loop
cargo test          # the eighteen property tests
cargo run           # prints derivation/bundle/dispatch/learner summary, writes out/*.json
```

The schema here is `bundle-v0-poc`, a reduced shape of the real contract in
[`evidence-bundle-schema.md`](../../docs/research/architecture/evidence-bundle-schema.md);
field names follow it where the subset overlaps.

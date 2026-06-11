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

Run:

```bash
cd poc/evidence-loop
cargo test          # the eight property tests
cargo run           # prints derivation/bundle summary, writes out/*.json
```

The schema here is `bundle-v0-poc`, a reduced shape of the real contract in
[`evidence-bundle-schema.md`](../../docs/research/architecture/evidence-bundle-schema.md);
field names follow it where the subset overlaps.

# Retention and prune contract

- **Status:** Approved
- **Contract version:** 1
- **Decision date:** 2026-07-17
- **Approved by:** alexey@chainargos.com
- **Authority:** Operator unblock directive, 2026-07-17
- **Research refresh:** 2026-07-17, GreptimeDB 1.1 documentation

## Decision

`parallax prune` is a bounded, resumable lifecycle operation. It plans every
eligible deletion first, excludes active, unresolved, and pinned evidence, and
defaults to dry-run. Execution requires `--execute` plus interactive
confirmation; non-interactive use additionally requires `--yes`. A successful
run means every planned logical deletion completed or was already complete.
Partial cross-store work is reported as partial failure and resumes from a
durable journal; it is never reported as success and completed steps are not
rolled back by restoring deleted telemetry.

GreptimeDB TTL deletion is asynchronous and happens during background
compaction. Changing a table TTL with `ALTER TABLE` applies the new retention
threshold to existing data, but does not make physical byte reclamation
synchronous. Therefore prune reports logical rows/objects reclaimed separately
from measured physical bytes. Pending compaction or garbage collection is an
honest successful-logical/pending-physical outcome, never fabricated immediate
disk recovery. This follows the current GreptimeDB
[TTL documentation](https://docs.greptime.com/user-guide/manage-data/overview/),
[ALTER reference](https://docs.greptime.com/reference/sql/alter/), and
[compaction documentation](https://docs.greptime.com/user-guide/deployments-administration/manage-data/compaction/).

## Ownership and lifecycle matrix

| Data class | Owner | Default lifecycle | Normal-prune eligibility | Protection/cascade |
| --- | --- | --- | --- | --- |
| Raw traces | GreptimeDB native `opentelemetry_traces` plus native helper tables | Configured `traces_ttl`, default `7d` | TTL-expired rows; explicit bounded deletion only when the plan names the native identity/time predicate | Native TTL remains uniform; Plan 106 must materialize bounded pin-owned evidence before raw expiry |
| Raw logs | GreptimeDB native `opentelemetry_logs` | Configured `logs_ttl`, default `7d` | Same as traces | Same materialization rule; no per-row TTL exemption |
| Raw metrics | GreptimeDB native per-metric tables | Configured `metrics_ttl`, default `14d` | TTL-expired samples; never a custom raw table | Same materialization rule; catalog reconciliation covers existing tables and ingest hints cover new tables |
| Derived error events | GreptimeDB `error_events` extension | Configured `error_events_ttl`, default `30d` | TTL-expired rows | Pin owner materializes needed evidence; this table never owns mutable issue state |
| Invocation metric points | GreptimeDB `invocation_metric_points` extension | Signal-matched `metrics_ttl` | TTL-expired rows | Same materialization rule |
| Metric exemplars | GreptimeDB `metric_exemplars` extension | Signal-matched `metrics_ttl` | TTL-expired rows | Same materialization rule |
| Future derived extensions | GreptimeDB, only when justified by the native-table decision | Must declare which raw-signal TTL it follows at creation | No implicit forever retention | Pin materialization and lifecycle registration are mandatory |
| Issues | Turso `issues` | Unresolved issues retained; resolved issues eligible 30 days after the persisted resolution time added by Step 3 | Resolved grace elapsed | Deleting an issue cascades its bounded tag cache and owned rows in one Turso transaction |
| Issue buckets | Turso `issue_buckets` | Same lifecycle as owning issue | Only through owner cascade | Unresolved owner protects dependents |
| Issue occurrences | Turso `issue_occurrences` | Same lifecycle as owning issue; standalone ledger compaction remains bounded to 30 days | Owner cascade or ledger compaction that cannot change issue counters | Unresolved owner protects dependents |
| Invocations | Turso `invocations` | Active/open retained; terminal invocation eligible 30 days after persisted terminal time | Terminal grace elapsed | A live evidence pin that names the invocation protects it |
| Dashboards | Turso `dashboards` | Retained until explicit user deletion | Never selected by normal prune | User-owned product state, not cache |
| Investigations | Turso `investigations` | Retained until explicit user deletion | Never selected by normal prune | User-owned product state, not cache |
| Saved views | Turso `saved_views` | Retained until explicit user deletion | Never selected by normal prune | User-owned product state, not cache |
| Alert rules | Turso `alert_rules` | Alert-owner policy; no normal-prune TTL | Never selected by normal prune | Rule deletion owns dependent-state behavior |
| Alert rule states | Turso `alert_rule_states` | Same lifecycle as owning rule | Only through alert-owner cascade | Active rule protects state |
| Alert incidents | Turso `alert_incidents` | Alert-owner policy; no lifecycle inferred here | Never selected by normal prune | Incident history remains intact until an approved alert lifecycle revision |
| Alert destinations | Turso `alert_destinations` | Explicit user deletion only | Never selected by normal prune | Referenced destinations cannot be surprise-deleted |
| Alert delivery events | Turso `alert_delivery_events` | Alert-owner retry/audit policy | Never selected by normal prune | Pending delivery and audit ownership remain intact |
| Alert checks | Turso `alert_checks` | Existing bounded newest-per-rule owner policy | Never selected by normal prune | Rule owner controls bounded audit rows |
| Spool frames and segments | Local disk | Existing configured maximum age and total bytes | Automatic reaper; manual prune includes all closed segments and truncates active segments safely | In-flight append/rotation owns its synchronization boundary |
| Pinned evidence | Plan 106 owner; metadata in Turso and a bounded materialized evidence artifact outside expiring native rows | Retained until explicitly unpinned or the pin's own expiry | Never while reachable from a live pin | Reachability protects the materialized artifact; it does not alter uniform native raw-table TTLs |

No data class silently inherits an infinite lifetime. Conversely, normal prune
never deletes unresolved issues, active invocations, saved dashboards or
investigations, alert configuration, or live pinned evidence.

## Legal and user expectations

Retention settings are destructive user policy. Product defaults may bound raw
telemetry and spool data exactly as disclosed, but must never surprise-delete
user-authored saved state, unresolved issues, active invocations, alert
configuration/audit state, or live pinned evidence. Dry-run must disclose each
eligible class, cutoff, exclusion, and estimated effect before confirmation.
The command must not imply legal-hold, regulatory archive, backup, or secure
erasure guarantees: Parallax V1 provides configured operational retention only.
Operators remain responsible for retention periods required by their law,
policy, contracts, and backup regime.

## Deterministic plan and bounds

The plan is computed from one explicit cutoff and one configuration snapshot.
Each item contains class, owner/store, table or object set, cutoff, row/object
estimate, byte estimate when measurable, protection exclusions, and warnings.
Items sort by owner, class, table/object identity, then cutoff. Catalog scans,
rows, and output are independently capped; exceeding a cap fails closed and
asks for a narrower scope instead of silently omitting candidates.

Dry-run and execution use the same immutable plan identity. Execution refuses
when the current configuration, protection generation, or catalog fingerprint
differs; the user must regenerate the plan. This makes dry-run eligibility
equal to executed eligibility without freezing concurrent ingest. Newly
expired data waits for the next plan.

## Cross-store recovery

Before the first destructive step, Turso records a plan journal containing the
plan identity and each store-owned step. A step transitions
`planned -> executing -> complete`; retries re-check the target and treat an
already-absent target as complete. GreptimeDB and local-disk work cannot join a
Turso transaction, so the journal is a saga record, not a false distributed
transaction. Any failure stops later dependent steps, returns non-zero, names
completed/pending work, and leaves the plan resumable.

Metadata owner cascades run transactionally before their journal step becomes
complete. Telemetry deletion never cascades into metadata. This direction
prevents a storage failure from deleting the only durable record that explains
what was attempted.

## Native metric TTL reconciliation

Startup and prune enumerate the bounded native metric catalog, excluding
system, native non-metric, and Parallax extension tables. Every recognized
native metric table is compared with configured `metrics_ttl`; drift is fixed
with native `ALTER TABLE ... SET 'ttl' = ...`. Tables created after the catalog
snapshot receive the same TTL through the OTLP creation hint and are discovered
on the next reconciliation. A second bounded catalog fingerprint check detects
creation races before success is reported. Unsupported native behavior is an
upstream/fix-forward failure, not permission to create a custom raw table.

## Compatibility and migration

The old no-argument command immediately pruned only spool files. Contract v1
replaces that incomplete behavior: no-argument `parallax prune` produces the
all-class dry-run plan and changes nothing. Operators use
`parallax prune --execute` and confirm, or add `--yes` in non-interactive
automation. Human and machine output identify contract version 1, plan ID,
logical result, measured physical bytes, pending compaction, exclusions, and
partial failure state.

Configuration keeps the shipped per-signal TTL keys and defaults. New prune
planning/journal limits are additive. Existing data is not rewritten; native
table TTL reconciliation applies configured policy to existing tables, and
Turso rows become eligible only under the state/grace rules above.

## Required proof before implementation closure

- Decision-policy validation rejects missing, malformed, draft, rejected,
  digest-mismatched, or incomplete contracts.
- Identical dry-run/execution eligibility fixtures cover every matrix row,
  cutoff edges, active/unresolved/pinned negatives, and catalog/output caps.
- Turso tests cover cascade, journal restart, idempotent retry, and partial
  Greptime/local-disk failure without false success.
- Live GreptimeDB tests cover existing/new metric tables, TTL `ALTER`, table
  creation races, bounded catalog exclusion, manual flush/compaction, and the
  distinction between logical row disappearance and measured physical bytes.
- CLI snapshots cover default dry-run, confirmation, `--yes`, JSON output,
  progress narration, partial failure, and pending physical reclaim.

Any change to ownership, default TTLs, grace periods, pin precedence,
confirmation, recovery, or reclaim truthfulness requires a new approved
contract version and compatibility evidence.

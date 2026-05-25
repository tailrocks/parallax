# Self-Hosted Simplicity Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This ledger turns the
[Self-hosted simplicity gate](self-hosted-simplicity-gate.md) into auditable
claim levels. The gate defines what to measure; the
[Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md)
pins the competitor versions and service shapes to refresh; this ledger defines
the run artifacts, row schemas, counting rules, expiry triggers, and product
wording before Parallax can claim a simple self-hosted tiny tier.

Current status: `not_measured`.

Central rule:

> No "simple to self-host" or "simpler than self-hosted Sentry" claim until a
> fresh-VM run proves first useful bundle time, service count, resource budget,
> Sentry/OTLP ingest smoke, restart durability, backup/restore, upgrade path,
> and redaction smoke against current baselines.

## Current Source Snapshot

| Source | Ledger consequence |
| --- | --- |
| [Sentry self-hosted docs](https://develop.sentry.dev/self-hosted/) | Sentry is the complexity baseline: Docker/Docker Compose plus scripts, no dedicated support, minimum 4 CPU cores, 16 GB RAM plus 16 GB swap, and larger installs becoming custom. |
| [Sentry self-hosted 26.5.0 release](https://github.com/getsentry/self-hosted/releases/tag/26.5.0) | The Sentry baseline must pin a real release and count the exact Compose graph from that tag, not a floating `main` checkout. Release-note action items also count: 26.5.0 requires a manual feature-flag update before `install.sh`, adds a new Docker container for Launchpad-powered mobile features, notes an objectstore gap for self-hosted Snapshots, and flags a weak hardcoded default `LAUNCHPAD_RPC_SHARED_SECRET`. |
| [SigNoz Docker install docs](https://signoz.io/docs/install/docker/) and [SigNoz v0.125.1 release](https://github.com/SigNoz/signoz/releases/tag/v0.125.1) | SigNoz is the compact OTLP-native baseline; Parallax must beat it on first bundle, Sentry migration, and dependency count, not only on Sentry's larger footprint. |
| [OpenObserve getting started](https://openobserve.ai/docs/getting-started/) and [OpenObserve v0.90.2 release](https://github.com/openobserve/openobserve/releases/tag/v0.90.2) | OpenObserve is the single-node Rust/self-hosted simplicity pressure test; every extra Parallax process needs evidence-bundle value. |
| [Bugsink Docker install](https://www.bugsink.com/docs/docker-install/) and [Bugsink 2.2.1 release](https://github.com/bugsink/bugsink/releases/tag/2.2.1) | Lightweight Sentry-compatible setup already exists; Parallax's simplicity claim must include cross-signal context, not just DSN-change ingestion. |
| [GreptimeDB standalone](https://docs.greptime.com/getting-started/installation/greptimedb-standalone/) | GreptimeDB is acceptable in the tiny tier only while it remains one standalone storage process with clear local persistence. |
| [Turso local development](https://docs.turso.tech/local-development) and [libSQL](https://github.com/tursodatabase/libsql) | Metadata must work as an embedded/local file or local libSQL path; required hosted Turso or Postgres would fail the tiny-tier claim. |
| [Rustrak server 0.2.5 release](https://github.com/AbianS/rustrak/releases/tag/%40rustrak/server%400.2.5) and [Rustrak latest release](https://github.com/AbianS/rustrak/releases/latest) | Rustrak is a monorepo with package-specific release tags; `releases/latest` currently resolves to `docs@0.1.16`, not the server package. Baseline refresh must record component-specific release streams, not only a generic latest URL. |
| [Traceway backend v1.7.27 release](https://github.com/tracewayapp/traceway/releases/tag/backend/v1.7.27), [GoSnag main commits](https://github.com/darkspock/gosnag/commits/main/), and [Urgentry v0.2.12 release](https://github.com/urgentry/urgentry/releases/tag/v0.2.12) | Lightweight challengers are versioning differently: component releases, no-release moving `main`, and published tiny/split deployment modes. The comparison ledger must mark release-stream confidence and moving-target risk per competitor. |
| [Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md) | This is the current baseline manifest. Measured runs must refresh it before claiming results because release tags, docs, release-note action items, and service graphs move quickly. |

## Claim Levels

| Level | Meaning | Minimum evidence |
| --- | --- | --- |
| `not_measured` | No current self-hosted simplicity run exists. | Default state. |
| `baseline_inventory_current` | Competitor versions, install paths, and service shapes were refreshed. | Source snapshot rows for Sentry, SigNoz, OpenObserve, at least one lightweight Sentry-compatible challenger, GreptimeDB, and Turso/libSQL. |
| `parallax_install_smoke` | Parallax tiny tier starts from public docs on a clean VM. | Command ledger and service inventory show no private steps or hidden dependencies. |
| `first_bundle_under_15m` | Fresh VM reaches the first useful issue context bundle inside the gate target. | Wall-clock row from first command to `parallax issue context <issue-id>` with bundle hash. |
| `service_budget_pass` | Tiny tier stays inside the long-running service budget. | Steady-state inventory shows no more than three required services and no broker/cache/Postgres/MCP sidecar. |
| `resource_budget_pass` | Tiny sample works on the target small VM. | Resource samples on 2 vCPU / 4 GB RAM pass documented idle and smoke-load limits. |
| `ingest_paths_pass` | The scoped Sentry and OTLP paths work without extra infrastructure. | Sentry Rust SDK event, OTLP trace, OTLP log, and OTLP metric are accepted and appear in the first bundle or missing-evidence report. |
| `restart_durability_pass` | Ordinary stop/start does not lose the issue context. | Restart row proves event, issue, trace link, raw ref, and bundle remain queryable. |
| `backup_restore_pass` | Sample data can be exported or snapshotted and restored into a clean instance. | Backup/restore row shows steps, elapsed time, restored hashes, and operator-visible caveats. |
| `upgrade_path_pass` | Tiny tier has a documented upgrade and rollback rehearsal. | Upgrade row proves one binary/image replacement plus explicit migrations and rollback notes. |
| `redaction_smoke_pass` | Seeded setup/event/log/CLI secrets stay out of the bundle and transcript. | Redaction smoke row reports zero visible canary leaks in JSON, Markdown, and committed run artifacts. |
| `sentry_comparison_pass` | Parallax beats self-hosted Sentry for the first useful evidence-bundle job. | Current Sentry baseline row plus Parallax row show lower setup time, service count, and resource burden for the scoped job. |
| `lightweight_comparison_pass` | Parallax remains close enough to lightweight challengers while covering more context. | Bugsink/Rustrak/Traceway/GoSnag/Urgentry comparison row records the tradeoff honestly. |
| `tiny_tier_self_hosted_claim` | Parallax can claim the tested tiny-tier self-hosted workflow. | All required Parallax rows pass and at least Sentry plus one lightweight challenger comparison is current. |
| `claim_expired` | A prior claim is stale. | Refresh trigger fired or max age elapsed. |
| `claim_failed` | A required fixture failed. | Any gate miss that changes allowed product wording. |

Initial claim level: `not_measured`.

## Result Artifacts

The durable result index lives at:

```text
docs/research/self-hosted-simplicity-results.md
```

Each run stores immutable artifacts under:

```text
docs/research/self-hosted-simplicity-runs/<run_id>/manifest.json
docs/research/self-hosted-simplicity-runs/<run_id>/source-snapshot.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/release-note-risk-results.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/command-ledger.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/service-inventory.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/port-config-secret-inventory.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/resource-samples.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/ingest-smoke-results.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/first-bundle-results.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/restart-durability-results.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/backup-restore-results.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/upgrade-results.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/redaction-smoke-results.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/comparison-scorecard.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/claim-ledger.jsonl
docs/research/self-hosted-simplicity-runs/<run_id>/hashes.sha256
```

Do not commit raw install logs, credentials, DSNs, generated admin tokens, IP
addresses, raw event payloads, raw stack traces, or unredacted Docker env files.
Commit redacted excerpts, hashes, and enough command metadata to reproduce the
run.

## Run Manifest

`manifest.json` must identify the environment, tested versions, and policy set:

```json
{
  "schema_version": "self-hosted-simplicity-v1",
  "run_id": "self-hosted-2026-05-25T130000Z",
  "research_date": "2026-05-25",
  "runner": "operator",
  "host_profile": {
    "provider": "local_vm",
    "os": "Ubuntu LTS",
    "cpu": "2 vCPU",
    "ram_gb": 4,
    "disk_gb": 40,
    "docker_version": "x.y.z",
    "compose_version": "x.y.z"
  },
  "network_profile": "public_internet",
  "parallax": {
    "git_commit": "<parallax_commit_sha>",
    "install_doc_hash": "sha256:<hex>",
    "compose_or_binary_hash": "sha256:<hex>"
  },
  "baselines": [
    {"system": "sentry_self_hosted", "version": "26.5.0", "release_stream": "self-hosted"},
    {"system": "signoz", "version": "v0.125.1", "release_stream": "signoz"},
    {"system": "openobserve", "version": "v0.90.2", "release_stream": "openobserve"},
    {"system": "bugsink", "version": "2.2.1", "release_stream": "bugsink"},
    {"system": "rustrak", "version": "@rustrak/server@0.2.5", "release_stream": "server_package"},
    {"system": "traceway", "version": "backend/v1.7.27", "release_stream": "backend"},
    {"system": "gosnag", "version": "418b8b1", "release_stream": "main_no_release"},
    {"system": "urgentry", "version": "v0.2.12", "release_stream": "urgentry"}
  ],
  "policies": {
    "redaction_policy": "a6-default-deny-vN",
    "bundle_schema_version": "0.1.0",
    "self_hosted_gate_version": "v1"
  },
  "result": "pass"
}
```

## Minimum Row Schemas

Source snapshot row:

```json
{
  "system": "sentry_self_hosted",
  "source_url": "https://github.com/getsentry/self-hosted/releases/latest",
  "latest_url_effective": "https://github.com/getsentry/self-hosted/releases/tag/26.5.0",
  "release_stream": "self-hosted",
  "resolved_version": "26.5.0",
  "resolved_at": "2026-05-25T13:00:00Z",
  "install_doc_url": "https://develop.sentry.dev/self-hosted/",
  "artifact_hash": "sha256:<hex>",
  "release_note_action_items": 0,
  "release_note_security_notes": 0,
  "moving_target": false,
  "result": "pass"
}
```

Release-note risk row:

```json
{
  "system": "sentry_self_hosted",
  "resolved_version": "26.5.0",
  "risk_type": "manual_preinstall_step|new_service_or_container|default_secret|unsupported_self_hosted_feature|moving_release_stream",
  "source_url": "https://github.com/getsentry/self-hosted/releases/tag/26.5.0",
  "operator_action_required": true,
  "affects_service_count": false,
  "affects_secret_safety": false,
  "summary": "Short redacted release-note summary.",
  "counting_decision": "record_only|affects_gate|excludes_claim"
}
```

Command ledger row:

```json
{
  "system": "parallax",
  "step": 4,
  "phase": "install",
  "command_class": "docker_compose_up",
  "started_at": "2026-05-25T13:04:00Z",
  "finished_at": "2026-05-25T13:06:20Z",
  "exit_code": 0,
  "manual_intervention": false,
  "redacted_output_hash": "sha256:<hex>"
}
```

Service inventory row:

```json
{
  "system": "parallax",
  "phase": "steady_state",
  "long_running_services": ["parallax-server", "greptimedb"],
  "init_services": [],
  "required_external_services": [],
  "exposed_ports": [4317, 4318, 8080],
  "service_count_pass": true
}
```

Resource sample row:

```json
{
  "system": "parallax",
  "phase": "idle_after_ingest",
  "sampled_at": "2026-05-25T13:12:00Z",
  "rss_mb_total": 1180,
  "cpu_percent_total": 6.4,
  "disk_bytes_used": 734003200,
  "resource_budget_pass": true
}
```

Ingest smoke row:

```json
{
  "system": "parallax",
  "signal": "sentry_rust_error",
  "sample_app": "parallax-rust-smoke",
  "sent_at": "2026-05-25T13:08:00Z",
  "accepted": true,
  "normalized_row_present": true,
  "bundle_ref_present": true,
  "extra_infrastructure_required": false,
  "result": "pass"
}
```

First bundle row:

```json
{
  "system": "parallax",
  "first_command_at": "2026-05-25T13:00:00Z",
  "first_useful_bundle_at": "2026-05-25T13:11:42Z",
  "elapsed_seconds": 702,
  "bundle_hash": "sha256:<hex>",
  "contains_grouped_error": true,
  "contains_redaction_report": true,
  "missing_evidence_reported": true,
  "under_15m": true
}
```

Restart durability row:

```json
{
  "system": "parallax",
  "restart_command_class": "docker_compose_restart",
  "event_present_after_restart": true,
  "issue_present_after_restart": true,
  "trace_link_present_after_restart": true,
  "raw_ref_present_after_restart": true,
  "bundle_hash_after_restart": "sha256:<hex>",
  "result": "pass"
}
```

Backup/restore row:

```json
{
  "system": "parallax",
  "backup_command_class": "documented_snapshot",
  "restore_target": "clean_vm",
  "elapsed_seconds": 420,
  "restored_bundle_hash": "sha256:<hex>",
  "manual_database_expertise_required": false,
  "result": "pass"
}
```

Upgrade row:

```json
{
  "system": "parallax",
  "from_version": "0.1.0-a",
  "to_version": "0.1.0-b",
  "migration_steps": 1,
  "rollback_documented": true,
  "bundle_hash_after_upgrade": "sha256:<hex>",
  "result": "pass"
}
```

Comparison scorecard row:

```json
{
  "system": "parallax",
  "baseline": "sentry_self_hosted",
  "time_to_first_error_or_bundle_seconds": 702,
  "baseline_time_seconds": 3600,
  "long_running_services": 2,
  "baseline_long_running_services": 72,
  "recommended_ram_gb": 4,
  "baseline_recommended_ram_gb": 16,
  "scoped_job": "first redacted issue context bundle",
  "comparison_pass": true
}
```

Claim ledger row:

```json
{
  "claim_level": "tiny_tier_self_hosted_claim",
  "run_id": "self-hosted-2026-05-25T130000Z",
  "scope": "single-node tiny tier, first issue context bundle",
  "granted_at": "2026-05-25T13:30:00Z",
  "expires_at": "2026-07-24T13:30:00Z",
  "result": "pass"
}
```

## Counting Rules

- Start from a fresh VM with Docker installed and no product data. Anything else
  is a local smoke test, not a self-hosted simplicity result.
- Measure Parallax from first documented install command to first useful
  `parallax issue context <issue-id>` bundle, not to first web page.
- Measure competitors using their official install path and their own first
  useful result: captured issue/error for error trackers, usable query/result
  for observability systems.
- Pin product code, image tags, docs hashes, and Compose files. Do not use
  estimates in the scorecard.
- Pin the correct release stream. For monorepos or package-specific releases,
  generic `releases/latest` is not enough unless it resolves to the component
  being measured. Moving `main` baselines must be marked as lower-confidence.
- Record release-note action items, new containers/services, default-secret
  notes, unsupported self-hosted features, and manual pre-install steps as
  release-note risk rows before counting a baseline as current.
- Count long-running services separately from init/migration containers, but
  record both.
- Any required broker, Redis, Postgres, hosted Turso, external Collector,
  Kubernetes, or MCP sidecar fails `service_budget_pass` for the tiny tier.
- Redaction is part of deployment simplicity. A transcript or bundle that leaks
  generated tokens, DSNs, admin passwords, env vars, or sample secrets cannot
  pass.
- Backup/restore must restore into a clean instance. Copying a working data
  directory without documented steps is not enough.
- Upgrade must include rollback notes. A version bump that works only from a
  private local build is not a claimable path.
- A pass is scoped to the host profile, exact versions, public docs, sample
  workload, and Parallax commit in the manifest.

## Required Pass Set

`tiny_tier_self_hosted_claim` requires all of these:

| Gate | Target |
| --- | --- |
| First useful Parallax bundle | <= 15 minutes from a clean VM with Docker already installed. |
| Required long-running services | <= 3, with no required broker/cache/Postgres/external Collector/MCP sidecar. |
| Minimum demo resources | 2 vCPU / 4 GB RAM for the tiny sample workload. |
| Setup shape | One install command or one Compose file, one generated config, one admin token, visible DSN and OTLP endpoints. |
| Sentry smoke | Latest scoped Rust Sentry SDK event path works by DSN change inside the supported envelope subset. |
| OTLP smoke | Trace, log, and metric accepted without deploying a separate Collector. |
| Restart durability | Event, issue, trace link, raw ref, and bundle survive ordinary restart. |
| Backup/restore | Clean-instance restore for sample data in <= 10 minutes. |
| Upgrade rehearsal | Binary/image replacement plus explicit migration and rollback notes. |
| Redaction smoke | Zero seeded canary leaks in bundle JSON, Markdown, and committed run artifacts. |

## Refresh Triggers

Mark the claim `claim_expired` and rerun when any of these changes:

- Sentry, SigNoz, OpenObserve, Bugsink, Rustrak, Traceway, GoSnag, Urgentry, or
  another measured baseline publishes a new relevant release or changes its
  install path.
- A measured baseline release note adds a service/container, manual install
  action, default-secret warning, unsupported self-hosted feature, or security
  caveat relevant to first useful output.
- Parallax adds, removes, or splits a service; changes GreptimeDB, metadata,
  raw retention, CLI, API, auth, or redaction setup; or requires a new external
  dependency.
- Docker, Docker Compose, host OS, minimum VM profile, or public installation
  docs change materially.
- The Sentry SDK fixture subset, OTLP receiver subset, A6 redaction policy, or
  evidence bundle schema changes.
- Sixty days pass for public product wording; ninety days pass for internal
  planning claims.

## Product Wording

Allowed before measurement:

> Parallax is designed to target a smaller self-hosted footprint than
> self-hosted Sentry, but the current simplicity claim is unmeasured.

Allowed after `first_bundle_under_15m`:

> In the tested environment, Parallax reached its first redacted issue context
> bundle in under 15 minutes.

Allowed after `tiny_tier_self_hosted_claim`:

> For the tested tiny-tier workflow, Parallax is simpler to self-host than
> self-hosted Sentry and remains close to lightweight error trackers while
> producing a broader evidence bundle.

Avoid:

- "Drop-in Sentry replacement."
- "One-command install" unless the command ledger proves it.
- "Production-ready HA."
- "Simpler than every observability product."
- "No operations required."
- "Air-gapped ready" unless the run used an air-gapped fixture.
- "Self-hosted claim" without naming the host profile, versions, and workload.

## Relationship To Other Research

- [Self-hosted simplicity gate](self-hosted-simplicity-gate.md) defines the
  measurement protocol and pass/fail thresholds.
- [Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md)
  supplies the current version and service-shape manifest that each run must
  refresh.
- [A7 scope discipline ledger](a7-scope-discipline-ledger.md) uses service count
  and dependency rows to keep the tiny tier from drifting.
- [A5 stack decision ledger](a5-stack-decision-ledger.md) consumes the deployment
  result before making stack-default claims.
- [Sentry SDK compatibility ledger](sentry-sdk-compatibility-ledger.md) and
  [OTLP conformance ledger](otlp-conformance-ledger.md) provide the protocol
  claims used by ingest smoke rows.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) controls when
  redaction smoke can become an agent-visible safety claim.

## Bottom Line

The self-hosted wedge is not a slogan. It is a timed, reproducible clean-VM run
that produces a useful redacted bundle with fewer services and less operator
burden than the current baselines. Until this ledger is green, Parallax can say
it is designed for low-ops self-hosting; it cannot claim the tiny tier proves it.

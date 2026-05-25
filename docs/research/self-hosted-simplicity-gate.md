# Self-Hosted Simplicity Gate

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This operationalizes kill criterion 6 from the [verdict](verdict.md): the first
Parallax deployment must be meaningfully simpler than self-hosted Sentry.
"Simpler" is not a brand claim. It is a measured Phase 1 gate.

This gate also protects against a subtler failure: building a narrower product
than Sentry, SigNoz, or OpenObserve while accidentally inheriting their
operational shape. If Parallax needs a broker, multiple product services, a
separate relational database, and complex background workers before it can emit
one useful issue context bundle, the self-hosted wedge is not real.
Result rows, claim levels, and product wording for this gate live in the
[Self-hosted simplicity ledger](self-hosted-simplicity-ledger.md).

## Current Baselines

The source-linked version pins, install paths, and service-shape notes for this
gate now live in
[Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md).
Use that note as the measurement manifest and refresh it before each real run.

| System | Current official posture | Operational read for Parallax |
| --- | --- | --- |
| Sentry self-hosted | Sentry describes self-hosted as a minimal setup for simple use cases with no dedicated support, Docker/Docker Compose plus install scripts, 4 CPU cores, 16 GB RAM plus 16 GB swap, 20 GB disk, and a single-node graph that still includes databases, brokers, and product services. The current pinned release also has manual release-note action/security items. | This is the main complexity baseline. Parallax does not need to beat Sentry's feature depth; it must beat Sentry's first-deployment burden for the narrower error-context job, including operator-visible release-note work. |
| SigNoz self-hosted | Official Docker setup clones the repo and runs Docker Compose; the verification example shows `signoz`, `signoz-otel-collector`, ClickHouse, and ZooKeeper containers. Architecture docs center ClickHouse and the SigNoz OTel Collector. | SigNoz is easier than Sentry for OTel-native observability, but ClickHouse plus ZooKeeper is still heavier than the Parallax tiny tier should be. |
| OpenObserve single-node | The official quickstart offers binary and Docker single-node self-hosted paths with root credentials and port 5080; its architecture page separates larger deployments into router, ingester, compactor, querier, and alert manager roles. | OpenObserve proves a Rust, single-node observability product can be simple. Parallax must justify any extra complexity with Sentry compatibility, evidence bundles, and agent-safe context. |
| GreptimeDB standalone | GreptimeDB can run as one standalone binary or one Docker container, with local data persisted in a directory and HTTP/RPC/MySQL/Postgres ports exposed. | Acceptable for the tiny tier if it remains one storage process and passes freshness/cost gates. |
| Turso/libSQL metadata | Turso docs support local SQLite file development without an auth token, `turso dev --db-file` for a local libSQL server, and libSQL commits to embeddability without a network connection. | Metadata must default to an embedded/local file mode. A required hosted Turso dependency would violate the self-hosted tiny-tier claim. |

Sources:

- [Sentry self-hosted docs](https://develop.sentry.dev/self-hosted/)
- [Sentry self-hosted repository](https://github.com/getsentry/self-hosted)
- [Sentry self-hosted Docker Compose](https://github.com/getsentry/self-hosted/blob/master/docker-compose.yml)
- [SigNoz self-hosted install docs](https://signoz.io/docs/install/self-host/)
- [SigNoz Docker standalone docs](https://signoz.io/docs/install/docker/)
- [SigNoz architecture docs](https://signoz.io/docs/architecture/)
- [OpenObserve quickstart](https://openobserve.ai/docs/getting-started/)
- [OpenObserve architecture docs](https://openobserve.ai/docs/architecture/)
- [GreptimeDB standalone install docs](https://docs.greptime.com/getting-started/installation/greptimedb-standalone/)
- [Turso local development docs](https://docs.turso.tech/local-development)
- [libSQL repository](https://github.com/tursodatabase/libsql)

## Tiny-Tier Target

The Phase 1 tiny tier should be measured against this profile:

```text
parallax-server
greptimedb standalone
embedded/local metadata file
local WAL/raw retention directory
parallax CLI
```

Allowed only if explicitly optional:

- local object-store emulator for a durability test;
- reverse proxy or TLS terminator for production polish;
- hosted Turso, Postgres, Iggy, NATS, Redpanda, S3/R2/B2, or Kubernetes.

Not allowed in the default tiny tier:

- Kafka, ZooKeeper, Pulsar, or any required broker;
- Redis or a required cache service;
- Postgres as the required metadata store;
- ClickHouse as the required default store;
- multiple Parallax product services before the first issue context works;
- MCP as a separate required service.

## Measurement Protocol

Run the same protocol for Sentry self-hosted, SigNoz Docker, OpenObserve
single-node, representative lightweight Sentry-compatible challengers from the
[lightweight competitor watch](lightweight-sentry-compatible-competitor-watch.md),
and Parallax tiny tier.

1. Start from a fresh Ubuntu LTS VM with Docker installed and no product data.
2. Pin the exact version, commit, or image tag being tested.
3. Record release-stream confidence, release-note action items, new
   services/containers, default-secret warnings, and unsupported self-hosted
   features before running the install.
4. Follow the official install path without private knowledge.
5. Record wall-clock time from first command to usable UI/API.
6. Count long-running processes or containers after startup.
7. Record required CPU/RAM/disk recommendations and actual idle RSS/CPU.
8. Record exposed ports, config files, required secrets, and generated
   credentials.
9. Ingest one Sentry error event from the latest Rust Sentry SDK path, one OTLP
   trace, one OTLP log, and one OTLP metric from a small sample app.
10. Query the first issue context bundle through the CLI and HTTP API.
11. Restart all services and verify the issue, raw event, trace link, and bundle
    still exist.
12. Run the documented backup/export path and restore into a clean instance.
13. Run the documented upgrade path or dry-run upgrade and record the operator
    steps.

For Parallax, the "first useful moment" is not seeing a dashboard. It is this:

```text
parallax issue context <issue-id>
```

returns a bundle containing the grouped error, stack frames, release/environment,
trace link if present, nearby logs if present, redaction report, and missing-data
warnings.

## Pass/Fail Gates

| Gate | Pass target for Parallax tiny tier | Failure consequence |
| --- | --- | --- |
| Time to first useful bundle | <=15 minutes from a fresh VM with Docker already installed. Stretch target: <=10 minutes. | If this misses badly, the self-hosted wedge weakens and Phase 1 cannot claim "simpler than Sentry." |
| Required services | <=3 long-running services: Parallax, GreptimeDB, and optional local object-store emulator only when testing object storage. Embedded metadata counts as zero extra services. | If a broker, Redis, Postgres, or multiple Parallax services are required, narrow the MVP or change architecture. |
| Minimum demo resources | Works on a 2 vCPU / 4 GB RAM VM for the tiny sample workload, with documented headroom limits. | If the tiny sample needs Sentry-class resources, Parallax loses its low-ops promise. |
| Commands and config | One install command or one `compose.yml`, one generated config file, one admin token, visible DSN and OTLP endpoints. | If setup needs hidden tribal knowledge, the gate fails even if the services start. |
| Sentry migration proof | Existing Sentry Rust SDK can send an error event by changing DSN only, within the scoped envelope-event subset. | If SDK compatibility needs app rewrites, the migration wedge weakens. |
| OTLP proof | OTLP HTTP/gRPC path accepts the sample trace/log/metric without deploying a separate collector and matches the [OTLP receiver conformance gate](otlp-receiver-conformance-and-collector-equivalence.md) for the supported subset. | If an external collector is mandatory for the tiny tier, document why or defer the claim. |
| Restart durability | Stop/start all services without losing the event, issue grouping, trace link, and raw reference. | Data loss on ordinary restart blocks Phase 1. |
| Backup/restore clarity | A documented local snapshot/export restores into a clean instance in <=10 minutes for the sample data. | If backup requires custom database expertise, the tiny tier is not operator-simple. |
| Upgrade path | One binary/image replacement plus explicit migrations and rollback notes. | If upgrades resemble a bespoke multi-service migration, the Sentry comparison becomes unfavorable. |
| Secret safety | The sample bundle redacts seeded tokens in event fields, tags, logs, env, and CLI output. | The deployment gate cannot pass before the redaction gate is at least green on fixture data. |

## Comparison Scorecard

Use this scorecard in Phase 1 docs and release notes. Do not fill it with
estimates; fill it only after running the protocol above.
The current version and deployment-shape manifest is
[Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md).

| Metric | Sentry self-hosted | SigNoz Docker | OpenObserve single-node | Lightweight challenger | Parallax tiny tier |
| --- | --- | --- | --- | --- | --- |
| Version/tag tested | TBD | TBD | TBD | TBD | TBD |
| Time to usable UI/API | TBD | TBD | TBD | TBD | TBD |
| Time to first error/context result | TBD | TBD | TBD | TBD | TBD |
| Long-running services | TBD | TBD | TBD | TBD | TBD |
| Recommended CPU/RAM/disk | TBD | TBD | TBD | TBD | TBD |
| Measured idle RSS/CPU | TBD | TBD | TBD | TBD | TBD |
| Required external broker/cache | TBD | TBD | TBD | TBD | TBD |
| Backup/restore steps | TBD | TBD | TBD | TBD | TBD |
| Upgrade steps | TBD | TBD | TBD | TBD | TBD |
| Sentry SDK DSN-change path | TBD | TBD | TBD | TBD | TBD |
| OTLP path without extra collector | TBD | TBD | TBD | TBD | TBD |
| Evidence bundle available | N/A | N/A | N/A | N/A | TBD |

## Design Consequences

1. The tiny tier should use an in-process normalizer, grouping engine, evidence
   graph builder, HTTP API, and CLI projection inside `parallax-server`.
2. Metadata should be embedded/local by default. Postgres or hosted Turso can be
   a later production profile, not the first proof.
3. The local WAL/outbox is the default durability boundary. Iggy/NATS/Redpanda
   only enter after the [ingest replay gate](ingest-log-replay-and-backpressure-gate.md)
   proves the extra service is worth it.
4. MCP must not be a separate tiny-tier service. The day-one agent surface is the
   CLI and HTTP API described in
   [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md).
5. OpenObserve is the simplicity pressure test. If Parallax is less simple than
   OpenObserve while claiming a narrower first product, the architecture needs
   another cut.

## Kill Trigger

Phase 1 should stop or narrow if any of these happen:

- the tiny tier needs Kafka, ZooKeeper, Redis, Postgres, or ClickHouse to return
  the first issue context;
- the first bundle takes more than 15 minutes to reach on a clean VM after docs
  are followed exactly;
- backup/restore or upgrade requires undeclared database internals;
- the product cannot ingest one scoped Sentry SDK event and one OTLP sample
  without extra infrastructure;
- measured resource use is close to Sentry's official minimum before any real
  workload exists.

The standard is not "as full-featured as Sentry." The standard is "much easier
to self-host for the first useful Sentry-compatible, OTLP-native issue context."

Related competitive check: [Lightweight Sentry-compatible competitor watch](lightweight-sentry-compatible-competitor-watch.md).

Related scope check: [A7 scope discipline ledger](a7-scope-discipline-ledger.md)
owns the feature, dependency, protocol, and service-count rows that must stay
green before this simplicity gate can support a Phase 1 claim.

Related result contract: [Self-hosted simplicity ledger](self-hosted-simplicity-ledger.md)
defines the clean-VM run artifacts, row schemas, expiry triggers, and allowed
wording before any public simplicity claim.

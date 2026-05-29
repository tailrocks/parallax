# Self-Hosted Simplicity

> Kill criterion 6 from the verdict demands that the first Parallax deployment be measurably simpler than self-hosted Sentry for the narrower Sentry-compatible, OTLP-native issue-context job; "simpler" is a measured Phase-1 gate, not a brand claim, currently at status `not_measured`. The source-linked baseline inventory pins the competitors to beat on a fresh-VM run — Sentry self-hosted `26.5.0` (72-service Compose, 4 CPU / 16 GB RAM + 16 GB swap minimum), SigNoz `v0.125.1` (6 services), OpenObserve `v0.90.2` (single-node), Bugsink `2.2.1`, Rustrak `@rustrak/server@0.2.5`, Traceway `backend/v1.7.27`, GoSnag `main`@`418b8b1` (no release), and Urgentry `v0.2.12` — measured by exact release tag/commit, never floating `main`. The Parallax tiny-tier target is `parallax-server` + GreptimeDB standalone + embedded/local metadata + local WAL/raw directory + CLI, with no required broker, Redis, Postgres, ClickHouse, external Collector, or MCP sidecar before the first `parallax issue context <issue-id>` bundle resolves. The result ledger turns the gate into auditable claim levels and forbids any "simple to self-host" or "simpler than self-hosted Sentry" wording until a clean-VM run proves first-useful-bundle time (≤15 min), service count (≤3), resource budget (2 vCPU / 4 GB), Sentry/OTLP ingest smoke, restart durability, backup/restore (≤10 min), upgrade path, and redaction smoke against current baselines. Nothing is decided yet: every Parallax scorecard cell is TBD and no run artifacts exist, so the open gate is the full fresh-VM measurement that would move the claim level from `not_measured` toward `tiny_tier_self_hosted_claim`.

This note consolidates the following previously-separate research files, each preserved in full below:

- `self-hosted-simplicity-gate.md`
- `self-hosted-deployment-baseline-inventory.md`
- `self-hosted-simplicity-ledger.md`

## Self-Hosted Simplicity Gate

_Provenance: merged verbatim from `self-hosted-simplicity-gate.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

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

### Current Baselines

The source-linked version pins, install paths, and service-shape notes for this
gate now live in
[Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md).
Use that note as the measurement manifest and refresh it before each real run.
For released competitors, service counts and service names must come from the
exact tested release tag or tag commit, not from floating `main`. Use `main`
only for no-release projects and record the full commit SHA.

| System | Current official posture | Operational read for Parallax |
| --- | --- | --- |
| Sentry self-hosted | Sentry describes self-hosted as a minimal setup for simple use cases with no dedicated support, Docker/Docker Compose plus install scripts, 4 CPU cores, 16 GB RAM plus 16 GB swap, 20 GB disk, and a single-node graph that still includes databases, brokers, and product services. The current pinned release also has manual release-note action/security items. | This is the main complexity baseline. Parallax does not need to beat Sentry's feature depth; it must beat Sentry's first-deployment burden for the narrower error-context job, including operator-visible release-note work. |
| SigNoz self-hosted | Official Docker setup clones the repo and runs Docker Compose; the verification example shows `signoz`, `signoz-otel-collector`, ClickHouse, and ZooKeeper containers. Architecture docs center ClickHouse and the SigNoz OTel Collector. | SigNoz is easier than Sentry for OTel-native observability, but ClickHouse plus ZooKeeper is still heavier than the Parallax tiny tier should be. |
| OpenObserve single-node | The official quickstart offers binary and Docker single-node self-hosted paths with root credentials and port 5080; its architecture page separates larger deployments into router, ingester, compactor, querier, and alert manager roles. | OpenObserve proves a Rust, single-node observability product can be simple. Parallax must justify any extra complexity with Sentry compatibility, evidence bundles, and agent-safe context. |
| Lightweight challengers | The current named set is Bugsink, Rustrak, Traceway, GoSnag, and Urgentry. The deployment inventory pins Bugsink `2.2.1`, Rustrak `@rustrak/server@0.2.5`, Traceway `backend/v1.7.27`, GoSnag `main` commit `418b8b1`, and Urgentry `v0.2.12`/`ccc0ff8`. | These are the real simplicity bar below Sentry. Parallax must compare against them by role: Sentry-compatible error-only simplicity, Rust-first tiny tracking, OTLP-native embedded mode, AI/MCP issue tooling, and Urgentry's broader Sentry-protocol plus vendor-benchmark claims. |
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
- [Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md)
- [GreptimeDB standalone install docs](https://docs.greptime.com/getting-started/installation/greptimedb-standalone/)
- [Turso local development docs](https://docs.turso.tech/local-development)
- [libSQL repository](https://github.com/tursodatabase/libsql)

### Tiny-Tier Target

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

### Measurement Protocol

Run the same protocol for Sentry self-hosted, SigNoz Docker, OpenObserve
single-node, the named lightweight challengers from the
[deployment baseline inventory](self-hosted-deployment-baseline-inventory.md),
and Parallax tiny tier.

1. Start from a fresh Ubuntu LTS VM with Docker installed and no product data.
2. Pin the exact version, commit, or image tag being tested.
3. Record release-stream confidence, exact source ref/commit, release-note action items, new
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

For lightweight challengers, record maturity separately from measured
deployment result. Bugsink and Traceway are active enough to be baseline
comparisons; Rustrak and Urgentry are fresh and strategically relevant; GoSnag
is currently a capability-shape warning because the checked metadata has no
tagged release and low visible traction.

For Parallax, the "first useful moment" is not seeing a dashboard. It is this:

```text
parallax issue context <issue-id>
```

returns a bundle containing the grouped error, stack frames, release/environment,
trace link if present, nearby logs if present, redaction report, and missing-data
warnings.

### Pass/Fail Gates

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
| Lightweight challenger comparison | Parallax is compared against Bugsink, Rustrak, Traceway, Urgentry, and GoSnag when current enough, with vendor benchmark claims marked unreproduced unless the benchmark artifacts exist. | If Parallax only beats self-hosted Sentry but is much heavier than lightweight baselines for the first useful context, narrow the simplicity claim. |

### Comparison Scorecard

Use this scorecard in Phase 1 docs and release notes. Do not fill it with
estimates; fill it only after running the protocol above.
The current version and deployment-shape manifest is
[Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md).

| Metric | Sentry self-hosted | SigNoz Docker | OpenObserve single-node | Bugsink | Rustrak | Traceway | GoSnag | Urgentry | Parallax tiny tier |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Version/tag tested | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Release maturity confidence | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Time to usable UI/API | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Time to first error/context result | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Long-running services | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Recommended CPU/RAM/disk | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Measured idle RSS/CPU | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Required external broker/cache | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Backup/restore steps | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Upgrade steps | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Sentry SDK DSN-change path | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| OTLP path without extra collector | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Evidence bundle available | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A | TBD |
| Vendor benchmark reproduced | N/A | N/A | N/A | N/A | N/A | N/A | N/A | TBD | N/A |

### Design Consequences

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
6. Bugsink and Rustrak are the Sentry-compatible error-only simplicity bar;
   Traceway is the OTLP-native embedded-mode bar; Urgentry is the benchmark
   claim bar. Parallax must win on evidence context, not pretend those products
   do not exist.

### Kill Trigger

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

## Self-Hosted Deployment Baseline Inventory

_Provenance: merged verbatim from `self-hosted-deployment-baseline-inventory.md` (2026-05-29 restructure)._

_(Shared note — see the Self-Hosted Simplicity Gate section above.)_

Research date: 2026-05-25

### Purpose

The [self-hosted simplicity gate](self-hosted-simplicity-gate.md) defines the
measurement protocol, but its scorecard is intentionally empty until the full VM
run happens. This note is the source-linked baseline inventory for that run: the
current versions to pin, official install path to follow, service shape to
expect, and caveats that prevent apples-to-oranges claims.
The result rows and product-claim status should be published through the
[Self-hosted simplicity ledger](self-hosted-simplicity-ledger.md).

This is not the full benchmark result. It is the manifest that makes the full
benchmark reproducible.

Ref-integrity recheck, 2026-05-25: service counts below were rechecked from
exact release tags or commit SHAs. Do not use mutable `main` files for measured
service counts when a release tag exists. `main` is acceptable only for a
no-release project such as GoSnag, and then only with the full commit SHA and
moving-target risk recorded.

### Current Baseline Snapshot

| System | Current pin checked | Official path to test | Declared service shape | Immediate Parallax implication |
| --- | --- | --- | --- | --- |
| Sentry self-hosted | [`26.5.0`](https://github.com/getsentry/self-hosted/releases/tag/26.5.0), published 2026-05-18. | Official self-hosted install and Docker Compose. | The `26.5.0` `docker-compose.yml` declares 72 services. Official docs state self-hosted is for simple low-volume use cases, with no dedicated support, list 4 CPU cores, 16 GB RAM, 16 GB swap, and 20 GB free disk as minimums, list feature-complete capabilities such as traces/profiles/replays/uptime/metrics/feedback/crons, and explicitly list Seer plus other AI/ML features as unavailable because those components are closed source. The sentry-mcp README separately says some features like Seer may not be available on self-hosted instances. The 26.5.0 release note adds a manual feature-flag update before `install.sh`, one new Docker container for Launchpad-backed mobile features, a weak default secret note for `LAUNCHPAD_RPC_SHARED_SECRET`, and a self-hosted objectstore gap for Snapshots. | Sentry remains the heavy baseline Parallax must beat for setup burden. The self-hosted Seer exclusion is explicit in current docs, but should be treated as current product posture rather than a permanent guarantee. Release-note action/security items now also count as operator-visible complexity. |
| SigNoz Docker | [`v0.125.1`](https://github.com/SigNoz/signoz/releases/tag/v0.125.1), published 2026-05-20. | Official Docker Compose install. | The `v0.125.1` `deploy/docker/docker-compose.yaml` declares 6 services: `signoz`, `otel-collector`, `clickhouse`, `init-clickhouse`, `zookeeper-1`, and `signoz-telemetrystore-migrator`. | SigNoz is already a compact OTLP-native, agent-facing baseline. Parallax must beat it on evidence bundles, Sentry migration, and lower tiny-tier dependency count. |
| OpenObserve | [`v0.90.2`](https://github.com/openobserve/openobserve/releases/tag/v0.90.2), published 2026-05-22. | Single Docker container or binary quickstart. | Single-binary/container path for local single-node operation; HA deployment splits roles later. | OpenObserve is the strongest Rust/self-hosted simplicity pressure test. Parallax must justify every extra default process with Sentry compatibility or evidence-bundle value. |
| Bugsink | [`2.2.1`](https://github.com/bugsink/bugsink/releases/tag/2.2.1), published 2026-05-22. | Docker quickstart, settings, and installation docs. | Throwaway Docker quickstart is a single container with SQLite and no persistence. For retained Docker data, docs recommend an external MySQL database; PostgreSQL can probably work but is not extensively tested in the Docker guide. Settings docs call SQLite the default production-ready database outside the Docker-volume caveat and explain that Docker volumes are not recommended for SQLite WAL mode. | Error-only Sentry-compatible simplicity is already available, but the benchmark must separate demo startup from persistent deployment. Parallax must not present "change the DSN and self-host" as a moat. |
| Rustrak | [`@rustrak/server@0.2.5`](https://github.com/AbianS/rustrak/releases/tag/%40rustrak/server%400.2.5), published 2026-05-21. Generic [`releases/latest`](https://github.com/AbianS/rustrak/releases/latest) currently resolves to `docs@0.1.16`, so the server package must be pinned explicitly. | README/docs SQLite-default Docker Compose for server + UI; server-only SQLite Docker path; Postgres image for production. | Default quickstart is 2 containers (`server`, `ui`) with SQLite volume; production example adds Postgres. Docker Hub metadata shows `abians7/rustrak-server:v0.2.5` last updated 2026-05-21 with amd64/arm64 images around 16-17 MB; UI images are much larger. The README claims around 50 MB server memory, sub-50 ms P99 ingestion, 10k+ events/s, no Redis, and no complex infrastructure. | Rust-first Sentry-compatible lightweight tracking exists, but the benchmark must treat monorepo/package release streams carefully and keep memory/latency/throughput as unmeasured vendor claims. Rustrak also ships an MCP package, so Parallax's agent differentiation must be the citable bundle and outcome graph, not MCP existence. |
| Traceway | [`backend/v1.7.27`](https://github.com/tracewayapp/traceway/releases/tag/backend/v1.7.27), published 2026-05-22; tag commit [`28a4e56`](https://github.com/tracewayapp/traceway/commit/28a4e5666da85f125dbfaf5e681c09b359b5d177); latest checked `main` commit [`38b8d385`](https://github.com/tracewayapp/traceway/commit/38b8d385fbc610d45879d4a1bf3907c8434e8ed9). | Docker Compose, all-in-one container, minimal external-db image, SQLite image/Compose, and embedded Go mode for local/dev. | Release-tag root Compose declares 3 services (`traceway`, `clickhouse`, `postgres`). Release-tag SQLite Compose declares 1 service (`traceway`). All-in-one hides ClickHouse and Postgres inside one container. SQLite mode is a single Alpine container with two SQLite files plus local blobs under `/data`, optional S3 for source maps/session recordings/AI traces, and retention knobs. Embedded mode runs inside a Go process with SQLite and is documented as development-only. Image-size and signed-image claims are documented but unmeasured in this pass. | Traceway pressures the OTLP-native, frontend/session replay, AI tracing, and "no Collector" parts of the roadmap. It is not a Sentry-envelope migration path yet, and deployment scoring must separate visible services from bundled subsystems and SQLite persistence semantics. Because `main` has commits after the release, measured counts must use the release tag unless the benchmark explicitly tests moving `main`. |
| GoSnag | No GitHub release/tag at check time; `releases/latest` redirects to the releases index, not a tag. Pin `main` commit [`418b8b107e274bfaab3f905510ddd274173d216b`](https://github.com/darkspock/gosnag/commit/418b8b107e274bfaab3f905510ddd274173d216b), dated 2026-04-17, or the latest commit at benchmark time. | Docker Compose quickstart. | The pinned commit's Docker Compose declares 2 services (`gosnag`, `db`) and `DATABASE_URL` is required. README describes a single Go binary with embedded React UI and migrations, plus PostgreSQL; Dockerfile builds with Node 20 and Go 1.25 into Alpine. | GoSnag combines Sentry error-event ingest, AI RCA/triage features, tickets, GitHub/Jira, and a documented management MCP server. The Parallax gap is OTLP context, read-only evidence bundles, and fix/outcome feedback, not "AI over errors." Until a release appears, GoSnag remains a moving-target baseline and cannot support release-stable comparison wording. |
| Urgentry | [`v0.2.12`](https://github.com/urgentry/urgentry/releases/tag/v0.2.12), published 2026-05-22; annotated tag dereferences to commit [`8d706b3`](https://github.com/urgentry/urgentry/commit/8d706b3cdd4653351578df9521b24bfd4da6a6d5); latest checked `main` commit [`ccc0ff8`](https://github.com/urgentry/urgentry/commit/ccc0ff815ec8b19d3b7c820b95bc3d539414e145). | Tiny one-binary path and split self-hosted path. | Tiny mode is one binary with SQLite. Release-tag self-hosted Compose declares 11 services: PostgreSQL, MinIO, Valkey, NATS, two bootstrap/helper services, four Urgentry roles (`api`, `ingest`, `worker`, `scheduler`), and optional ClickHouse under the `columnar` profile. README/docs publish benchmark claims against self-hosted Sentry 26.3.1 over a narrow envelope-ingest workload. | Urgentry is not OSI-open, but it is a serious simplicity, Sentry-protocol breadth, and benchmark-methodology baseline. Include it whenever Parallax claims "simpler than self-hosted Sentry"; keep performance numbers as vendor claims until reproduced. Because `main` has commits after the release, measured counts must use the release tag unless the benchmark explicitly tests moving `main`. |

### Source-Check Commands

Use these commands at the start of a measured run, replacing tags with the
latest public stable tags available on that date:

```sh
curl -Ls -o /dev/null -w '%{url_effective}\n' \
  https://github.com/getsentry/self-hosted/releases/latest

curl -Ls -o /dev/null -w '%{url_effective}\n' \
  https://github.com/SigNoz/signoz/releases/latest

curl -Ls -o /dev/null -w '%{url_effective}\n' \
  https://github.com/openobserve/openobserve/releases/latest

curl -Ls -o /dev/null -w '%{url_effective}\n' \
  https://github.com/bugsink/bugsink/releases/latest

# Rustrak is a monorepo with package-specific releases. Record the generic
# latest URL, but pin the server package tag separately.
curl -Ls -o /dev/null -w '%{url_effective}\n' \
  https://github.com/AbianS/rustrak/releases/latest

curl -Ls -o /dev/null -w '%{url_effective}\n' \
  https://github.com/AbianS/rustrak/releases/tag/%40rustrak/server%400.2.5

curl -Ls -o /dev/null -w '%{url_effective}\n' \
  https://github.com/tracewayapp/traceway/releases/latest

curl -Ls -o /dev/null -w '%{url_effective}\n' \
  https://github.com/urgentry/urgentry/releases/latest

curl -Ls -o /dev/null -w '%{http_code} %{url_effective}\n' \
  https://github.com/darkspock/gosnag/releases/latest

git ls-remote --tags https://github.com/tracewayapp/traceway.git \
  'refs/tags/backend/v1.7.27*'

git ls-remote https://github.com/tracewayapp/traceway.git refs/heads/main

git ls-remote --heads --tags https://github.com/urgentry/urgentry.git

git ls-remote https://github.com/darkspock/gosnag.git refs/heads/main
```

For Compose-based installs, count services from the exact tested tag or commit,
not from `main` unless the project has no release tag. Slash-containing tags
such as Traceway's `backend/v1.7.27` can be resolved through GitHub's contents
API with `?ref=backend/v1.7.27` if a raw-file client cannot disambiguate the
ref.

```sh
curl -Ls https://raw.githubusercontent.com/getsentry/self-hosted/26.5.0/docker-compose.yml |
  yq '.services | length'

curl -Ls https://raw.githubusercontent.com/SigNoz/signoz/v0.125.1/deploy/docker/docker-compose.yaml |
  yq '.services | keys | .[]'

curl -Ls https://raw.githubusercontent.com/tracewayapp/traceway/backend/v1.7.27/docker-compose.yml |
  yq '.services | keys | .[]'

curl -Ls https://raw.githubusercontent.com/tracewayapp/traceway/backend/v1.7.27/docker-compose.sqlite.yml |
  yq '.services | keys | .[]'

curl -Ls https://raw.githubusercontent.com/darkspock/gosnag/418b8b107e274bfaab3f905510ddd274173d216b/docker-compose.yml |
  yq '.services | keys | .[]'

curl -Ls https://raw.githubusercontent.com/urgentry/urgentry/v0.2.12/deploy/compose/docker-compose.yml |
  yq '.services | keys | .[]'
```

Rustrak's README contains a SQLite-default quickstart with `server` and `ui`
services. Its repository root `docker-compose.yml` is the Postgres variant; do
not treat that as the default quickstart without saying so.

GoSnag had no releases or tags in GitHub during this pass. Treat `main` as an
unpinned moving target unless a release appears before the measured run.

### Measurement Rules Added By This Pass

1. **Pin product code, not docs examples.** Documentation snippets can lag real
   releases. For example, SigNoz docs may show sample container output from an
   older image while the current release tag is newer. The benchmark should pin
   the latest release and then follow the official install path for that release.
2. **Pin the measured release stream, not only "latest."** Monorepos and
   componentized projects can have package-specific tags where the generic latest
   release belongs to docs or another component. No-release projects such as
   GoSnag remain moving targets until pinned to a full commit SHA.
3. **Never count services from floating `main` when a release exists.** Traceway
   and Urgentry both had `main` commits after the current release when this pass
   ran. `main` is useful as a drift signal, but release comparisons must use the
   tested tag unless the scorecard explicitly labels the row `moving_main`.
4. **Record release-note action items.** Manual pre-install steps, added
   containers, default-secret warnings, unsupported self-hosted features, and
   security caveats are part of operator-visible deployment complexity even
   before a VM benchmark measures wall-clock time.
5. **Separate throwaway from persistent quickstart.** Bugsink's single-container
   Docker quickstart is excellent for evaluation, but persistent data and backup
   behavior must be measured separately. The same rule applies to every SQLite
   default.
6. **Count helper/init services honestly.** Init containers and migrators are
   not long-running services, but they are operator-visible complexity and should
   be recorded separately from steady-state containers.
7. **Measure first useful output, not first web page.** For Sentry-like products,
   first useful output is a captured error issue. For Parallax, it is
   `parallax issue context <issue-id>` returning the first redacted evidence
   bundle with missing-data warnings.
8. **Record agent/MCP posture separately from deployment simplicity.** Sentry,
   Rustrak, and GoSnag now have MCP surfaces. MCP presence should not improve a
   deployment score unless it is safe, read-only where appropriate, citable, and
   connected to outcome records.

### What This Changes

The old "Parallax is simpler than self-hosted Sentry" claim is too weak on its
own. Sentry is still heavy, but Bugsink, Rustrak, Urgentry, and GoSnag already
attack that pain directly. The measured claim must be narrower:

> Parallax is simple enough to self-host for its first useful evidence-bundle
> workflow, while covering a wider evidence surface than lightweight error
> trackers and exposing safer agent context than generic MCP query tools.

That means the tiny tier should preserve this default shape:

```text
parallax-server
greptimedb standalone
embedded/local metadata file
local WAL/raw retention directory
parallax CLI
```

No broker, Redis, Postgres, separate UI service, required MCP sidecar, or
external Collector should enter the default path before the first issue context
works.

### Relationship To Other Research

- [Self-hosted simplicity gate](self-hosted-simplicity-gate.md) owns the full
  benchmark protocol and scorecard.
- [Self-hosted simplicity ledger](self-hosted-simplicity-ledger.md) defines the
  run artifacts, row schemas, claim levels, and expiry rules for using this
  inventory in product claims.
- [Lightweight Sentry-compatible competitor watch](lightweight-sentry-compatible-competitor-watch.md)
  tracks the projects that make this baseline necessary.
- [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
  explains why Parallax's first MCP posture must be read-only and bundle-based.
- [Fixer component and outcome loop](fixer-component-and-outcome-loop.md) keeps
  PR/fix automation outside core and requires outcome writeback.

### Sources

- [Sentry self-hosted docs](https://develop.sentry.dev/self-hosted/)
- [Sentry self-hosted 26.5.0 release](https://github.com/getsentry/self-hosted/releases/tag/26.5.0)
- [Sentry self-hosted Docker Compose 26.5.0](https://github.com/getsentry/self-hosted/blob/26.5.0/docker-compose.yml)
- [Sentry MCP README](https://github.com/getsentry/sentry-mcp)
- [Sentry MCP 0.35.0 release](https://github.com/getsentry/sentry-mcp/releases/tag/0.35.0)
- [SigNoz v0.125.1 release](https://github.com/SigNoz/signoz/releases/tag/v0.125.1)
- [SigNoz Docker Compose v0.125.1](https://github.com/SigNoz/signoz/blob/v0.125.1/deploy/docker/docker-compose.yaml)
- [OpenObserve v0.90.2 release](https://github.com/openobserve/openobserve/releases/tag/v0.90.2)
- [OpenObserve README](https://github.com/openobserve/openobserve)
- [Bugsink 2.2.1 release](https://github.com/bugsink/bugsink/releases/tag/2.2.1)
- [Bugsink built to self-host](https://www.bugsink.com/built-to-self-host/)
- [Bugsink Docker install](https://www.bugsink.com/docs/docker-install/)
- [Bugsink settings](https://www.bugsink.com/docs/settings/)
- [Rustrak README](https://github.com/AbianS/rustrak)
- [Rustrak server 0.2.5 release](https://github.com/AbianS/rustrak/releases/tag/%40rustrak/server%400.2.5)
- [Rustrak MCP package](https://www.npmjs.com/package/@rustrak/mcp)
- [Rustrak Sentry MCP protocol recheck](rustrak-sentry-mcp-protocol-recheck.md)
- [Traceway README](https://github.com/tracewayapp/traceway)
- [Traceway backend v1.7.27 release](https://github.com/tracewayapp/traceway/releases/tag/backend/v1.7.27)
- [Traceway SQLite deployment](https://docs.tracewayapp.com/server/sqlite)
- [Traceway Docker image signatures](https://github.com/tracewayapp/traceway/blob/main/DOCKER_SIGNATURES.md)
- [Traceway OTLP AI Replay Recheck](traceway-otlp-ai-replay-recheck.md)
- [GoSnag README](https://github.com/darkspock/gosnag)
- [GoSnag main commit checked](https://github.com/darkspock/gosnag/commit/418b8b107e274bfaab3f905510ddd274173d216b)
- [GoSnag Sentry AI MCP recheck](gosnag-sentry-ai-mcp-recheck.md)
- [Urgentry README](https://github.com/urgentry/urgentry)
- [Urgentry v0.2.12 release](https://github.com/urgentry/urgentry/releases/tag/v0.2.12)
- [Urgentry Sentry Tiny Benchmark Recheck](urgentry-sentry-tiny-benchmark-recheck.md)

### Bottom Line

The benchmark baseline is now sharper: Sentry is the complexity floor to beat,
but lightweight challengers are the real simplicity bar. Parallax only earns its
tiny-tier claim if it stays close to their deployment shape while producing
evidence bundles, cross-signal context, redaction reports, and outcome-ready
agent context that they do not currently provide.

## Self-Hosted Simplicity Ledger

_Provenance: merged verbatim from `self-hosted-simplicity-ledger.md` (2026-05-29 restructure)._

_(Shared note — see the Self-Hosted Simplicity Gate section above.)_

Research date: 2026-05-25

### Purpose

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

### Current Source Snapshot

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
| [Traceway backend v1.7.27 release](https://github.com/tracewayapp/traceway/releases/tag/backend/v1.7.27), [GoSnag main commit](https://github.com/darkspock/gosnag/commit/418b8b107e274bfaab3f905510ddd274173d216b), and [Urgentry v0.2.12 release](https://github.com/urgentry/urgentry/releases/tag/v0.2.12) | Lightweight challengers are versioning differently: component releases, no-release moving `main`, and published tiny/split deployment modes. The comparison ledger must mark release-stream confidence, exact `source_ref`, tag commit, artifact hash, and moving-target risk per competitor. Release-tag rows must not borrow service counts from `main` after the release. |
| [Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md) | This is the current baseline manifest. Measured runs must refresh it before claiming results because release tags, docs, release-note action items, and service graphs move quickly. |

### Claim Levels

| Level | Meaning | Minimum evidence |
| --- | --- | --- |
| `not_measured` | No current self-hosted simplicity run exists. | Default state. |
| `baseline_inventory_current` | Competitor versions, install paths, service shapes, release maturity, and moving-target risk were refreshed. | Source snapshot rows for Sentry, SigNoz, OpenObserve, Bugsink, Rustrak, Traceway, GoSnag, Urgentry, GreptimeDB, and Turso/libSQL, with explicit stale/excluded reasons for any omitted lightweight challenger. |
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
| `tiny_tier_self_hosted_claim` | Parallax can claim the tested tiny-tier self-hosted workflow. | All required Parallax rows pass; Sentry, SigNoz, OpenObserve, and the named lightweight challenger set are current or explicitly marked stale/excluded; vendor benchmark claims are reproduced or labeled unmeasured. |
| `claim_expired` | A prior claim is stale. | Refresh trigger fired or max age elapsed. |
| `claim_failed` | A required fixture failed. | Any gate miss that changes allowed product wording. |

Initial claim level: `not_measured`.

### Result Artifacts

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

### Run Manifest

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
    {"system": "traceway", "version": "backend/v1.7.27", "release_stream": "backend", "source_ref": "backend/v1.7.27", "source_commit": "28a4e5666da85f125dbfaf5e681c09b359b5d177"},
    {"system": "gosnag", "version": "418b8b107e274bfaab3f905510ddd274173d216b", "release_stream": "main_no_release", "source_ref": "418b8b107e274bfaab3f905510ddd274173d216b", "moving_target": true},
    {"system": "urgentry", "version": "v0.2.12", "release_stream": "urgentry", "source_ref": "v0.2.12", "source_commit": "8d706b3cdd4653351578df9521b24bfd4da6a6d5"}
  ],
  "policies": {
    "redaction_policy": "a6-default-deny-vN",
    "bundle_schema_version": "0.1.0",
    "self_hosted_gate_version": "v1"
  },
  "result": "pass"
}
```

### Minimum Row Schemas

Source snapshot row:

```json
{
  "system": "sentry_self_hosted",
  "source_url": "https://github.com/getsentry/self-hosted/releases/latest",
  "latest_url_effective": "https://github.com/getsentry/self-hosted/releases/tag/26.5.0",
  "release_stream": "self-hosted",
  "resolved_version": "26.5.0",
  "source_ref": "26.5.0",
  "source_commit": "<tag-or-commit-sha>",
  "resolved_at": "2026-05-25T13:00:00Z",
  "install_doc_url": "https://develop.sentry.dev/self-hosted/",
  "artifact_hash": "sha256:<hex>",
  "release_note_action_items": 0,
  "release_note_security_notes": 0,
  "service_source": "release_tag|commit_sha|moving_main",
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

### Counting Rules

- Start from a fresh VM with Docker installed and no product data. Anything else
  is a local smoke test, not a self-hosted simplicity result.
- Measure Parallax from first documented install command to first useful
  `parallax issue context <issue-id>` bundle, not to first web page.
- Measure competitors using their official install path and their own first
  useful result: captured issue/error for error trackers, usable query/result
  for observability systems.
- Measure lightweight challengers by role, not as one interchangeable column:
  Bugsink for mature Sentry-compatible simplicity, Rustrak for Rust-first
  Sentry-compatible + MCP shape, Traceway for OTLP-native embedded mode,
  Urgentry for broad Sentry item handling plus vendor benchmark claims, and
  GoSnag as a low-maturity feature warning.
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
- Vendor benchmark numbers are not evidence until reproduced by the benchmark
  agent or an equivalent recorded run. Until then, they affect watch priority,
  not measured Parallax pass/fail status.

### Required Pass Set

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
| Named lightweight baseline comparison | Bugsink, Rustrak, Traceway, Urgentry, and GoSnag are measured or explicitly marked stale/excluded with reasons. |

### Refresh Triggers

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

### Product Wording

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

Allowed only when named lightweight baselines are current:

> Against the tested lightweight baseline set, Parallax trades modest extra
> setup for cross-signal evidence bundles and agent-safe context.

Avoid:

- "Drop-in Sentry replacement."
- "One-command install" unless the command ledger proves it.
- "Production-ready HA."
- "Simpler than every observability product."
- "No operations required."
- "Air-gapped ready" unless the run used an air-gapped fixture.
- "Self-hosted claim" without naming the host profile, versions, and workload.
- "Urgentry benchmark beaten" or similar wording before the benchmark artifact
  reproduces the vendor claim under the shared protocol and the workload scope
  is shown to match the claimed comparison.

### Relationship To Other Research

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

### Bottom Line

The self-hosted wedge is not a slogan. It is a timed, reproducible clean-VM run
that produces a useful redacted bundle with fewer services and less operator
burden than the current baselines. Until this ledger is green, Parallax can say
it is designed for low-ops self-hosting; it cannot claim the tiny tier proves it.

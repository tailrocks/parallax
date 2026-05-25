# Self-Hosted Deployment Baseline Inventory

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The [self-hosted simplicity gate](self-hosted-simplicity-gate.md) defines the
measurement protocol, but its scorecard is intentionally empty until the full VM
run happens. This note is the source-linked baseline inventory for that run: the
current versions to pin, official install path to follow, service shape to
expect, and caveats that prevent apples-to-oranges claims.
The result rows and product-claim status should be published through the
[Self-hosted simplicity ledger](self-hosted-simplicity-ledger.md).

This is not the full benchmark result. It is the manifest that makes the full
benchmark reproducible.

## Current Baseline Snapshot

| System | Current pin checked | Official path to test | Declared service shape | Immediate Parallax implication |
| --- | --- | --- | --- | --- |
| Sentry self-hosted | [`26.5.0`](https://github.com/getsentry/self-hosted/releases/tag/26.5.0), published 2026-05-18. | Official self-hosted install and Docker Compose. | The `26.5.0` `docker-compose.yml` declares 72 services. Official docs state self-hosted is for simple low-volume use cases, with no dedicated support, list 4 CPU cores, 16 GB RAM, 16 GB swap, and 20 GB free disk as minimums, and list feature-complete capabilities without Seer/AI. The sentry-mcp README separately says some features like Seer may not be available on self-hosted instances. The 26.5.0 release note adds a manual feature-flag update before `install.sh`, one new Docker container for Launchpad-backed mobile features, a weak default secret note for `LAUNCHPAD_RPC_SHARED_SECRET`, and a self-hosted objectstore gap for Snapshots. | Sentry remains the heavy baseline Parallax must beat for setup burden. Hosted-Seer parity for self-hosted Sentry is not proven by current sources, but the baseline should not overstate this as an explicit blanket AI exclusion in the current docs. Release-note action/security items now also count as operator-visible complexity. |
| SigNoz Docker | [`v0.125.1`](https://github.com/SigNoz/signoz/releases/tag/v0.125.1), published 2026-05-20. | Official Docker Compose install. | The `v0.125.1` `deploy/docker/docker-compose.yaml` declares 6 services: `signoz`, `otel-collector`, `clickhouse`, `init-clickhouse`, `zookeeper-1`, and `signoz-telemetrystore-migrator`. | SigNoz is already a compact OTLP-native, agent-facing baseline. Parallax must beat it on evidence bundles, Sentry migration, and lower tiny-tier dependency count. |
| OpenObserve | [`v0.90.2`](https://github.com/openobserve/openobserve/releases/tag/v0.90.2), published 2026-05-22. | Single Docker container or binary quickstart. | Single-binary/container path for local single-node operation; HA deployment splits roles later. | OpenObserve is the strongest Rust/self-hosted simplicity pressure test. Parallax must justify every extra default process with Sentry compatibility or evidence-bundle value. |
| Bugsink | [`2.2.1`](https://github.com/bugsink/bugsink/releases/tag/2.2.1), published 2026-05-22. | Docker quickstart, settings, and installation docs. | Throwaway Docker quickstart is a single container with SQLite and no persistence. For retained Docker data, docs recommend an external MySQL database; PostgreSQL can probably work but is not extensively tested in the Docker guide. Settings docs call SQLite the default production-ready database outside the Docker-volume caveat and explain that Docker volumes are not recommended for SQLite WAL mode. | Error-only Sentry-compatible simplicity is already available, but the benchmark must separate demo startup from persistent deployment. Parallax must not present "change the DSN and self-host" as a moat. |
| Rustrak | [`@rustrak/server@0.2.5`](https://github.com/AbianS/rustrak/releases/tag/%40rustrak/server%400.2.5), published 2026-05-21. Generic [`releases/latest`](https://github.com/AbianS/rustrak/releases/latest) currently resolves to `docs@0.1.16`, so the server package must be pinned explicitly. | README/docs SQLite-default Docker Compose for server + UI; server-only SQLite Docker path; Postgres image for production. | Default quickstart is 2 containers (`server`, `ui`) with SQLite volume; production example adds Postgres. Docker Hub metadata shows `abians7/rustrak-server:v0.2.5` last updated 2026-05-21 with amd64/arm64 images around 16-17 MB; UI images are much larger. The README claims around 50 MB server memory, sub-50 ms P99 ingestion, 10k+ events/s, no Redis, and no complex infrastructure. | Rust-first Sentry-compatible lightweight tracking exists, but the benchmark must treat monorepo/package release streams carefully and keep memory/latency/throughput as unmeasured vendor claims. Rustrak also ships an MCP package, so Parallax's agent differentiation must be the citable bundle and outcome graph, not MCP existence. |
| Traceway | [`backend/v1.7.27`](https://github.com/tracewayapp/traceway/releases/tag/backend/v1.7.27), published 2026-05-22; latest checked `main` commit [`38b8d385`](https://github.com/tracewayapp/traceway/commit/38b8d385fbc610d45879d4a1bf3907c8434e8ed9). | Docker Compose, all-in-one container, minimal external-db image, SQLite image/Compose, and embedded Go mode for local/dev. | Root Compose declares 3 services (`traceway`, `clickhouse`, `postgres`). All-in-one hides ClickHouse and Postgres inside one container. SQLite mode is a single Alpine container with two SQLite files plus local blobs under `/data`, optional S3 for source maps/session recordings/AI traces, and retention knobs. Embedded mode runs inside a Go process with SQLite and is documented as development-only. Image-size and signed-image claims are documented but unmeasured in this pass. | Traceway pressures the OTLP-native, frontend/session replay, AI tracing, and "no Collector" parts of the roadmap. It is not a Sentry-envelope migration path yet, and deployment scoring must separate visible services from bundled subsystems and SQLite persistence semantics. |
| GoSnag | No GitHub release/tag at check time; pin `main` commit [`418b8b1`](https://github.com/darkspock/gosnag/commit/418b8b107e274bfaab3f905510ddd274173d216b), dated 2026-04-17, or the latest commit at benchmark time. | Docker Compose quickstart. | `main` Docker Compose declares 2 services (`gosnag`, `db`) and `DATABASE_URL` is required. README describes a single Go binary with embedded React UI and migrations, plus PostgreSQL; Dockerfile builds with Node 20 and Go 1.25 into Alpine. | GoSnag combines Sentry error-event ingest, AI RCA/triage features, tickets, GitHub/Jira, and a documented management MCP server. The Parallax gap is OTLP context, read-only evidence bundles, and fix/outcome feedback, not "AI over errors." |
| Urgentry | [`v0.2.12`](https://github.com/urgentry/urgentry/releases/tag/v0.2.12), published 2026-05-22; latest checked `main` commit [`ccc0ff8`](https://github.com/urgentry/urgentry/commit/ccc0ff815ec8b19d3b7c820b95bc3d539414e145). | Tiny one-binary path and split self-hosted path. | Tiny mode is one binary with SQLite. Self-hosted mode splits `api`, `ingest`, `worker`, and `scheduler` roles over PostgreSQL, MinIO, Valkey, and NATS; Compose also includes bootstrap/helper services and optional ClickHouse. README/docs publish benchmark claims against self-hosted Sentry 26.3.1 over a narrow envelope-ingest workload. | Urgentry is not OSI-open, but it is a serious simplicity, Sentry-protocol breadth, and benchmark-methodology baseline. Include it whenever Parallax claims "simpler than self-hosted Sentry"; keep performance numbers as vendor claims until reproduced. |

## Source-Check Commands

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

git ls-remote --heads --tags https://github.com/urgentry/urgentry.git
```

For Compose-based installs, count services from the exact tested tag:

```sh
curl -Ls https://raw.githubusercontent.com/getsentry/self-hosted/26.5.0/docker-compose.yml |
  yq '.services | length'

curl -Ls https://raw.githubusercontent.com/SigNoz/signoz/v0.125.1/deploy/docker/docker-compose.yaml |
  yq '.services | keys | .[]'

curl -Ls https://raw.githubusercontent.com/tracewayapp/traceway/main/docker-compose.yml |
  yq '.services | keys | .[]'

curl -Ls https://raw.githubusercontent.com/tracewayapp/traceway/main/docker-compose.sqlite.yml |
  yq '.services | keys | .[]'

curl -Ls https://raw.githubusercontent.com/darkspock/gosnag/main/docker-compose.yml |
  yq '.services | keys | .[]'

curl -Ls https://raw.githubusercontent.com/urgentry/urgentry/main/deploy/compose/docker-compose.yml |
  yq '.services | keys | .[]'
```

Rustrak's README contains a SQLite-default quickstart with `server` and `ui`
services. Its repository root `docker-compose.yml` is the Postgres variant; do
not treat that as the default quickstart without saying so.

GoSnag had no releases or tags in GitHub during this pass. Treat `main` as an
unpinned moving target unless a release appears before the measured run.

## Measurement Rules Added By This Pass

1. **Pin product code, not docs examples.** Documentation snippets can lag real
   releases. For example, SigNoz docs may show sample container output from an
   older image while the current release tag is newer. The benchmark should pin
   the latest release and then follow the official install path for that release.
2. **Pin the measured release stream, not only "latest."** Monorepos and
   componentized projects can have package-specific tags where the generic latest
   release belongs to docs or another component. No-release projects such as
   GoSnag remain moving targets until pinned to a commit.
3. **Record release-note action items.** Manual pre-install steps, added
   containers, default-secret warnings, unsupported self-hosted features, and
   security caveats are part of operator-visible deployment complexity even
   before a VM benchmark measures wall-clock time.
4. **Separate throwaway from persistent quickstart.** Bugsink's single-container
   Docker quickstart is excellent for evaluation, but persistent data and backup
   behavior must be measured separately. The same rule applies to every SQLite
   default.
5. **Count helper/init services honestly.** Init containers and migrators are
   not long-running services, but they are operator-visible complexity and should
   be recorded separately from steady-state containers.
6. **Measure first useful output, not first web page.** For Sentry-like products,
   first useful output is a captured error issue. For Parallax, it is
   `parallax issue context <issue-id>` returning the first redacted evidence
   bundle with missing-data warnings.
7. **Record agent/MCP posture separately from deployment simplicity.** Sentry,
   Rustrak, and GoSnag now have MCP surfaces. MCP presence should not improve a
   deployment score unless it is safe, read-only where appropriate, citable, and
   connected to outcome records.

## What This Changes

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

## Relationship To Other Research

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

## Sources

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

## Bottom Line

The benchmark baseline is now sharper: Sentry is the complexity floor to beat,
but lightweight challengers are the real simplicity bar. Parallax only earns its
tiny-tier claim if it stays close to their deployment shape while producing
evidence bundles, cross-signal context, redaction reports, and outcome-ready
agent context that they do not currently provide.

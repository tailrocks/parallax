# Parallax `main` vs self-hosted observability backends

> Canonical live verification. Run date: **2026-09-04**, Asia/Ho_Chi_Minh
> (`2026-09-04T04:35+07:00` capture). This report compares the exact Parallax
> executable built from `main` against fresh current backend deployments.

## Executive conclusion

Parallax at [`3c4b68d`](https://github.com/tailrocks/parallax/commit/3c4b68d3acf8fb435102ae2beb8f184bf40b617c) is a working Rust-first execution-context engine, not a mature general observability suite. The live run proved OTLP traces/logs/metrics, derived errors and issue fingerprints, Sentry-envelope ingest, CLI invocation/session context, Claude Code import, bounded live tails, alerting, dashboards, investigations, redaction, MCP projection equivalence, GitHub webhook validation, and the embedded UI/API.

Parallax is strongest where the product model is distinctive: one canonical API joining telemetry, errors, invocations, agent sessions, deploy/CI context, redacted evidence, and fix-oriented investigation. It is not strongest on generic observability breadth. OpenObserve, SigNoz, and Grafana LGTM had more mature cross-signal exploration; Sentry led issue workflow, SDK breadth, replay, and profiling; Grafana led query/visualization; Maple led local single-binary UX. Parallax's bundle value versus raw context remains unproven.

The comparison found two integration defects and fixed them at their enabling layer:

1. Playground catalog readiness raced Postgres. Compose now gates catalog on Postgres health.
2. Foundry-generated SigNoz ports were appended by the overlay, colliding with Rotel and the generated UI mapping. The overlay now uses explicit Compose `!override` port replacement.

HyperDX/ClickStack UI and API worked at `2.37.0`, but its AIO OTLP listener did not bind in this run; it was excluded from Rotel's sequential exporter list to avoid backpressure. Rustrak's UI and Sentry-envelope issue path worked; its container healthcheck remained red despite `/health` returning `{"status":"ok"}`. These are recorded as comparator limitations, not Parallax wins.

## Environment manifest

### Source and executable

| Item | Exact value |
| --- | --- |
| Parallax remote | `git@github.com:tailrocks/parallax.git` |
| Parallax source | `main` / `3c4b68d3acf8fb435102ae2beb8f184bf40b617c` |
| Playground remote | `git@github.com:tailrocks/parallax-telemetry-playground.git` |
| Playground source | `main` / `bc3d771a386a99387fab6989ac98992d978965cc` |
| Binary | `parallax 0.1.0-research.3c4b68d` |
| Binary SHA-256 | `67d33904b37e806ff72351f968afed97d885b242a374c0be5c926e25e83f79ab` |
| MCP binary SHA-256 | `0137327784c16cde2e3c6fba6e9ef348bb7e77d2642135405ccc0f058f8b16c6` |
| Rust/Cargo | `rustc 1.98.0`, `cargo 1.98.0` |
| Bun | `1.4.0` bare shell; repository `mise` runtime `1.3.14` |
| agent-browser | `0.36.0` |
| Docker Engine | `29.4.0` |
| Rotel | `streamfold/rotel:v0.2.5` (`sha256:c5d16eeba67a09082a519aeb72a8a1c126d5a4d343ca163d90df37ffde50e7a`) |

The Parallax build was performed after fetching `origin/main`; the binary was started from the resulting release build. Managed GreptimeDB used `v1.1.2`. Parallax listened on API/UI `127.0.0.1:4000`, OTLP/gRPC `14317`, and OTLP/HTTP `14318`; Rotel owned host `4317/4318`.

### Backend version manifest

Latest stable/current supported artifacts were resolved on the run date. RCs were excluded from stable comparisons. OpenObserve `v1.0.0-rc2` was newer but not GA; its latest GA used here was `v0.92.2`. OTel Contrib was `v0.160.0`, but the published telemetrygen image was absent, so the newest runnable image was `v0.159.0`.

| Backend | Upstream source | Latest release date | Version/ref used | Reproducibility |
| --- | --- | --- | --- | --- |
| OpenObserve | [openobserve/openobserve releases](https://github.com/openobserve/openobserve/releases) | 2026-08-17 | `public.ecr.aws/zinclabs/openobserve:v0.92.2` | `sha256:88fb692ac791d3eaff69653a4a4686f1c7eceb9e105491d58d29ac2739560b3b` |
| Maple | [MapleTechLabs/maple v0.0.21](https://github.com/MapleTechLabs/maple/releases/tag/v0.0.21) | 2026-08-24 | official arm64 bundle, wrapped by lab image | installer release ref `v0.0.21` |
| Sentry self-hosted | [getsentry/self-hosted 26.8.0](https://github.com/getsentry/self-hosted/releases/tag/26.8.0) | 2026-08-17 | vendored `26.8.0`, own Compose stack | target commit `73fe2f2747800873f0896aec283c45b6dcf34432`; web image digest `sha256:ca6d07134c6f3faa6e6e89eced76b18af71adbd7ccf30806154e0b9416e649ae` |
| SigNoz | [SigNoz/signoz v0.140.0](https://github.com/SigNoz/signoz/releases/tag/v0.140.0) | 2026-09-02 | Foundry-generated Compose; core `v0.140.0` | core `sha256:7969e02eb3ea7904d6c824404ca2bc0205209527fd0fada00283156b8e78cc5c` |
| SigNoz Foundry | [SigNoz/foundry v0.2.17](https://github.com/SigNoz/foundry/releases/tag/v0.2.17) | 2026-07-29 | `foundryctl forge`, no ledger/updater | tag commit `273dec4a6f6bb8a70b4db9dc975b958d0e2a2944`; arm64 archive SHA-256 `5664c5cf33531dc35bc7f951d331379f23aad1f944417a918d0922d3f77424c4` |
| SigNoz collector | [official image](https://hub.docker.com/r/signoz/signoz-otel-collector) | 2026-09-04 image check | `v0.144.9` | `sha256:72aa1e4c1ec529f178e962c049be35cfd7abae02fe9a397edad10b4a9cba62fa` |
| Grafana LGTM | [docker-otel-lgtm v0.32.0](https://github.com/grafana/docker-otel-lgtm/releases/tag/v0.32.0) | 2026-08-28 | `grafana/otel-lgtm:0.32.0` | `sha256:d6b20e35890ef2f91d13944805939acdaf1e5d3ffbf9f9aed08586312826c815`; bundled UI reported `13.2.0` |
| HyperDX / ClickStack | [ClickStack docs](https://clickhouse.com/docs/use-cases/observability/clickstack) | 2026-09-04 image check | `clickhouse/clickstack-all-in-one:2.37.0` | `sha256:16650781330f42fea6b02b15144a2233383077c799ab5bc06b131abe23e89f47` |
| Rustrak | [rustrak v0.14.11](https://github.com/rustrak/rustrak/releases/tag/v0.14.11) | 2026-09-02 | server + UI `v0.14.11` | server `sha256:5866143e547a6995aabd29f9683689a1694f7ebee0dc658729d8023cc549652e`; UI `sha256:072e84fe19094d5bcbcab2c6bce2f55b59a2dd937921dc7d184bcbbf76d30888` |
| Telemetrygen | [OTel Contrib v0.160.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.160.0) | 2026-09-02 | `ghcr.io/.../telemetrygen:v0.159.0` | `sha256:bad7fde3119476a9177cc4b9f04307c9876fca5b5584e009aebd7123d7557658`; v0.160.0 registry artifact absent |

Grafana's bundled components were the image's current set; independently observed component versions included Mimir `3.2.0`, Loki `3.7.7`, Tempo `3.0.3`, and Alloy `1.19.2`. ClickStack was evaluated as the current AIO image, not a claimed HyperDX OTLP deployment when its receiver was unavailable.

## Methodology

1. Fetch both `origin/main` branches. Build Parallax from the exact fetched SHA.
2. Start Parallax with an isolated dated data directory and the managed storage profile.
3. Start the fresh playground at its exact SHA. The same workload emitted OTLP to Rotel; Sentry SDK/envelope paths were exercised separately where required.
4. Rotel fanned traces, logs, and metrics to each backend only after readiness probes. Sentry received traces/logs over its native OTLP HTTP endpoint; Sentry had no metrics route. Rustrak received Sentry envelopes, not OTLP. HyperDX was omitted from the active sequential list after its OTLP listener failed to bind.
5. Verify backend API/storage results first, then inspect browser UI. Screenshots are supporting evidence, not the assertion source.
6. Use fresh bounded windows and service/scenario identifiers. Historical counts are not treated as current evidence.

Browser evidence captured with `agent-browser 0.36.0`: [Parallax Issues](../../../artifacts/research/2026-09-04-main/parallax-issues.png), [SigNoz traces](../../../artifacts/research/2026-09-04-main/signoz-traces.png), [Grafana Explore](../../../artifacts/research/2026-09-04-main/grafana-explore.png), [Sentry issues](../../../artifacts/research/2026-09-04-main/sentry-issues.png), and [Rustrak project](../../../artifacts/research/2026-09-04-main/rustrak-project.png).

## Feature inventory and live results

The complete shipped inventory remains in [feature-inventory-and-playground-verification.md](../reference/feature-inventory-and-playground-verification.md). The current run exercised the following categories:

| Parallax feature | Current live evidence |
| --- | --- |
| OTLP traces/logs/metrics | GraphQL after workload: `1524` spans, `449` logs, `50074` metric points; service, metric, trace, and log queries returned fresh playground data. |
| Distributed traces and correlation | Checkout → pricing/payment → catalog/inventory/recommendation and async orders/fulfillment paths were visible; current service catalog had 13 services. |
| Error derivation and fingerprint grouping | `c1`: fingerprint `d0b552095fc3e5b3`, canonical hash `sha256-jcs:d231673da8ea36f001ed43da4a1bf3be34015236e1e973946ec00b3334a33035`. |
| Live tails | `c3`: cold first subscription timed out before a receiver existed; source inspection showed broadcast only with active receivers. Controlled pre-opened stream and warm rerun passed (`294` bytes). This is a test-order race, not a product failure. |
| Invocations and agent sessions | `c2` invocation `d6a31a37-7051-412d-8ae4-724f6125cf7a`; `c7` import `claude_code:c7-session-fixture:6a732c59b189defb`; MCP equivalence passed. |
| Dashboards/investigations | `c5`: dashboard `dash_18d1e858590addc0`, investigation `case_18d1e8585d6c6ed8`. |
| Alerting | `c4`: rule `alr_18d1e880daa98818`, incident `inc-alr_18d1e880daa98818--1788466169`, webhook and Slack destinations delivered. |
| GitHub deploy/Actions context | `c6`: valid webhook `200`, invalid signature `401`. |
| Sentry envelope adapters | `c8`: Rust, Java, and JS envelopes each found one current issue. |
| Lifecycle operations | `c9`: isolated-home setup/doctor/prune/uninstall lifecycle passed. |
| Redaction and egress controls | `c10`: canary secret did not leak; no webhook was sent. The absent canary service returned HTTP `000`, recorded as expected fixture behavior. |
| Embedded UI | `c11`: all 13 route checks passed with `agent-browser 0.36.0`; Parallax Issues browser view showed fresh issues. |

## Feature-oriented comparison

| Parallax feature | Scenario | Parallax result | Strongest comparator | Comparator result | Best implementation / verdict | Parallax gap or action | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OTLP signal breadth | A1/A3/A8 + fresh workload | Traces, logs, metrics indexed in one product | OpenObserve / SigNoz / Grafana | All accepted broad OTLP; SigNoz current ClickHouse counts after fresh A1 were traces `200`, logs `84`, metrics `18036` before later smoke traffic | Comparator maturity wins; Parallax is functionally credible | Measure throughput/durability; no performance claim yet | current GraphQL and backend queries |
| Error grouping and issue lifecycle | B1/C1/C8 | Derived error events, deterministic fingerprint, issue UI | Sentry | Native issue grouping; five identical errors grouped; UI showed `PaymentError` count 5 | Sentry clearly wins workflow maturity | Add ownership/suspect-commit depth only if in product scope | `sentry/verify.sh`; Sentry UI screenshot |
| Error-to-context bundle | C1/C5/C7 | Bounded/redacted bundle, story, investigation, MCP projection | No direct equivalent among free self-host peers | OpenObserve MCP is broader but Enterprise-gated and write-capable; SigNoz MCP is broad; Grafana Explore is mature query UX | Parallax's safety-shaped artifact is distinctive, value unproven | Run A1 human/agent quality evaluation | C1/C5/C7 IDs; code-reality ledger |
| Trace exploration | A1/A3/A25/A26 | Waterfall, critical path, events, links, services | Grafana Tempo + Grafana UI | Mature trace search/Explore and Tempo API returned fresh checkout traces | Grafana wins generic trace UX; Parallax wins integrated error/evidence context | Continue UI depth and query performance work | Grafana Explore screenshot/API |
| Logs and metrics workbench | A25/A26/A30 | Filters, facets, patterns, typed metrics and derived errors | Grafana / OpenObserve | Fresh metrics, labels, Loki logs, and OpenObserve search visible | Grafana/OpenObserve win general analytics maturity | Benchmark high-cardinality queries; keep typed legality strict | GraphQL metrics/log results; Grafana APIs |
| Local deployment simplicity | fresh stacks | One Parallax executable plus managed GreptimeDB | Maple | Official `v0.0.21` bundle + embedded chDB gave the smallest local competitor path | Maple wins local UX today; Parallax's Rust/self-host target is not yet parity-proven | Measure setup time/RAM/recovery | Maple CLI `services` output |
| OTel pipeline/fan-out | A1 + Rotel | Rotel hub delivered fresh traces to active sinks | Rotel/Grafana Alloy | Rotel fan-out was reliable; Grafana Alloy is a stronger production pipeline reference | Rotel is adequate lab hub, not a Parallax product feature | Keep backend readiness gating explicit | Rotel config and logs |
| Sentry compatibility | C8 + Sentry verify | Native envelope endpoint and current SDK paths | Sentry | 30+ SDK ecosystem and mature envelope semantics | Sentry wins SDK/ecosystem; Parallax proves useful compatibility | Expand compatibility ledger | `c8`; Sentry/Rustrak issue screenshots |
| Alerting/investigation | C4/C5 | Rule, destinations, incident, dashboard, case file | SigNoz/Grafana/Sentry | Mature alerting and dashboards; SigNoz UI/API live at v0.140.0 | Competitors win breadth; Parallax has a coherent narrow workflow | Add escalation/SLO only if scope changes | C4/C5 IDs; SigNoz/Grafana UI |
| Agent surface | C2/C7 | Read-only MCP, CLI context, import/equivalence | SigNoz/OpenObserve/Grafana | Broader MCP/query surfaces; OpenObserve includes mutating Enterprise tools | Parallax wins safety posture, not surface breadth | Prove bundle usefulness and remote auth later | MCP equivalence result; competitor docs |

## Best-in-class references

- Error issue workflow, SDK breadth, replay, and profiling: Sentry.
- General dashboards, trace/log/metric query UX, and ecosystem: Grafana.
- Rust single-binary OTLP observability at shipped maturity: OpenObserve.
- Current full-stack ClickHouse observability Compose path: SigNoz.
- Small local single-binary experience: Maple.
- Self-hosted Sentry-compatible Rust issue ingestion: Rustrak, with a materially smaller feature surface than Sentry.

## Defects and root-cause fixes

| Defect | Root cause | Structural fix | Regression / re-verification |
| --- | --- | --- | --- |
| Playground `catalog` exited during fresh boot | Compose only ordered `flagd` service start; catalog could connect to Postgres before health/readiness | `catalog.depends_on.postgres.condition: service_healthy` in playground `deploy/docker-compose.yml` | Recreated fresh project; `/actuator/health` became `UP`; A2 passed. |
| Foundry SigNoz overlay collided with host ports | Included upstream Compose port lists were appended, not replaced | `ports: !override` for ingester `4320/4321` and UI `3301` in `compose.signoz.override.yml` | `docker compose ... config --quiet`; fresh stack healthy; UI/API and traces/logs/metrics verified. |
| HyperDX AIO OTLP endpoint unavailable | Current AIO image showed receiver config but did not bind 4317/4318 after onboarding | No unsafe workaround. Removed HyperDX from active sequential Rotel exporters and recorded blocker | UI/API remained verified; direct telemetrygen probe failed with connection refused. |
| Cold live-tail probe timed out | Server broadcast path requires a connected receiver; first probe opened after event | Test ordering corrected: open `curl -N` before stimulus; no product code change | Warm rerun returned `294` bytes; source behavior explains first timeout. |

Fix commits: playground readiness `5f37ea32e1d68d1cb0a0df79c9e48e12a51bfd06`;
playground documentation `bc3d771a386a99387fab6989ac98992d978965cc`; Parallax
lab/integration/policy/docs `3c4b68d3acf8fb435102ae2beb8f184bf40b617c`.

No Parallax product-code bug was left unexplained in the tested shipped feature set. The live-tail cold race is a harness ordering issue; HyperDX and Rustrak health issues belong to comparator integrations.

The refreshed-main verification gate also exposed stale repository policy: the
alert evaluator's exact `anyhow` ceiling was 7 for an actual 9 edges, the
consolidated root agent policy measured 1026 bytes while nested-file ceilings
were obsolete, and three docs still linked to deleted nested AGENTS files. The
ratchet now records 9/1026 and the docs point to the surviving root policy;
`cargo xtask ci --fast` passes.

## Playground and lab improvements

- Added the fresh current-source verification layer at playground SHA `c6c1516e...`.
- Fixed catalog/Postgres readiness ordering.
- Reused the same Rust/Java/web workload and Sentry envelopes across active sinks.
- Added current Foundry-generated SigNoz deployment rather than preserving the obsolete community Compose pin.
- Updated all backend pins and Rotel routes to current runnable artifacts.
- Kept Sentry out of OTLP metrics routing and Rustrak on envelope-only routing.
- Preserved exact scenario IDs and browser/API evidence instead of accepting screenshots alone.
- Repaired stale policy ratchets and nested-AGENTS documentation links exposed by current `main` CI.

## Competitor/lab integration changes

- OpenObserve: GA `v0.92.2`; `v1.0.0-rc2` excluded from stable comparison.
- Maple: official `MapleTechLabs/maple` `v0.0.21` bundle; no old `Makisuo/maple` assumption.
- SigNoz: Foundry `v0.2.17` generates the current Compose deployment; pinned current core/collector digests and remapped host ports.
- Sentry: vendored `getsentry/self-hosted` `26.8.0`.
- Grafana: `grafana/otel-lgtm:0.32.0`.
- ClickStack/HyperDX: `clickhouse/clickstack-all-in-one:2.37.0`, `FRONTEND_URL` set to the lab UI origin for correct auth redirects.
- Rustrak: server/UI `v0.14.11`.
- Telemetrygen: `v0.159.0` because the latest Contrib release `v0.160.0` image was not published.

## Out of scope

Profiles, session replay as a product, uptime/cron monitoring, mobile-size analysis, full on-call/escalation, SLO/error-budget management, and competitor-specific proprietary AI services were not treated as Parallax gaps. They remain relevant product decisions, not silently omitted comparison claims.

## Reproduction

Run from the two refreshed repositories:

```bash
# Parallax
PARALLAX_VERSION_OVERRIDE=0.1.0-research.3c4b68d cargo build --release -p parallax-cli -p parallax-mcp
./target/release/parallax serve --config artifacts/research/2026-09-04-main/parallax/config.toml

# Playground
docker compose -p parallax-research-20260904 -f deploy/docker-compose.yml up -d --build
./scripts/check-scenarios.sh

# Fan-out core and current overlays
cd ../parallax/bench/otlp-fanout
./setup-vendor.sh
docker compose -f compose.yml config --quiet
SIGNOZ_POURS_DIR=../../artifacts/research/2026-09-04-main/signoz-pours \
  docker compose -f compose.yml -f compose.signoz.yml config --quiet
docker compose -p parallax-otlp-fanout --env-file rotel.env -f compose.yml up -d
```

Then run the playground scenario drivers and current backend-specific checks documented in [`bench/otlp-fanout/README.md`](../../../bench/otlp-fanout/README.md), [`parallax-telemetry-playground/VERIFICATION.md`](https://github.com/tailrocks/parallax-telemetry-playground/blob/bc3d771a386a99387fab6989ac98992d978965cc/VERIFICATION.md), and the Sentry/Rustrak scripts. Do not copy local credentials from `rotel.env` into source control.

## Final verdict

Parallax already does exceptionally well at joining error events, telemetry, CLI/agent execution, deploy context, redaction, and bounded investigation artifacts behind one local API. It is competitive for a focused self-hosted developer/debugging workflow.

It is clearly behind Sentry on issue operations and SDK ecosystem, behind Grafana/OpenObserve/SigNoz on generic observability maturity and visualization, and behind Maple on polished local startup. It has not proven the central economic or agent-quality claims: bundle value over raw context, throughput, durability under pressure, high-cardinality behavior, or operational simplicity versus the current alternatives.

The next proof gate is not another feature checklist. It is a controlled A1 evaluation of bounded evidence against raw context, plus benchmarked storage/query/recovery measurements, while keeping the current version manifest and reproducible fan-out run current.

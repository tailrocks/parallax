# Urgentry Sentry Tiny Benchmark Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check the existing Urgentry claim from primary sources because prior Parallax
notes treated it mostly as "Tiny mode plus benchmark claims." That was too
thin: Urgentry may be the strongest lightweight pressure on Parallax's
Sentry-compatible simplicity story, and its vendor benchmarks are easy to
over-read without source-level limits.

## Short Verdict

Urgentry is a stronger Sentry-replacement warning than the previous watchlist
said. Source confirms a broad Sentry-style ingest and product surface: legacy
`/store/`, envelopes, minidumps, security reports, OTLP HTTP/JSON traces, logs,
and metrics, plus envelope side effects for transactions, sessions, replays,
profiles, client reports, check-ins, attachments, and metric buckets.

It does **not** close the Parallax wedge:

- license is FSL-1.1-ALv2 source-available, not OSI-open;
- benchmark numbers are vendor claims and were not reproduced in this pass;
- OTLP is HTTP/JSON only in checked source, with protobuf explicitly rejected;
- no MCP server was found in checked README/docs/source;
- the `autofix` API is a deterministic compatibility/stub-like surface that
  records completed summaries and skipped PR behavior, not a real AI/coding
  agent fixer loop;
- no portable evidence-bundle schema, source-policy manifest, redaction report,
  projection hash, missing-evidence model, or action/outcome audit was found.

Parallax should treat Urgentry as a high-quality simplicity baseline and a
Sentry-protocol coverage challenge, not as proof that the open evidence-context
engine thesis is closed.

## Version And Source Snapshot

| Field | Current check |
| --- | --- |
| Repository | [urgentry/urgentry](https://github.com/urgentry/urgentry) |
| Latest release | [`v0.2.12`](https://github.com/urgentry/urgentry/releases/tag/v0.2.12), published 2026-05-22 |
| Latest checked `main` | [`ccc0ff815ec8b19d3b7c820b95bc3d539414e145`](https://github.com/urgentry/urgentry/commit/ccc0ff815ec8b19d3b7c820b95bc3d539414e145), dated 2026-05-22 |
| Visible traction at check time | Roughly 55 GitHub stars and 5 forks |
| License | [FSL-1.1-ALv2](https://github.com/urgentry/urgentry/blob/main/LICENSE), with Apache-2.0 future-license language |
| Runtime/build note | `go.mod` declares Go `1.26.0`; quickstart says Go 1.26+ |

## Ingest Surface Checked

The checked HTTP server mounts these ingest endpoints when the ingest role is
enabled:

| Route family | Source-level read |
| --- | --- |
| Sentry event/store | `POST /api/{project_id}/store/` accepts legacy JSON events and returns an event id. |
| Sentry envelope | `POST /api/{project_id}/envelope/` parses envelopes, queues `event` and `transaction` items, applies transaction sampling when configured, then persists side effects. |
| Native/security reports | `minidump`, `unreal`, `security`, `csp-report`, and `nel` routes are registered. |
| OTLP | `POST /api/{project_id}/otlp/v1/{traces,logs,metrics}/` routes exist. Checked handlers accept JSON and reject `application/x-protobuf` with `415`; no gRPC receiver was found in this pass. |

Envelope side effects are materially broader than Rustrak or GoSnag in their
current checked forms. Urgentry handles or stores:

- `event` and `transaction` through the pipeline;
- `user_report`;
- `attachment`;
- `replay_event`, `replay_recording`, `replay_recording_not_chunked`, and
  `replay_video`;
- `profile`;
- `client_report`;
- `session` and `sessions`;
- `check_in`;
- `statsd` and `metric_buckets`;
- unknown item types as logged skips.

This matters for Parallax because "Sentry-compatible" competitors are no longer
only error-event parsers. A Parallax compatibility ledger must distinguish:

```text
error-event ingest
vs
explicit unsupported-item outcomes
vs
broad Sentry replacement behavior
```

## Deployment Shape

| Mode | Source-level shape | Parallax implication |
| --- | --- | --- |
| Tiny | `docs/tiny/README.md` describes the full product in one process with one SQLite data directory. Backup is copying `URGENTRY_DATA_DIR` or the mounted volume. | This is the low-ops baseline Parallax's first useful bundle must stay near. |
| Self-hosted | `docs/self-hosted/README.md` describes split `api`, `ingest`, `worker`, and `scheduler` roles on PostgreSQL, MinIO, Valkey, and NATS. | This is a reasonable scale-out topology, but not the tiny-tier bar. |
| Compose | The checked Compose file includes PostgreSQL, MinIO, Valkey, NATS, MinIO/bootstrap helpers, four Urgentry roles, and optional ClickHouse profile. | Count helper/init services separately from steady-state roles in the simplicity benchmark. |

## Benchmark Claim Boundary

Urgentry publishes benchmark tables for Tiny, self-hosted Urgentry, and
self-hosted Sentry. The important source-level caveat is methodological: the
benchmark note says the workload is intentionally narrow and covers envelope
ingest, a 70/30 small/medium error mix, and issue/event query probes after load.

Do not turn these into Parallax evidence without a benchmark artifact:

- Tiny claims: `400 eps`, ingest p95 around `10.08 ms`, query p95 around
  `78.66 ms`, peak memory around `52.3 MB`.
- Self-hosted Urgentry claims: `2200 eps`, ingest p95 around `0.71 ms`, query
  p95 around `48.82 ms`, peak memory around `391.8 MB`.
- Self-hosted Sentry `26.3.1` reference claim: `1000 eps`, query p95 around
  `1400.81 ms`, peak memory around `8191.7 MB`.
- Small-box note says Sentry self-hosted did not complete on that host.

These are useful benchmark-design inputs and positioning pressure. They are not
measured Parallax evidence.

## Agent And Autofix Boundary

No MCP server was found by checking README, docs, `internal`, `cmd`, or `deploy`
for `MCP`/`Model Context Protocol`/`mcp`.

There is an `autofix` API path under issue routes, but checked source builds a
deterministic payload from issue title/culprit/operator instruction, stores the
run as `COMPLETED`, records empty repositories/codebases, and, for `open_pr`,
sets pull request status to `SKIPPED` because no linked repository integration
is available. That should not be counted as an AI fixer, PR-opening agent, or
outcome loop.

There is also a Sentry-shaped `GET /api/0/seer/models/` stub returning an empty
AI models list. Count this as Sentry API compatibility surface, not agent-native
debugging capability.

## Parallax Impact

What Urgentry weakens:

- "simpler than self-hosted Sentry" as a standalone public claim;
- "Sentry-compatible replacement" as a unique migration story;
- any fixture plan that only tests error events and ignores transactions,
  sessions, client reports, replay/profile/check-in behavior;
- any OTLP claim that does not say whether protobuf/gRPC is supported.

What Urgentry does not weaken:

- open-source thesis, because FSL is source-available;
- portable evidence-bundle schema;
- source-policy and missing-evidence semantics;
- redacted, hash-equivalent CLI/API/MCP projections;
- coding-agent command/file/approval/patch/test audit;
- accepted/rejected/reverted fix-outcome corpus.

## Required Parallax Response

1. Keep Urgentry in the self-hosted simplicity baseline, but label vendor
   performance numbers unmeasured until reproduced by benchmark-agent artifacts.
2. Update Sentry compatibility wording to require explicit unsupported-item
   outcomes, not silent drops, because Urgentry shows broader item handling can
   fit a lightweight product.
3. Keep the tiny tier from requiring Postgres, Redis/Valkey, NATS, MinIO, a
   separate UI, a Collector, or MCP before the first bundle works.
4. Do not answer Urgentry by becoming a broad Sentry clone. Answer with the
   open evidence contract and action/outcome audit.
5. If Parallax uses OTLP in public wording, distinguish OTLP HTTP/JSON, OTLP
   HTTP/protobuf, and OTLP/gRPC conformance.

## Falsification Triggers

Reopen the GO verdict or narrow the Parallax wedge if Urgentry publishes any of
the following:

- OSI-open license change;
- read-only MCP/CLI/API evidence-bundle export with schema and redaction
  manifest;
- independently reproducible benchmark artifacts under a shared protocol;
- OTLP protobuf/gRPC receiver support and Collector-equivalence evidence;
- real AI/coding-agent remediation with patch/PR/outcome audit;
- portable evidence graph or bundle format with missing-evidence semantics.

## Sources

- [Urgentry repository](https://github.com/urgentry/urgentry)
- [Urgentry release `v0.2.12`](https://github.com/urgentry/urgentry/releases/tag/v0.2.12)
- [Urgentry license](https://github.com/urgentry/urgentry/blob/main/LICENSE)
- [Urgentry benchmark docs](https://github.com/urgentry/urgentry/blob/main/docs/benchmarks.md)
- [Urgentry Tiny mode docs](https://github.com/urgentry/urgentry/blob/main/docs/tiny/README.md)
- [Urgentry self-hosted docs](https://github.com/urgentry/urgentry/blob/main/docs/self-hosted/README.md)
- [Urgentry Compose file](https://github.com/urgentry/urgentry/blob/main/deploy/compose/docker-compose.yml)
- [Urgentry HTTP route source](https://github.com/urgentry/urgentry/blob/main/internal/http/server.go)
- [Urgentry envelope handler source](https://github.com/urgentry/urgentry/blob/main/internal/ingest/envelope_handler.go)
- [Urgentry envelope side-effect source](https://github.com/urgentry/urgentry/blob/main/internal/ingest/envelope_side_effects.go)
- [Urgentry OTLP handler source](https://github.com/urgentry/urgentry/blob/main/internal/ingest/otlp_handler.go)
- [Urgentry metrics OTLP handler source](https://github.com/urgentry/urgentry/blob/main/internal/ingest/otlp_metrics_handler.go)
- [Urgentry Autofix API source](https://github.com/urgentry/urgentry/blob/main/internal/api/autofix.go)

## Bottom Line

Urgentry raises the bar for lightweight Sentry-compatible breadth and setup
simplicity. It does not replace Parallax's research target unless it adds open,
portable, redacted evidence bundles plus real coding-agent action and outcome
audit.

# Audit: research docs vs shipped code (2026-07-17)

**Goal:** Re-read project research against `main` source; correct stale product
claims; improve navigation; keep market comparisons multi-angle and non-biased
(capability + economics + openness/hidden cost).

**Code authority:** [code-reality-ledger.md](../code-reality-ledger.md)

## Verified against source (sample)

| Area | Code reality | Was research wrong? |
| --- | --- | --- |
| OTLP gRPC/HTTP all 3 signals | `parallax-server` otlp_* + `parallax-ingest` | Mostly OK; some market cells still 🏗 planned |
| Sentry envelope HTTP | `sentry_http.rs` + `sentry_envelope.rs` + `analysis/sentry.rs`; plan **118 DONE** | **Yes** — root README + design bodies still "future adapter / not V1" |
| GreptimeDB + Turso mandatory | adapters + AGENTS.md policy | OK on decisions; residual historical fallback language bannered |
| GraphQL surface | `ui/graphql/schema.graphql`: **76** Query, **14** Mutation | "80/15" interim claims inverted truth — **corrected to 76/14** |
| CLI | serve, invocation, issue, trace, metrics, logs, traces, sql, doctor, prune, uninstall, context | OK |
| UI | ~16 feature modules under `ui/src/features/` | "19-route" prose was approximate |
| Local MCP | `parallax-mcp` plan 112 DONE | Legacy matrices still "MCP planned" |
| Evidence bundle + redaction | `parallax-evidence` + `parallax-redaction` | Code shipped; **A1 value unproven** — keep gate language |
| Fixer outcome loop | plan 123 only | Correctly unshipped |
| Autonomous loop | `poc/evidence-loop` | Correctly PoC-only |

## Fixed this pass

1. **Code-reality ledger** created (`docs/research/code-reality-ledger.md`).
2. **Root README** — status + Working Direction: Sentry is shipped, not future/V1-excluded; link ledger.
3. **Research index + agenda** — front door to ledger; GraphQL 76/14; conventions; plan 118 DONE wording.
4. **capture/sentry-ingest.md** — historical body banner; executive summary no longer present-tense "future only".
5. **capture/rust.md**, **storage/streaming/messaging-…** — Sentry "future" lines corrected/bannered.
6. **Market competitors** — bulk replace OTLP/error 🏗 → ✅🧪 shipped pre-release across 31 deep-dives; TMA1 BLUF no longer "Parallax has not shipped architecture"; SigNoz/Datadog/Sentry/TMA1 economics/hidden-cost axes tightened; competitors README economics rules + correction invitation; combination claim no longer "(future) Sentry".
7. **Legacy matrices** — stronger historical banners on `observability-feature-matrix.md`, `competitive-comparison-matrix.md`, `closest-to-parallax-ranked.md` update.

## Deferred (honest)

| Item | Why deferred |
| --- | --- |
| Full rewrite of every capture/storage white paper body | ~280-file corpus; priority was current-truth surfaces + code-claim alignment |
| Re-fetch all competitor prices/versions from live web | Environment not required for code-reality; cells already dated 2026-07-17 where re-verified — mark **unverified** only when re-fetch fails (none invented this pass) |
| Complete A1/A2 empirical gates | Explicit non-goal |
| Four-way large server storage benches | Explicit non-goal |
| Dual-maintain every legacy deep-research.md number | Canonical = `competitors/`; legacies are sources |
| Ingest-time A6 redaction product completion | Code + gate remain as designed |

## Objectivity posture (confirmed)

- Competitor deep-dives keep scoped "who wins" per axis; no unscoped "Parallax is better overall."
- A1 / unproven language retained for bundle value and outcome loop.
- Economics cover sticker + self-host ops + contribute/lock-in + ecosystem size (OSS free ≠ free TCO; closed SaaS has contribute-block + lock-in).
- Corrections invited via PR + primary sources (competitors README rule 7).

## How to re-verify later

```bash
# Staleness patterns
rg -n -i 'future (adapter|migration)|Sentry.*future|not V1 scope' docs/research README.md --glob '*.md'
rg -n 'OTLP.*(🏗)|error_event.*\(🏗\)' docs/research/market/competitors --glob 'parallax-vs-*.md'
rg -n '76 quer|14 mutation|76 Query|14 Mutation|76 GraphQL' docs/research --glob '*.md'
# Ledger still points at real paths
test -f crates/parallax-server/src/sentry_http.rs
test -f crates/parallax-mcp/src/main.rs
test -f schema/evidence-bundle.v1.schema.json
```

## Follow-up (skeptic pass, same day)

Present-tense GraphQL counts fixed to **76/14** (schema SoT; 80/15 was wrong) on `api-concept`, `strategic-coverage`,
`rust-workspace-map`, `v1-implementation-spec`, and the full-observability
snapshot note. Thesis no longer says plan 118 “owns remaining migration.”
Live assert: `ui/graphql/schema.graphql` = 76 Query / 14 Mutation.

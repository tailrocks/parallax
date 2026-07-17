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
| GraphQL surface | `ui/graphql/schema.graphql`: **76** Query, **14** Mutation | SoT when description blocks are skipped; naive line-match yields false **80/15** — see ledger count method |
| CLI | serve, invocation, issue, trace, metrics, logs, traces, sql, doctor, prune, uninstall, context | OK |
| UI | ~16 feature modules under `ui/src/features/` | "19-route" prose was approximate |
| Local MCP | `parallax-mcp` plan 112 DONE | Legacy matrices still "MCP planned" |
| Evidence bundle + redaction | `parallax-evidence` + `parallax-redaction` | Code shipped; **A1 value unproven** — keep gate language |
| Fixer outcome loop | offline SM + Turso outcomes **shipped** (plan 123 DONE); draft-PR deferred; live value **unproven** | Was stale “active plan 123” with dead plan path — fixed to validation evidence |
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
# Wrong GraphQL count (naive description-token parse) — present-tense should be 76/14
rg -n '80 quer|15 mutation|80 Query|15 Mutation|\*\*80\*\*.*mutation|80/15' docs/research README.md --glob '*.md'
# Ledger still points at real paths
test -f crates/parallax-server/src/sentry_http.rs
test -f crates/parallax-mcp/src/main.rs
test -f schema/evidence-bundle.v1.schema.json
```

## GraphQL field-count method (do not thrash)

Live SoT for root fields is **`ui/graphql/schema.graphql`**:

1. Take the body of `type Query { … }` / `type Mutation { … }`.
2. **Skip** multiline `"""…"""` and single-line `"…"` description blocks.
3. Match only lines that start a field: `name(` or `name:`.

That yields **76 Query / 14 Mutation** (2026-07-17). A naive
`^\s+\w+\(` over every non-comment line **inside** descriptions also matches
prose (`registration`, `kind`, `legality`, `retention`) and falsely reports
**80/15**. Any recount must use the description-skipping method above (or
Juniper `impl Query` / `impl Mutation` field methods — same total).

## Follow-up (same day)

- Present-tense product surfaces hold **76/14** after structural pass `2426cce8`.
- Thesis plan **118 DONE** ownership fixed earlier.
- Ledger + this audit document the count method so 80/15 thrash does not recur.

## Re-verification (goal re-entry, same day)

Full plan verification re-run against live `main`:

- Ledger path gate: all cited non-glob paths exist; GraphQL **76/14** (description-skipping method).
- Product-level “future Sentry / not V1” greps: clean.
- Competitor OTLP/`error_event` 🏗 greps: clean.
- Canonical competitors multi-angle (SigNoz OSS, Sentry closed SaaS, TMA1 peer, Datadog SaaS): price/TCO/contribute/lock-in + A1 unproven retained.
- **Gap closed:** dead `plans/123-fixer-outcome-loop.md` ownership on ledger, agenda, fixer-boundary, problem-audience — plan **123 DONE** offline residual; point at validation evidence.

## Competitor prose pass (skeptic, same day)

Emoji cells were already ✅🧪; body prose still said “plans compatibility / planned/unproven”
for shipped error derivation, Sentry envelope, bundle assembler, and treated fix-outcome as
fully unshipped. Bulk-fixed `parallax-vs-*.md` + `competitors/README.md`:

- Sentry envelope → **ships** (plan 118 DONE; multi-SDK unproven)
- error derivation → **shipped** (pre-release)
- evidence bundle → **code-shipped**, A1 **value** unproven
- fix-outcome → offline residual **plan 123 DONE**; live value unproven (not “unshipped”)
- README Planned-only list no longer includes fix-outcome as fully planned

### Second skeptic pass — residual verdict/edge lists

Fixed present-tense leftovers the first pass missed:

- `parallax-vs-tma1.md` verdict + honest-read residual
- `parallax-vs-coroot.md` “Where Parallax edges” list (error planned → shipped; bundle planned → code-shipped)
- `parallax-vs-highlight.md` “backend telemetry unshipped” → OTLP shipped
- `parallax-vs-sumo.md` production error-workflow planned → shipped
- fix-outcome “planned/unproven” on maple/dynatrace/holmesgpt/signoz → plan 123 DONE offline residual
- SigNoz/HolmesGPT MCP cells: local-stdio shipped (not 🏗 planned)

Folder re-grep for product deep-dives: clean on planned/unshipped for those surfaces.

## Structural phrase-lock pass (strategist, same day)

Locked six ledger status phrases and applied a whole-folder mechanical pass on
`competitors/parallax-vs-*.md` + `README.md`. Cleared residual freestyle
`designed (unproven)` on SigNoz (bundle, fix-outcome, redaction) and false-positive
same-line `bundle.*planned` hits. Strategist residual gate re-run: **EMPTY**.

# Loop-stage claim status recheck (agenda #6, pass 154)

<!-- markdownlint-disable MD013 -->

**Research date:** 2026-07-17  
**Pass:** 154  
**Claim under test:** Detect/Dispatch/Validate/Learn stage designs hold under
replayed telemetry (research-agenda item **#6**).  
**Verdict:** **Split** — offline kernels + plan 123 offline residual **exist**;
live replay / Detect-trigger ledger / product-level stage claims remain
**`not_measured`**.

## Why this pass

Agenda #6 is the autonomous-fix-loop measurement gate. Risk of drift: treating
`poc/evidence-loop` kernels or plan 123 offline outcome SM as product passes for
Detect precision/recall or dispatch idempotency on real telemetry.

## Evidence class

| Layer | Status | Evidence |
| --- | --- | --- |
| Loop design | **Present** | [architecture/autonomous-fix-loop.md](../architecture/autonomous-fix-loop.md) |
| PoC 20 kernels | **Present** | `poc/evidence-loop/` — fixtures only |
| Claim map | **Present** | [poc-evidence-loop-coverage.md](../architecture/poc-evidence-loop-coverage.md) — every product claim still `not_measured` |
| Detect trigger ledger | **Absent** | No `detect-trigger-ledger` / dated precision-recall rows under `docs/research/` |
| Plan 123 offline residual | **DONE (offline only)** | [2026-07-plan-123-fixer-offline/README.md](2026-07-plan-123-fixer-offline/README.md) — `fixer_outcome` SM + Turso append-only |
| Product `fixer_outcome` tests | **Pass (unit)** | `cargo test -p parallax-evidence --lib fixer_outcome --locked` → **3 ok** (2026-07-17): PR≠success; review+no-recurrence required; multi-arm failures preserved |
| Live replay harness | **Open** | No dated product run over recorded multi-service telemetry for Detect/Dispatch |
| Draft-PR adapter | **Deferred** | Explicit STOP in plan 123 — not core |

## Allowed vs forbidden wording

**Allowed:**

- "Offline outcome residual and PoC kernels exist for the loop design."
- "Opened PR is never success; offline tests enforce review + non-recurrence."
- "Live Detect/Dispatch/Learn product claims remain not measured."

**Forbidden:**

- "Loop stages validated in production" / "Detect precision proven" from PoC alone.
- "Autonomous fixer product ready" from plan 123 DONE.

## Falsification / upgrade path

1. Create Detect trigger ledger + freeze replay corpus.  
2. Measure precision/recall and dispatch idempotency on that corpus.  
3. Publish dated result rows; only then raise stage claim levels.  
4. Live outcome feedback learning still requires product path beyond offline SM.

## Parallax goal fit

North-star loop is design pressure, not V1 product. Pass 154 keeps **offline
machinery** distinct from **gate pass** — same discipline as A1/A4/A6.

## Sources checked

- Repo: plan 123 README; `poc/evidence-loop`; `parallax-evidence::fixer_outcome` tests (ran).  
- Design: autonomous-fix-loop.md gate table; poc-evidence-loop-coverage.md.  

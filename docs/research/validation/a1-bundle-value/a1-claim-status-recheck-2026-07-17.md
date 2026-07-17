# A1 claim status recheck — design/code vs empirical (pass 118 + pass 159)

<!-- markdownlint-disable MD013 -->

**Research date:** 2026-07-17 (pass 118); **pass 159 recheck 2026-07-18**  
**Pass:** 118 + **159** (deep-research-parallax indefinite program)  
**Claim under test:** "A bounded Parallax evidence bundle makes a coding agent's diagnosis and fix materially better than raw context" (kill criterion 3 / assumption A1).  
**Verdict this pass:** **`not_measured`** — unchanged. Product and eval *machinery* exist; **no published comparative agent run** exists in this repository.

## Why this pass

A1 is the #1 product-validation risk on the [research agenda](../../research-agenda.md). Prior notes
design the experiment thoroughly; shipment notes say the real bundle producer should be the eval
subject. The risk of drift is conflating **(a)** "we can assemble a bundle" with **(b)** "bundles beat
raw context on fix quality." This pass separates those layers with live repo + primary-source checks.

## Evidence class

| Layer | Status | Evidence |
| --- | --- | --- |
| Experiment design | **Present** | [bundle-value-evaluation.md](bundle-value-evaluation.md), [runtime-dependence-and-raw-baseline.md](runtime-dependence-and-raw-baseline.md), [bundle-value-phase0-runbook.md](bundle-value-phase0-runbook.md) |
| Overlay / no-cheat contract | **Present** | [phase0-telemetry-overlay-contract.md](phase0-telemetry-overlay-contract.md) |
| Result-ledger *policy* | **Present** | [a1-eval-result-ledger-and-model-refresh.md](a1-eval-result-ledger-and-model-refresh.md) defines claim levels and artifact shape |
| Result-ledger *instance* | **Absent** | No `result-ledger.md`, no `result-ledger.jsonl`, no dated run_id under this directory (repo search 2026-07-17; **pass 159 reconfirm**: only policy file `a1-eval-result-ledger-and-model-refresh.md` matches `*ledger*` / `*result*`) |
| Bundle producer (product) | **Present** | `crates/parallax-evidence` — golden stability test `bundle_v1_golden_fixture_is_stable` loads `fixtures/bundle-v1-golden.json` (**pass 159:** `cargo test -p parallax-evidence bundle_v1_golden` → **ok**) |
| Loop PoC fixtures | **Present** | `poc/evidence-loop/` with `fixtures/` (agent-sessions, OTLP, outcome-rows, etc.) — mechanism proof, not A1 arms |
| Comparative agent arms A/B/B′/C/D | **Not run** | No per-arm resolved-rate rows, no model snapshot, no C-vs-B delta |
| Public SWE-style task sources | **Still available** | [SWE-bench/SWE-bench_Lite](https://huggingface.co/datasets/SWE-bench/SWE-bench_Lite) still published (HF HTTP 200; API `lastModified` **2025-04-29**, ~25.6k downloads — pass 159). Does **not** include production telemetry — still issue+repo only |

## What is *not* A1 pass

Do not upgrade claim level from any of the following alone:

1. **Golden bundle JSON stability** — proves schema serialization is deterministic, not that agents fix better.
2. **`poc/evidence-loop` kernels** — prove Detect/Dispatch/outcome residual shapes offline; they are not Phase-0 arms with hidden tests.
3. **MCP / GraphQL bundle delivery** — proves the product can *serve* a bundle; does not grade fix quality vs raw dump.
4. **SWE-bench leaderboard movement** — measures repo+issue coding agents; [field gap](bundle-value-evaluation.md) (no telemetry leg) remains.

## Claim level (policy labels)

Per [a1-eval-result-ledger-and-model-refresh.md](a1-eval-result-ledger-and-model-refresh.md):

| Label | Met? |
| --- | --- |
| `not_measured` (implicit default) | **Yes** — no run ledger |
| `harness_debug` | **No** — no incomplete run published either |
| `provisional_signal` / `a1_gate_pass` / `production_claim` | **No** |

Allowed public wording until a ledger exists:

> A1 is **not measured**. The product can produce bounded evidence bundles; whether those bundles beat raw telemetry context for agent fix quality is an open experiment.

Forbidden wording:

> "Bundles improve agent fixes" / "A1 passes" / "the schema moat is validated by eval."

## Falsification / upgrade path (unchanged, still owed)

1. Freeze task manifest + HF pins per [a1-task-source-freeze-check.md](a1-task-source-freeze-check.md).
2. Generate Phase-0 overlays under [phase0-telemetry-overlay-contract.md](phase0-telemetry-overlay-contract.md).
3. Run arms against the **shipped** `parallax-evidence` producer (not a mock packer).
4. Publish `result-ledger.md` + per-run JSONL with model snapshots and C-vs-B (or C-vs-B′) by runtime class R0–R3.
5. Only then assign `provisional_signal` or higher.

**What would falsify the product bet (not this recheck):** C ≤ B (or ≤ B′) on R1–R3 after a clean Phase-0 gate — kill criterion 3.

## Pass 159 addendum (2026-07-18)

Desk + code recheck only — **still `not_measured`.**

| Check | Result |
| --- | --- |
| Result-ledger instance files | **None** (policy markdown only) |
| Golden fixture test | **Pass** (`bundle_v1_golden_fixture_is_stable`) |
| SWE-bench_Lite source liveness | **Live** (HF 200 + API) |
| Comparative arms | **Still not run** |

Upgrade path in §Falsification unchanged: freeze → overlays → arms → publish ledger. Golden ok ≠ A1 gate.

## Pass 192 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** (policy markdown only) |
| Golden fixture test | **ok** |
| SWE-bench_Lite | HTTP **200** |
| Claim level | still **`not_measured`** |

## Pass 214 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** |
| Claim level | still **`not_measured`** |

## Pass 222 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Pass 239 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (`bundle_v1_golden_fixture_is_stable`) |
| Claim level | still **`not_measured`** |

## Pass 246 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** (only policy `a1-eval-result-ledger-and-model-refresh.md`; no `result-ledger.md` / JSONL / run_id dirs) |
| Golden fixture test | **ok** — re-ran `cargo test -p parallax-evidence bundle_v1_golden` → `bundle_v1_golden_fixture_is_stable` **passed** |
| SWE-bench_Lite | HF dataset page HTTP **200** (liveness only; not a freeze pin) |
| Comparative arms A/B/B′/C/D | **Still not run** |
| Claim level | still **`not_measured`** |

**Not A1:** golden schema stability + HF liveness + product producer existence. Next real move remains operator-owned Phase-0: freeze → overlays → arms → publish ledger.

## Pass 265 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** (policy file only) |
| Golden fixture test | **ok** — re-ran `cargo test -p parallax-evidence bundle_v1_golden` |
| SWE-bench_Lite | HF HTTP **200** |
| Comparative arms | **Still not run** |
| Claim level | still **`not_measured`** |

## Pass 277 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Comparative arms | **Still not run** |
| Claim level | still **`not_measured`** |

## Pass 284 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Comparative arms | **Still not run** |
| Claim level | still **`not_measured`** |

## Pass 290 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran with loop hygiene same pass) |
| Comparative arms | **Still not run** |
| Claim level | still **`not_measured`** |

## Pass 296 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Comparative arms | **Still not run** |
| Claim level | still **`not_measured`** |

## Pass 299 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Comparative arms | **Still not run** |
| Claim level | still **`not_measured`** |

## Pass 304 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** — only policy `a1-eval-result-ledger-and-model-refresh.md`; no `result-ledger.md` / JSONL / dated run_id dirs under `a1-bundle-value/` |
| Golden fixture test | **ok** — re-ran `cargo test -p parallax-evidence bundle_v1_golden` → `bundle_v1_golden_fixture_is_stable` **passed** |
| SWE-bench_Lite | HF dataset page HTTP **200**; API `lastModified` still **2025-04-29**, downloads ~**25,648** (liveness only — not a Phase-0 freeze pin) |
| Comparative arms A/B/B′/C/D | **Still not run** |
| Claim level | still **`not_measured`** |

**Not A1:** golden schema stability + HF liveness + product producer. Kill "bundles do not beat raw" still **unfired** because **open ≠ failed**. Next real move remains operator-owned Phase-0: freeze → overlays → arms → publish ledger.

## Pass 313 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran `bundle_v1_golden`) |
| SWE-bench_Lite | HF HTTP **200** (liveness only) |
| Comparative arms | **Still not run** |
| Claim level | still **`not_measured`** |

## Pass 321 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Comparative arms | **Still not run** |
| Claim level | still **`not_measured`** |

## Pass 325 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Pass 328 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Pass 333 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Pass 337 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Pass 343 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| SWE-bench_Lite | HF HTTP **200** |
| Claim level | still **`not_measured`** |

## Pass 346 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Pass 349 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Pass 352 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Pass 355 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Pass 358 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Result-ledger instance | Still **absent** |
| Golden fixture test | **ok** (re-ran) |
| Claim level | still **`not_measured`** |

## Uncertainty


- Pass 118 did **not** execute agent runs (token cost + harness ownership sit outside pure desk research; agenda still marks comparative runs owed). Pass 159/192/214/222/239/246/265/277/284/290/296/299/304/313/321 re-ran **only** the golden unit test, not arms.
- HF row counts / revision SHAs for Lite were not re-hashed this pass; source liveness only. Pinning remains in the freeze-check notes when Phase 0 starts.

## Parallax goal fit

Confirms the existential risk is still **empirical**, not design debt. Shipping the bundle producer and UI does **not** close A1. Research record must keep `not_measured` loud so marketing, go-no-go language, and competitor notes do not smuggle an unrun eval.

## Sources checked

- Repo: `docs/research/validation/a1-bundle-value/*` (no result-ledger instance); `crates/parallax-evidence/src/bundle/tests.rs` golden; `poc/evidence-loop/`.
- Primary external: [SWE-bench_Lite on Hugging Face](https://huggingface.co/datasets/SWE-bench/SWE-bench_Lite); [SWE-bench Lite overview](https://www.swebench.com/lite.html).
- Internal policy: [a1-eval-result-ledger-and-model-refresh.md](a1-eval-result-ledger-and-model-refresh.md), [research-agenda.md](../../research-agenda.md) item 1.

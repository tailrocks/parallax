# A1 — Bundle Value (does a Parallax bundle beat raw context?)

> **A1 is the existential assumption.** Kill criterion 3: a Parallax evidence bundle must
> measurably improve coding-agent fix quality versus a raw-telemetry-dump control. The eval
> design, seed corpus, Phase-0 runbook, deterministic overlay contract, and result/governance
> ledgers below are all in service of one dated, reproducible number — which is currently
> **`not_measured`**. No "bundles beat raw context" claim is allowed until the Phase-0 eval runs
> and publishes per-arm results through the ledger.

This directory groups the A1 bundle-value evidence (assumption A1 in the A1–A7 set). Files are
kept separate because each is an executable contract on its own.

## Eval design and runbook
- [bundle-value-evaluation.md](bundle-value-evaluation.md) — experiment design for the existential claim: arms (including the raw-telemetry-dump control), dataset options, metrics, and the decision gate.
- [runtime-dependence-and-raw-baseline.md](runtime-dependence-and-raw-baseline.md) — **fair-test sharpening (2026-05-29)**: the runtime-dependence bug taxonomy (R0 repo-logic … R3 cross-tier), a steelmanned **agentic-raw** baseline (B′), and a per-class decision gate so A1 decisively answers "does the bundle beat raw context?"
- [bundle-value-phase0-runbook.md](bundle-value-phase0-runbook.md) — concrete first-pass runbook: task mix, arms, artifact contract, agent-run protocol, scoring, analysis, and continue/kill thresholds.
- [phase0-telemetry-overlay-contract.md](phase0-telemetry-overlay-contract.md) — deterministic telemetry-overlay artifact contract: provenance labels, no-cheat rules, normalized rows, raw-vs-bundle evidence parity, redaction, pass/fail gates.

## Seed corpus and task sources
- [bundle-value-seed-corpus.md](bundle-value-seed-corpus.md) — seed-corpus selection: executable SWE-style task sources, eligibility gates, telemetry-overlay requirements, and manifest shape before Phase 0.
- [a1-task-source-freeze-check.md](a1-task-source-freeze-check.md) — task-source freshness/pinning: Hugging Face dataset SHAs, row/split counts, field quarantine, and manifest requirements (SWE-bench-Live, SWE-rebench).
- [a1-source-drift-and-leakage-recheck.md](a1-source-drift-and-leakage-recheck.md) — source-governance recheck: HF source roles, preview-truncation limits, Python-only classification, full selected-row hashing, and exclusion of trajectory/leaderboard/result datasets.
- [a1-huggingface-row-hash-procedure.md](a1-huggingface-row-hash-procedure.md) — concrete HF procedure: pinned revision metadata, stable row identity, `load_dataset`/pinned-file retrieval, deterministic JSON canonicalization, field-policy hashes, pass/fail rules.

## Result ledger and external reference
- [a1-eval-result-ledger-and-model-refresh.md](a1-eval-result-ledger-and-model-refresh.md) — result-ledger and refresh policy: public run manifests, model snapshots, contamination tiers, per-arm result rows, claim levels, and expiry/rerun triggers.
- [datadog-bits-ai-eval-loop.md](datadog-bits-ai-eval-loop.md) — Datadog Bits AI SRE / Dev Agent eval-platform lessons (world snapshots, noise manifests, segmentation, score history, model-refresh) — validates methodology, not Parallax bundle value.

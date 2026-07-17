# A6 claim status recheck — synthetic canaries vs agent-visible gate (pass 120 + pass 166)

<!-- markdownlint-disable MD013 -->

**Research date:** 2026-07-17 (pass 120); **pass 166 recheck 2026-07-18**  
**Pass:** 120 + **166** (deep-research-parallax indefinite program)  
**Claim under test:** "Redaction can be made trustworthy enough to expose evidence to agents and third-party models" (assumption A6).  
**Verdict this pass:** **Split claim** — engine + public-safe unit canaries support partial **`synthetic_canary_pass`**; full **`agent_visible_mixed_pass`** / third-party-model safety wording remains **open**. Do not collapse to a single `not_measured` *or* a single "A6 closed."

## Why this pass

[Plan 111](2026-07-plan-111-a6/README.md) marks the redaction pipeline **DONE** with
claim level "measured (public-safe canaries)." The research note
[capture/redaction.md](../capture/redaction.md) still banners **`not_measured`** for the
broader red-team ledger. That tension risks either **over-claiming** (marketing "AI-safe
redaction") or **under-claiming** (ignoring shipped detectors). Pass 120 applies the
note's own claim-level table honestly.

## Evidence class

| Layer | Status | Evidence |
| --- | --- | --- |
| Runtime engine | **Shipped** | `crates/parallax-redaction` — `redact` / `project_text` / `sanitize_text`, `detectors-v1`, `redaction-lite-v3` |
| Default-deny + fail-closed | **Shipped (code)** | Unknown fields drop; detector panic → strip; not on ingest hot path |
| Public-safe unit canaries | **Pass (harness)** | Plan 111 table (ghp_/sk_live_/DSN/private key/Basic/generic/password); test `a6_public_safe_canaries_are_not_projected_raw_by_detectors` |
| Formal `redaction-red-team-results.md` | **Absent** | No dated multi-surface result ledger under the A6 contract in this pass |
| Multi-encoding / hostile encodings | **Not ledgered** | Spec in redaction.md; not a published matrix run |
| MCP `structuredContent` + resources projection audit | **Not ledgered** as A6 gate pass | MCP code exists; A6 claim levels require explicit projection audit rows |
| Offline multi-scanner matrix (Gitleaks/Betterleaks/TruffleHog/…) | **Optional / not run this pass** | Comparators re-pinned only |
| Retroactive purge (Tempo-style) | **Out of scope** | Explicit residual in plan 111 + redaction.md |

## Offline comparator re-pin (2026-07-17; **pass 166** reconfirm)

GitHub releases API (pass 166):

| Tool | Latest pin | vs pass 105/120 |
| --- | --- | --- |
| Gitleaks | **v8.30.1** (2026-03-21) | unchanged |
| Betterleaks | **v1.6.1** (2026-06-30) | unchanged |
| TruffleHog | **v3.95.9** (2026-07-09) | unchanged |
| detect-secrets | **v1.5.0** (2024-05-06) | unchanged |
| Presidio | **2.2.363** (2026-06-28) | not re-polled pass 166 |

Still **offline validators only** — never runtime deps on the tiny tier.

### Pass 166 addendum

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **`a6_public_safe_canaries_are_not_projected_raw_by_detectors` → ok** |
| Formal multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |
| Claim split | **Unchanged** — synthetic partial ≠ mixed agent-visible gate |

### Pass 195 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| Gitleaks / TruffleHog pins | Still **v8.30.1** / **v3.95.9** |
| Multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 219 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| Multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 226 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 240 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| Multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 247 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** — re-ran `cargo test -p parallax-redaction a6_public_safe` → `a6_public_safe_canaries_are_not_projected_raw_by_detectors` **passed** |
| Related evidence path | `cargo test -p parallax-evidence redaction` → `sentry_event_cannot_bypass_canonical_bundle_redaction` **passed** |
| Multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |
| Claim split | **Unchanged** — synthetic canary partial ≠ mixed agent-visible gate |

### Pass 267 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** — re-ran `cargo test -p parallax-redaction a6_public_safe` |
| Multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 277 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran with A1 golden same pass) |
| Multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 286 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| Multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 294 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| Multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 300 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| Multi-surface red-team ledger | **Still absent** |
| `agent_visible_mixed_pass` | **Still open** |

## Claim levels applied

From [redaction.md claim table](../capture/redaction.md):

| Level | Met? | Notes |
| --- | --- | --- |
| `not_measured` | **Too coarse alone** | Engine + unit canaries exist — pure "design only" wording is **stale** |
| `synthetic_canary_pass` | **Partial yes** | Public-safe backend-shaped strings via unit tests / plan 111; **not** full backend/Sentry/OTLP fixture corpus as a single published run artifact |
| `cli_ci_bundle_pass` … `structured_provider_projection_pass` | **Not claimed** | No formal ledger rows this pass |
| `agent_visible_mixed_pass` | **No** | Required for "agent-visible bundles are red-team tested for configured surfaces" |
| Plan 111 "measured" | **Reconcile as** | = detector unit + public-safe canaries (**`synthetic_canary_pass` partial**), **not** full A6 mixed gate |

### Pass 306 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** — re-ran `cargo test -p parallax-redaction a6_public_safe` → `a6_public_safe_canaries_are_not_projected_raw_by_detectors` **passed** |
| `agent_visible_mixed_pass` | **Still open** (no mixed agent-visible red-team ledger published) |
| Claim split | **Unchanged** — synthetic canary partial ≠ mixed agent-visible gate |

### Pass 315 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran `a6_public_safe`) |
| `agent_visible_mixed_pass` | **Still open** |
| Claim split | synthetic canary partial ≠ mixed agent-visible gate |

### Pass 323 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 328 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 334 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 339 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 343 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 346 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 351 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 355 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 358 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 361 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 364 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 367 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 370 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 373 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 376 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 379 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Pass 382 addendum (2026-07-18)

| Check | Result |
| --- | --- |
| Public-safe canary unit test | **ok** (re-ran) |
| `agent_visible_mixed_pass` | **Still open** |

### Allowed wording

- "Parallax ships a default-deny Rust redaction engine and public-safe canary unit tests."
- "A6 synthetic detector canaries for common token shapes pass in CI (plan 111)."
- **Not:** "AI-safe redaction" / "safe for third-party models on all agent projections" without naming surfaces and a dated red-team results file.

### Forbidden wording

- "A6 closed" / "redaction proven for agents" from plan 111 DONE alone.
- "Redaction is only designed / not implemented" (engine is shipped).

## Falsification / upgrade path

1. Publish `redaction-red-team-results.md` (or equivalent) with run_id, policy versions, projection list (canonical JSON, Markdown, MCP structuredContent, resources if claimed).
2. Zero seeded-canary leaks across claimed projections + source-field isolation + usefulness gate.
3. Optional: offline multi-scanner delta table at pinned versions above.
4. Only then raise claim level surface-by-surface toward `agent_visible_mixed_pass`.

**What would falsify partial synthetic pass:** public-safe canary samples leak raw through `project_text` / assemble path in CI (plan 111 verify filter).

## Uncertainty

- Did not re-run `cargo nextest` redaction filter in this desk pass (path/test presence checked; full suite is CI-owned).
- Detector count/coverage vs GitHub provider pattern corpus is **incomplete by design** (residual risk accepted in plan 111).
- Betterleaks remains the active pattern-evolution comparator; Gitleaks is stable/security-patch mode.

## Parallax goal fit

Agent-context product **requires** trustworthy projection safety. Shipping detectors is necessary and now real; treating unit canaries as full A6 still **overstates** trust for third-party model exposure. Keep agenda item A6 **open** for the mixed agent-visible gate; use split claim levels in product language.

## Sources checked

- Repo: `docs/research/capture/redaction.md`, `docs/research/validation/2026-07-plan-111-a6/README.md`, `crates/parallax-redaction/src/lib.rs`.
- Primary: GitHub releases API for Gitleaks / Betterleaks / TruffleHog / detect-secrets / Presidio; PyPI `presidio-analyzer` / `detect-secrets`.

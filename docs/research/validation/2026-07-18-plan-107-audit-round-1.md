# Plan 107 audit round 1 — findings and dispositions (2026-07-17/18)

Two independent auditors ran from clean detached checkouts at candidate
`d76de3f9` (Auditor A: architecture/contracts/source; Auditor B: CI/release/
security/lifecycle). Every finding was resolved in source; the Step 4 rerun
froze a new candidate C0 `0e5392a2`.

## Auditor B (CI/release/security/lifecycle) — FINDINGS(2) + 2 informational

| # | Sev | Location | Defect | Disposition |
|---|-----|----------|--------|-------------|
| B-1 | MEDIUM | `.github/workflows/ci.yml` + `release.yml` | `github.event_name == 'workflow_call'` is dead code inside a called reusable workflow (caller's event propagates), so release-tag CI classified only the tagged commit's paths and could skip gates while `ci-required` stayed green via skipped-as-success | Fixed `13f81c6f`: explicit `force_all` workflow_call input passed by release.yml plus a tag-ref guard; classify step skipped in forced runs |
| B-2 | LOW | `scheduled-measurement.yml` | `\|\| true` masked `cargo bench` failure; surfaced only indirectly as MISSING baseline | Fixed `91c4898a`: explicit `shell: bash` (pipefail), grep moved after the run |
| B-3 | INFO | closure-final job | read-only `GH_READONLY_TOKEN` present in the "no secrets" lane (mise rate limits only) | Documented exception comment in the job (`13f81c6f`) |
| B-4 | INFO | `ui/.cta.json` | historical `"chosenAddOns": ["eslint"]` scaffolding record | Inert — no invocation path; accepted |

All other areas PASS with reproduced evidence: 19-job `ci-required` topology
complete; closure-final verifier fail-closed with real tamper fixtures;
zero rustls in the lockfile (deny + xtask double enforcement); Bun-only file
policy; exact two-entry Oxc prestable allowlist; release authorization and
signing chain (cosign/syft/SLSA); bench ratchets cannot rewrite baselines;
plans/ bijection with blockers 089/114 freshly reproduced live (crates.io
0.18.0 + PR #58 open; sole rolling `preview` release).

## Auditor A (architecture/contracts/source) — FINDINGS(13)

| # | Sev | Location | Defect | Disposition |
|---|-----|----------|--------|-------------|
| A-1 | HIGH | `crates/parallax-analysis` | `cargo fmt --check` red at candidate (peer commit `4102befe`) | Fixed on main by loop maintenance before C0 |
| A-2 | HIGH | `crates/parallax-metadata/src/turso/tests.rs:652` | clippy `too_many_lines` 107/100 under `-D warnings` | Fixed on main before C0 (workspace clippy clean) |
| A-3 | HIGH | `deny.toml` | `borrow-or-share v0.2.4` (MIT-0 via fluent-uri) rejected: MIT-0 missing from allowlist | Fixed `e29c9980` |
| A-4 | HIGH | 24 `*.generated.ts` | GraphQL codegen drift vs schema at same lockfile | Fixed `1762c84d` (regenerated; byte-stable) |
| A-5 | MEDIUM | `ui/test-matrix.json` | e2e spec missing matrix owner; inventory ratchet stale | Fixed by matrix reconciliation before C0 (policy zero) |
| A-6/A-7 | MEDIUM | parallax-metadata / parallax-analysis | structural growth breaches + missing exact ratchet rows | Reconciled before C0 (policy zero) |
| A-8 | MEDIUM | `ratchet.toml` | ~24 stale rows scoped to `.claude/worktrees/**` (writer accepted paths outside the repo tree) | Rows removed before C0; zero worktree-scoped rows remain |
| A-9 | MEDIUM | ui deps | 7 codegen packages flagged unused; elkjs EPL-2.0 and remedial `(MIT OR Apache-2.0)` failed license coverage | Fixed `b29e875c`: codegen plugins listed as reviewed-non-ast (config-discovered), SPDX OR disjunction accepted, EPL-2.0 allowed for unmodified binary deps per ASF Category B |
| A-10 | MEDIUM | turso/test_reporting.rs | anyhow-edge count 32 vs exact ceiling 31 | Fixed `91c4898a` |
| A-11 | MEDIUM | `crates/parallax-evidence/Cargo.toml` | unused `regex` dependency (cargo shear) | Fixed `e29c9980` |
| A-12 | LOW/ENV | `crates/parallax-xtask/src/dependencies.rs` | cross-native-tls check host-dependent (Apple hosts never activate openssl-src) | Fixed `e29c9980`: tree pinned to `x86_64-unknown-linux-gnu` |
| A-13 | LOW | ui vitest | nondeterminism under CPU contention (4 fails racing cargo lane; 539/539 isolated) | Run-order note recorded for shared-verification runs; not a product defect |

Architecture areas PASS at first candidate and unchanged: tier direction,
facades, native-table rule (only justified extension tables), native TLS,
zero-copy hot path (`bytes::Bytes` refcount clones), GraphQL schema↔resolver
alignment, suppression samples tight, five documentation claims verified in
source. Process note: the first audit worktree was deleted externally
mid-run; interrupted lanes were re-executed in a fresh clean detached
checkout at the same SHA.

## Step 4 rerun at C0 `0e5392a2`

- Auditor B: **CLEAN** — B-1/B-2/B-3 verified closed with no new holes
  (release-tag scenario re-reasoned; actionlint clean; dry-run verifier
  passes; blockers 089/114 re-reproduced fresh; xtask suite 95/95).
- Auditor A: rerun in flight at the same C0.

Full shared baseline verified green in the main tree at the same head before
freezing C0: fmt clean, workspace clippy zero warnings, nextest 645/645
(+9 live-engine skips), policy zero, ui typecheck/lint clean, vitest 539/539,
`cargo deny check licenses` ok, `cargo shear` clean.

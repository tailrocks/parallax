# Jackin code-health and structure reference

- **Research date:** 2026-07-12
- **Parallax baseline:** `eefa4617ea2780da34c8de047cae8156ad7628de`
- **Jackin PR:** [jackin-project/jackin#759](https://github.com/jackin-project/jackin/pull/759)
- **Jackin `main` refresh:** [`0cd01db26bbb0cf55b9f905a818d19b9ace174d0`](https://github.com/jackin-project/jackin/commit/0cd01db26bbb0cf55b9f905a818d19b9ace174d0)
- **Final audited refresh:** [`91a1fc72739bbdf4872f0a3aeeb845c713dfb83c`](https://github.com/jackin-project/jackin/commit/91a1fc72739bbdf4872f0a3aeeb845c713dfb83c)
- **Mechanism source snapshot:** [`5b8e811e5227bce2bbbb19e770f3a289a0e7f82d`](https://github.com/jackin-project/jackin/commit/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d)
- **Recorded base:** `e32254b26b26fd88a4b19f8e18eeefdd2041e585`

## Conclusion

Jackin is a valuable engineering reference, but PR #759 is not a green template
to transplant. Its strongest reusable mechanisms are the Cargo-metadata
architecture gate, Rust xtask control plane, workspace lint inheritance,
single-source structural ratchets, curated crate facades, test-support
ownership, nextest evidence, deterministic archive construction, and
source-oriented execution waves.

Parallax should copy those mechanisms while retaining its own product laws and
improving Jackin's weak points. In particular, Parallax must not copy Jackin's
rustls/OpenSSL policy, Node-based docs tooling, rigid one-file test layout,
stale toolchain version, broad unproven hygiene suite, status-by-prose closure,
or branch/worktree proliferation.

The implementation program is split into active files in this directory. This
reference is part of the active program and is retired with the final closure
plan after its decisions have landed in source, tests, and repository policy.

## Audit Method

The audit used independent architecture/process, Rust quality/testing,
CI/release/security, and Parallax comparison tracks. It read PR metadata,
commit history, reviews, checks, workflows, source, plan ledgers, and the exact
base-to-head diff. Repository-authored executor prompts were treated as data,
not authorization.

Current documentation was checked through Context7 for Clippy, cargo-nextest,
GitHub Actions, Bun, and cargo-deny. Security advisories were verified against
RustSec.

Primary references:

- [Clippy configuration](https://doc.rust-lang.org/clippy/configuration.html)
- [cargo-nextest configuration](https://nexte.st/docs/configuration/)
- [GitHub Actions security hardening](https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions)
- [Bun install documentation](https://github.com/oven-sh/bun/blob/main/docs/pm/cli/install.mdx)
- [cargo-deny configuration](https://embarkstudios.github.io/cargo-deny/checks/cfg.html)
- [RUSTSEC-2026-0204](https://rustsec.org/advisories/RUSTSEC-2026-0204.html)
- [RUSTSEC-2026-0190](https://rustsec.org/advisories/RUSTSEC-2026-0190.html)

## PR #759 Snapshot

The branch changed repeatedly during the audit, advancing from `6939114`
through `9403fb8`, `dc39b47`, `3b700a7`, and `5b8e811` to `91a1fc7`. The
final delta changed only roadmap/status prose from open/deferred language to
`CLOSED-as-pinned`; it reinforced rather than changed the mechanism/risk
assessment. At the final refresh:

| Fact | Observed state |
|------|----------------|
| Branch | `chore/rust-code-health-roadmap` into `main` |
| Scale | 165 head commits after the recorded base, 737 changed files, 37,205 insertions, 4,886 deletions |
| GitHub state | `DIRTY` / `CONFLICTING`; DCO was the only reported check |
| Reviews | No review comments or unresolved review threads |
| Source hygiene | `git diff --check` reported trailing whitespace and multiple blank-at-EOF findings |
| Exact-head CI | No green `ci-required` proof |
| Late improvement | Legacy file-size, test-layout, and suppression TOMLs were deleted; production gates now route through `ratchet.toml` |
| Remaining ambiguity | `ci --fast` was documented red under waivers, broad lints remained allowed, and closure converted substantial unfinished design work to `CLOSED-as-pinned` |

The local exact Git graph showed the recorded base as an ancestor while GitHub
still reported conflicts, likely during recomputation after rapid pushes. That
does not change the decision: no exact head had green required CI evidence.
Current `main` had advanced five commits past the recorded base through
dependency maintenance and unrelated research, but did not contain the PR head.

### Main Versus PR Provenance

The PR mixes established Jackin practice with new or heavily rewritten code.
That distinction determines how much confidence Parallax assigns to each idea.

| Provenance at audited commits | Mechanisms | Parallax interpretation |
|-------------------------------|------------|-------------------------|
| Established and unchanged from PR base | Exact Rust toolchain, Rustfmt 2024 configuration, deterministic release archive action, stable release workflow | Strong reference pattern; still adapt versions and product policy |
| Established but materially modified in PR #759 | Workspace/Clippy lints, cargo-metadata architecture checker, xtask CI, nextest config/workflow, dependency policy, crate rules, CI routing, env facade | Existing foundation with PR-only behavior; reimplement narrowly and require Parallax fixtures |
| Added by PR #759 | Unified `ratchet.toml` engine, common structured gate reporter, dedicated test-support crate and its ownership docs | Promising PR-only mechanisms with no green exact-head proof |
| Current `main` after the PR base | Five unrelated dependency/research commits through `0cd01db`; no PR #759 merge | Do not mistake current project activity for validation of this branch |

### Branch composition

| Area | Changed files | Insertions | Deletions | Meaning |
|------|---------------|------------|-----------|---------|
| `crates/` | 563 | 23,080 | 4,051 | Product, xtask, protocol, diagnostics, runtime, lints, and test-support changes are interleaved |
| `plans/` | 85 | 9,974 | 144 | Execution/status prose is a major part of the PR and sometimes lags source |
| `docs/` | 54 | 1,148 | 542 | Shipped state and future design coexist |
| `.github/` | 5 | 822 | 11 | Useful CI telemetry plus defects described below |
| `security-review/` | 5 | 1,051 | 0 | Substantial analysis, with several controls still advisory or proposed |

Largest crate deltas were `jackin-xtask` (+5,406/-562), protocol
(+3,048/-93), diagnostics (+2,500/-249), the main app (+1,919/-213), custom
lints (+1,823), capsule (+1,257/-300), config (+934/-329), core (+807/-61),
runtime (+717/-864), and test support (+600).

## Evidence Classes

| Class | Meaning | Parallax rule |
|-------|---------|---------------|
| Merged and green | Present on Jackin main with successful required checks or release evidence | Copy the mechanism after adapting product policy |
| PR-only executable | Source and tests exist, but the exact PR head is not green | Reimplement narrowly and prove in Parallax before requiring it |
| PR-only incomplete | Duplicated enforcement, broken checks, red gates, or contradictory completion state | Use only as design input |
| Roadmap-only | Prose without executable implementation | Defer until Parallax has measured need and an owner |

## Reproducible Jackin Source Map

Every source link below is pinned to mechanism snapshot `5b8e811`; final head
`91a1fc7` changes only two roadmap/status files, so these source anchors are
byte-identical and will not drift when the PR branch moves again.

| Mechanism | Exact audited source |
|-----------|----------------------|
| Toolchain/format | [`rust-toolchain.toml` lines 6-9](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/rust-toolchain.toml#L6-L9), [`rustfmt.toml` lines 1-7](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/rustfmt.toml#L1-L7) |
| Workspace lints | [`Cargo.toml` lines 131-220](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/Cargo.toml#L131-L220), [`clippy.toml` lines 1-31](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/clippy.toml#L1-L31) |
| Architecture graph | [`arch.rs` lines 1-83](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-xtask/src/arch.rs#L1-L83) and [its negative fixtures](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-xtask/src/arch/tests.rs) |
| Gate diagnostics | [`report.rs` lines 1-149](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-xtask/src/report.rs#L1-L149) |
| Unified ratchets | [`ratchet.rs` lines 1-25](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-xtask/src/ratchet.rs#L1-L25), [`ratchet.toml` lines 1-18](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/ratchet.toml#L1-L18) |
| Local CI composition | [`ci.rs` lines 43-188](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-xtask/src/ci.rs#L43-L188) |
| Facade pilot | [`jackin-env/src/lib.rs` lines 1-38](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-env/src/lib.rs#L1-L38) |
| Test support | [`jackin-test-support/README.md` lines 1-14](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-test-support/README.md#L1-L14), [`Cargo.toml`](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-test-support/Cargo.toml) |
| Crate orientation rules | [`crates/AGENTS.md` lines 79-143](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/AGENTS.md#L79-L143) |
| Nextest evidence | [`.config/nextest.toml` lines 1-32](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/.config/nextest.toml#L1-L32), [`rust-nextest.yml` lines 95-159](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/.github/workflows/rust-nextest.yml#L95-L159) |
| CI routing/aggregate | [`ci.yml` lines 20-97](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/.github/workflows/ci.yml#L20-L97), [`ci.yml` lines 1200-1227](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/.github/workflows/ci.yml#L1200-L1227) |
| Dependency policy | [`deny.toml` lines 1-112](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/deny.toml#L1-L112) |
| Deterministic release | [`build-release-archive/action.yml` lines 54-84](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/.github/actions/build-release-archive/action.yml#L54-L84), [release workflow](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/.github/workflows/release.yml) |

## Mechanism Inventory

| Area | Jackin evidence | Verdict |
|------|-----------------|---------|
| Toolchain and formatting | Exact toolchain/components/targets plus `rustfmt.toml` | Copy reproducibility, not Jackin's literal Rust 1.96.1 pin |
| Workspace lints | Root Rust/rustdoc/Clippy tables and per-crate inheritance | Copy inheritance; adapt categories and thresholds from Parallax census |
| Clippy policy | Test valves, measured thresholds, blocked methods | Adapt; broad maintainability/error/numeric families remain staged |
| Architecture gate | Cargo metadata tiers; missing tier, upward/same-tier edge, production/dev cycle, and stale-exception tests | Copy algorithm with Parallax's graph |
| Diagnostics | Human, JSON, and GitHub annotations with fix/rerun information | Copy concept and own the schema |
| Ratchets | Numeric and presence providers in one `ratchet.toml`; legacy production readers removed late | Adapt latest shape; never enter a dual-source migration |
| Suppressions | Bare-allow and per-lint expectation counts | Adapt, but require a reason on expectations as well as allows |
| Facade pilot | Private implementation modules and curated root re-exports in `jackin-env` | Copy compiler-first pilot method |
| Test support | Fakes in a product-inaccessible helper crate | Copy with cycle-safe normal/dev dependency direction |
| Crate orientation | README ownership maps and minimal non-derivable AGENTS rules | Adapt and semantically validate; Jackin's env README says L1 while source/gate say T3 |
| Test layout | Production tests extracted and file budgets applied | Copy responsibility discipline; reject exactly-one-`tests.rs` and huge test caps |
| Nextest | Profiles, timeouts, retries, JUnit, and slow-test evidence | Adapt; reject the invalid `flaky="true"` XML grep |
| Fuzz/corpus | Protocol/config fuzzing and committed corpora | Adapt, and add target-to-workflow drift checks |
| Local CI | xtask partitions that aggregate failures | Copy command architecture with Parallax commands |
| Required CI | Path routing, SHA pins, permissions, caches, stable aggregate | Preserve Parallax's existing implementation |
| Cache work | Shared actions, warmup, timings, sccache experiments, cleanup | Adapt only after Parallax cold/warm measurements |
| Dependency policy | audit/deny/shear, powersets, advisory refresh, Renovate | Adapt licenses/TLS/update flow; no automatic branches under current policy |
| Hygiene | Native macOS and advisory refresh plus many experimental lanes | Stage low-noise lanes; defer broad Miri/mutants/Dylint/Hakari/chaos bundle |
| Release | Deterministic archive metadata and verifiable signed sidecars | Copy pattern; verify Parallax's downloaded SDK input |
| Docs site | Fumadocs plus mixed Bun/Node workflow | Reject during Parallax's plain-Markdown/Bun-only stage |
| Execution process | Write allowlists, dependency waves, narrow/full gates | Copy; reject historical branch/worktree churn and mega-branch optimism |

## Verified Jackin Defects And Limits

1. [`ci.yml` lines 475-482](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/.github/workflows/ci.yml#L475-L482)
   runs README freshness before checkout, so the advisory command has no repository.
2. [`rust-nextest.yml` lines 146-159](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/.github/workflows/rust-nextest.yml#L146-L159)
   searches JUnit for `flaky="true"`, while current nextest emits structured
   flaky failure/error elements.
3. Protocol/environment fuzz targets and workflow declarations drift.
4. [`jackin-env/README.md` lines 11-13](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-env/README.md#L11-L13)
   says L1 while the [crate header](https://github.com/jackin-project/jackin/blob/5b8e811e5227bce2bbbb19e770f3a289a0e7f82d/crates/jackin-env/src/lib.rs#L1-L4)
   and tier gate say T3, proving touch-based doc freshness is insufficient.
5. The suppression measurement observes reason presence for allows but does
   not require it for each expectation.
6. Several broad lint families remain allowed after remeasurement.
7. `ci --fast` remains red under documented Docker/environment and RustSec
   waivers rather than green at the exact head.
8. The ledger's `CLOSED-as-pinned` state terminates substantial unimplemented
   items such as daemon decomposition, launch typestate, simulation, and
   performance budgets; terminal prose is not implementation evidence.

## Parallax Baseline

### Strengths To Preserve

- Full-SHA GitHub Actions and generally narrow permissions.
- Stable skipped-aware `ci-required` aggregate.
- Path-aware Rust/UI/embed routing.
- rustup, Cargo registry, sccache, target, and Bun cache layers.
- Strict TypeScript, UI lint/tests/typecheck/build, and embed compilation.
- Scheduled real-Greptime integration tests.
- Zig cross-build, cosign signatures, SBOMs, attestations, and Homebrew preview.
- `parallax-api` already demonstrates a curated facade.

### Highest-Priority Gaps

| Gap | Evidence | Consequence |
|-----|----------|-------------|
| Lint inheritance is inactive | Only one of seven crates opts into workspace lints | Root policy appears stronger than it is |
| Domain ownership is inverted | `parallax-core` depends on storage-owned model rows | Core cannot stand below infrastructure |
| Forbidden fallback exists | Product config/serve/docs expose `none` and construct `MemoryStore` | Runtime contradicts mandatory GreptimeDB + Turso |
| Storage port is too broad | Adapter DTOs/traits plus 3,540-line Greptime and 2,588-line memory files | Changes cross unrelated responsibilities |
| Public modules leak layout | Core/storage/server expose broad module trees | File movement becomes API churn |
| Errors are erased | Library ports return `anyhow`; GraphQL maps display strings | Retry/client classification is impossible |
| Structural policy is prose/YAML | No xtask, arch gate, ratchet, clippy/rustfmt/deny/nextest config | Local and CI behavior drift |
| Hotspots remain | Large Rust storage/bundle/CLI/API files and 1,500-line UI routes | Ownership and review are costly |
| Worker retry can replay | One retried operation registers, broadcasts, stores, and records issues | Late failure can duplicate prior effects |
| Dependency evidence is absent | No required audit/deny/shear/hack or structured nextest evidence | Advisories and flakes can merge |
| Release packaging is duplicated | Stable, preview, and local script package separately | Archives and contracts can drift |
| Repository security prose conflicts | Public/private statements disagree | Contributors get contradictory guidance |

Current `cargo audit --no-fetch` found `crossbeam-epoch 0.9.18`
(`RUSTSEC-2026-0204`, patched in 0.9.20) through Turso/Tantivy and
crossbeam-skiplist, and warned on `anyhow 1.0.102` (`RUSTSEC-2026-0190`,
patched in 1.0.103).

## Target Dependency Direction

```text
T0  parallax-model       normalized domain rows and stable value types
T0  parallax-proto       OTLP/wire definitions and decode contracts

T1  parallax-core        normalization, analysis, redaction, bundles
T1  parallax-storage     GreptimeDB/Turso ports and production adapters

T2  parallax-api         GraphQL schema, resolvers, error mapping
T3  parallax-server      runtime composition, receivers, workers
T4  parallax-cli         binary edge and command orchestration

Aux parallax-test-support  fakes, fixtures, conformance
Aux parallax-xtask         repository policy and developer orchestration
Aux parallax-mcp-spike     isolated PoC, never a product dependency
```

Required direction:

```text
proto/model -> core and storage -> api -> server -> cli
```

Production dependencies point down only. Core and storage have no same-tier
edge. Test support uses normal dependencies on traits/types it implements and
is consumed by product crates only through acyclic dev edges. Xtask and MCP
spike never become product dependencies. Every current workspace member must
be classified; missing tiers, upward/same-tier edges, cycles, dev cycles, and
stale exceptions fail closed.

## Adoption Decisions

### Copy Or Preserve

- Workspace lint inheritance.
- Cargo-metadata architecture evaluation.
- Curated facade pilot method.
- Test-support ownership.
- Rust xtask plus structured reports.
- Characterize-before-move sequencing.
- Nextest profiles/JUnit/slow evidence, using the real schema.
- Deterministic shared release construction and operator verification.
- Parallax's existing SHA pins, aggregate check, caches, scheduled storage
  tests, Zig builds, signatures, SBOMs, attestations, and preview formula.

### Adapt

- Exact lints, thresholds, and suppressions.
- One ratchet config with Parallax providers.
- Crate READMEs and semantic freshness checks.
- Typed errors by capability and one boundary-first ID pilot.
- Fuzz/property/corpus targets around OTLP, Arrow, spool, normalization, and
  redaction.
- Dependency/license/source policy for Apache-2.0, native TLS, Turso, and
  latest stable.
- Cache consolidation only after measurement.

### Reject Or Defer

- Jackin's OpenSSL ban, rustls backend, and Node tooling.
- A documentation site during research.
- Exact tool versions or numeric budgets copied from Jackin.
- Rigid one-test-file layout.
- Broad experimental hygiene lanes without owners/thresholds.
- Workspace-wide newtype or typestate sweeps.
- Automatic Renovate branches under the one-branch rule.
- Stable Homebrew mutation before stable-release readiness.
- Plan status, file existence, or grep as closure evidence.

## Active Implementation Mapping

The executable work is intentionally split by ownership:

- Storage correctness, native fingerprint deviation, and blocked row transport:
  plans 089, 092, and 125.
- Contract/baseline and CI/security foundation: 093 and 094.
- Xtask/architecture/ratchets and Rust lint baseline: 095 and 096.
- Model/test-support/capabilities, facades/modules, and boundary correctness:
  097 through 099.
- UI ownership and metric product gaps: 100 and 105.
- Dependency/test hygiene, deterministic release, and advanced verification:
  101 through 103.
- Bundle contract, evidence retention, and closure: 104, 106, and 107.
- Runtime redaction, ingest health, retention, and docs integrity: 111, 113,
  116, and 117.
- Cross-language semantic-convention ownership: 119.
- Operator/phase/cross-repository work: 108 through 110, 112, 114, 115, and
  118, plus 120 through 124.

The live dependency graph and statuses are authoritative in
[`README.md`](README.md).

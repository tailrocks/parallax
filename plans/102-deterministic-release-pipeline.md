# Plan 102: Prove the deterministic release pipeline externally

> **Executor instructions**: The local packaging/verifier design is in place.
> Do not reopen archive layout, sidecar set, identity, or workflow design.
> Retire this plan only after protections stay open **and** one preview
> produced by an implementation that satisfies `verify_object` on **all four**
> targets verifies end to end.

## Status

- **Priority**: P1
- **Effort**: S–M remaining (Mach-O line-table embed + one preview proof)
- **Risk**: HIGH
- **Depends on**: 094, 096, 101; structural Mach-O line-table fix; post-merge
  preview run
- **Category**: release / packaging / provenance
- **Planned at**: `a1d8bf82`, revised 2026-07-15, blocked-update 2026-07-17
- **Status**: IN PROGRESS — Mach-O line-table embed landed; external
  four-target preview + release-verify still required before retirement

## Completed Contract

Preview, stable, and rehearsal use one deterministic Rust archive producer.
The local verifier binds archive layout, target, version, executable identity,
line tables, checksum, CycloneDX SBOM, Sigstore bundle, and GitHub provenance;
its independently malformed fixtures fail closed. Workflows validate identity
before builds, use digest-verified SDK inputs, preserve least privilege, and
cannot manually publish stable artifacts before readiness. Parallax contains no
tap write credential or cross-repository checkout. The tap's own scheduled and
manual workflow downloads all four preview targets and sidecars, verifies them,
and updates only `parallax-preview.rb` with repository-local authority.

Durable operator and verification instructions live in
[`docs/guide/releases.md`](../docs/guide/releases.md). Implementation and live
configuration evidence lives in
[`docs/research/validation/2026-07-13-plan-102-release-baseline.md`](../docs/research/validation/2026-07-13-plan-102-release-baseline.md).

## Remaining Work

1. ~~Operator enables stable readiness~~ **DONE 2026-07-17**:
   `STABLE_RELEASE_ENABLED=true`, reviewer-protected `stable-release`
   environment, active `refs/tags/v*` ruleset (`stable tag protection`).
2. ~~**Fix Mach-O release binaries so `verify_object` accepts them**~~
   **DONE 2026-07-17** — post-link dSYM→`__DWARF` embed in
   `release-package` (`macho_dwarf.rs`), Apple headerpad + packed dSYM via
   `.cargo/config.toml`, Apple CI legs on `macos-latest`. Local proof:
   `cargo test -p parallax-xtask --lib release::` (includes aarch64-apple
   embed + `verify_object`). Evidence:
   [`docs/research/validation/2026-07-17-plan-102-macho-line-tables.md`](../docs/research/validation/2026-07-17-plan-102-macho-line-tables.md).
3. After that fix reaches `main` and CI is green, publish one complete
   four-target preview asset set from the current implementation SHA.
4. Download that set and run `cargo xtask release-verify` with its exact source
   SHA/ref for every target. Confirm the tap pull workflow accepts the same set
   and updates only the rolling preview formula.
5. Preserve sanitized protection and preview-verification evidence, then delete
   this plan and its index row.

## Fresh Blocker Evidence (2026-07-17)

> **Update 2026-07-17 later**: Mach-O embed structural fix landed (see remaining work item 2). Protections remain OK; external four-target preview is now the residual blocker.

## Fresh Blocker Evidence (2026-07-17) — pre-fix snapshot

Protections (read-back OK):

- `STABLE_RELEASE_ENABLED=true`
- environment `stable-release` with required reviewer `donbeave`
- ruleset `stable tag protection` on `refs/tags/v*`
  (creation/update/deletion/non-fast-forward)

Preview proof still missing:

- Rolling `preview` release + formula still target
  `4e8edfa5f92cd8060dfdd46dccb82a0fa26613f8` (pre-verifier).
- Post-unification preview builds fail Package on Apple:
  `error: release binary is missing line tables` (e.g. run `29548131177` on
  `ba85f86`).
- Local + Linux-container zigbuild of a minimal release binary for
  `aarch64-apple-darwin` produces **no** `__debug_line` / `__DWARF` in the
  final executable; `split-debuginfo=packed` only yields a `.dSYM` companion.
- Current `release-package` also rejects the already-published
  `4e8edfa` aarch64-apple archive for the same reason.

Full write-up:
[`docs/research/validation/2026-07-13-plan-102-release-baseline.md`](../docs/research/validation/2026-07-13-plan-102-release-baseline.md)
section **External proof attempt (2026-07-17T08:30Z UTC) — BLOCKED**.

## STOP Conditions

- Do not create environments, repository variables, or tag rulesets without
  operator authorization.
- Do not publish a rehearsal or treat an older preview as proof of the current
  byte-producing implementation.
- Do not restore a Parallax-owned tap credential or cross-repository write path.
- Do not weaken `verify_object` to pass empty Mach-O DWARF without a real
  in-binary line-table / symbolication surface.

## Remove When

Delete this plan and index row after the protection trigger remains configured
and a preview from an implementation that passes atomic verification on all
four targets is accepted by the tap pull workflow.

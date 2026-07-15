# Plan 102: Prove the deterministic release pipeline externally

> **Executor instructions**: The local implementation is complete. Do not
> reopen archive, sidecar, identity, or workflow design. Retire this plan only
> after the repository protections are opened and one preview produced by the
> current implementation verifies end to end.

## Status

- **Priority**: P1
- **Effort**: S remaining
- **Risk**: HIGH
- **Depends on**: 094, 096, 101; external repository configuration and a
  post-merge preview run
- **Category**: release / packaging / provenance
- **Planned at**: `a1d8bf82`, revised 2026-07-15
- **Status**: BLOCKED — stable release authorization is not configured and no
  preview from the completed implementation has been published

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

1. The operator enables stable readiness by setting
   `STABLE_RELEASE_ENABLED=true`, creating a reviewer-protected
   `stable-release` environment, and activating a `refs/tags/v*` ruleset that
   restricts creation, deletion, and non-fast-forward updates.
2. After the completed implementation reaches `main`, allow the preview
   workflow to publish one complete four-target asset set.
3. Download that set and run `cargo xtask release-verify` with its exact source
   SHA/ref for every target. Confirm the tap pull workflow accepts the same set
   and updates only the rolling preview formula.
4. Preserve sanitized protection and preview-verification evidence, then delete
   this plan and its index row.

## Fresh Blocker Evidence

On 2026-07-15 at branch head `5be1190`:

- `cargo test --locked -p parallax-xtask release` passed all 15 focused tests;
  strict all-target/all-feature xtask clippy and Actionlint for both release
  workflows passed.
- GitHub returned `404` for the `stable-release` environment. The only active
  ruleset is branch-targeted `main protection`; there is no tag ruleset.
- The rolling `preview` release targets `4e8edfa5f92cd8060dfdd46dccb82a0fa26613f8`,
  which predates this branch's finalized verifier and cannot prove it.
- `tailrocks/homebrew-parallax/.github/workflows/update-preview.yml` is present
  and uses the tap repository's own scoped `contents: write` authority after
  independently checking the complete asset set.

## STOP Conditions

- Do not create environments, repository variables, or tag rulesets without
  operator authorization.
- Do not publish a rehearsal or treat an older preview as proof of the current
  byte-producing implementation.
- Do not restore a Parallax-owned tap credential or cross-repository write path.

## Remove When

Delete this plan and index row after the protection trigger is configured and a
preview from the current implementation passes atomic verification and the tap
pull workflow.

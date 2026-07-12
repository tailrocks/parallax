# Plan 102: Unify and verify deterministic release artifacts

> **Executor instructions**: Preserve public archive names/layout, the rolling
> preview formula, Zig cross-builds, signatures, SBOMs, and attestations. Build
> preview, stable, and local rehearsal artifacts through one byte-producing
> implementation and prove tamper failures before switching workflows.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 094, 101
- **Category**: release / packaging / provenance
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

## Why

Parallax already has strong SHA pins, Zig builds, signatures, SBOMs,
attestations, and preview Homebrew automation, but preview, stable, and local
packaging construct archives separately. Tar/gzip metadata is not one proven
contract, release jobs can start independently of complete version validation,
and there is no operator command that verifies the artifact set atomically.

## Scope

- One local composite action or repository-owned implementation for archive
  construction used by every release mode.
- Digest-pinned and verified external build inputs, including the macOS SDK,
  with caches keyed by the verified digest.
- Deterministic ordering, paths, timestamps, ownership, permissions, and gzip
  metadata; isolated target/Zig caches.
- Version/tag/archive/binary coherence before build or publish.
- Protected stable-release environment and `v*` tag authorization evidence.
- Checksum, cosign bundle, CycloneDX SBOM, and GitHub provenance attestation
  tied to exactly the same archive bytes.
- Fully implemented xtask rehearse and verify commands.
- End-to-end preview proof and operator verification documentation.

Out of scope:

- Changing the stable Homebrew formula before stable-release readiness.
- Publishing from a rehearsal or adding a second release path.
- Mutable Action tags, long-lived credentials, or broader workflow permissions.
- Public archive layout changes without an explicit compatibility migration.

## Steps

### Step 1: Freeze the artifact contract

Inventory current preview/stable/script outputs, names, layout, executable
modes, target triples, checksums, SBOMs, signatures, attestations, and formula
expectations. Add fixtures that characterize the current public contract.
Resolve any tag/workspace/binary version contradiction before packaging begins.

Decide the current manual stable-release contradiction explicitly. Before
stable readiness, disable stable publication from arbitrary
`workflow_dispatch` on `main`. When stable readiness is opened, protect `v*`
tag creation/update/deletion through the live repository ruleset and route
publication through a protected stable-release environment with explicit
operator approval. Record sanitized live protection evidence. Require an
existing authorized, validated release tag (or an operator-approved, rehearsed
tag-creation contract) and prove tag, source commit, workspace version, binary
version, archive, and release name coherence before any build/publish job
starts. Keyless signing identity is provenance, not release authorization.

### Step 2: Build identical bytes through one implementation

Extract a repository-owned deterministic archive action/helper shared by
preview, stable, and xtask rehearsal. Normalize file order, relative paths,
mtime from a declared source epoch, uid/gid/uname/gname, permissions, and gzip
header metadata. Keep Zig and per-target caches isolated so cached state cannot
enter the archive. Run twice from clean directories and compare digests.

Pin every downloaded SDK/tool archive to an approved URL plus SHA-256 (or
stronger) digest, verify it before extraction, and key its cache by that digest.
The current macOS SDK download must never be `curl | tar` or trust a mutable URL
without an independently pinned checksum. Add wrong/missing-digest fixtures.

### Step 3: Generate one coherent sidecar set

Produce checksums, cosign bundle/signature, CycloneDX SBOM, and GitHub artifact
attestation from the finalized archive bytes. Use OIDC and least-privilege job
permissions. Record workflow identity/issuer and source commit. Do not sign or
attest an intermediate file that is later recompressed or renamed.

### Step 4: Implement local rehearsal and atomic verification

Implement parser/dispatch-tested `cargo xtask release-rehearse` and
`cargo xtask release-verify`. Rehearsal uses the same packaging implementation
without publishing. Verification checks archive digest/layout/mode, checksum,
cosign identity/issuer, GitHub attestation subject/commit, SBOM parse/content,
target/version coherence, and completeness of the artifact set. No placeholder
or unconditional-success subcommand may land.

### Step 5: Prove workflow behavior

Make release build/publish jobs depend on successful version and policy gates.
Run one preview end to end, verify downloaded assets independently, and update
only `parallax-preview` after every required asset is published and verified.
Stable formula mutation remains disabled until stable readiness is opened.

## Test Plan

- Golden archive layout/mode/name tests per supported target.
- Two clean rehearsals with identical SHA-256 digests.
- Tampered archive, checksum, signature/bundle, attestation, SBOM, missing
  sidecar, wrong target, and version mismatch tests; each must fail closed.
- Wrong/missing external SDK digest and poisoned-cache tests.
- Manual stable-dispatch tests proving pre-stable publication is disabled and
  stable-ready publication is tag/version coherent.
- Authorization tests proving an unprotected/unauthorized `v*` tag cannot reach
  publication and the stable-release environment requires operator approval.
- Actionlint and least-privilege workflow review.
- End-to-end preview asset download and `release-verify` proof.
- Homebrew preview formula update only after verified publication.

## Done Criteria

- [ ] Preview, stable, and rehearsal use one archive implementation.
- [ ] Archive construction is deterministic across two clean identical inputs.
- [ ] Tag, workspace, binary, archive, and release versions agree before build.
- [ ] External SDK/tool inputs are digest-verified before extraction and cache use.
- [ ] Manual stable publication is disabled until explicitly opened and tagged.
- [ ] Stable publication requires protected `v*` tag authorization and approval
  through the protected stable-release environment.
- [ ] Every sidecar identifies the same finalized archive and source commit.
- [ ] Xtask rehearsal cannot publish and verification fails every tamper fixture.
- [ ] Release jobs are ordered behind validation with least-privilege permissions.
- [ ] One published preview asset set verifies end to end.
- [ ] Only the rolling preview formula mutates before stable readiness.

## STOP Conditions

- The shared path changes public archive names/layout or Homebrew expectations
  without a reviewed migration.
- Two identical clean rehearsals differ and the source of nondeterminism is
  not understood.
- Signing/attestation would require a long-lived secret or broad permissions.
- A sidecar refers to bytes other than the published archive.
- Any external build input is downloaded/extracted without digest verification.
- Stable publishing or formula mutation is opened implicitly.
- Stable publication can bypass tag protection or operator environment approval.

## Remove When

Delete this plan and index row after all release modes share deterministic
construction, atomic verification rejects tampering, and a preview proves the
published contract.

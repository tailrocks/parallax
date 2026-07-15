# Plan 102: Unify and verify deterministic release artifacts

> **Executor instructions**: Preserve public archive names/layout, the rolling
> preview formula, Zig cross-builds, signatures, SBOMs, and attestations. Build
> preview, stable, and local rehearsal artifacts through one byte-producing
> implementation and prove tamper failures before switching workflows.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 094, 096, 101
- **Category**: release / packaging / provenance
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: IN PROGRESS

## Local implementation evidence

- 2026-07-14: built the host `aarch64-unknown-linux-gnu` embedded-UI binary
  with `PARALLAX_VERSION_OVERRIDE=0.1.0-dev+69db387` and
  `--features embed-ui,cross-release-vendored`. `cargo xtask release-rehearse`
  packaged it twice with source epoch `1784059938`; both archives matched and
  produced SHA-256
  `9d744cf76159352b3ff498e7d60558b66db816b7231f706131c025980da9c80a`.
  The promoted archive contains one root-owned executable `parallax` entry
  (mode `0755`), and the binary contains the matching release-identity marker.
- This is a local rehearsal only. Cross-target Zig builds, signed/SBOM/
  provenance sidecars, workflow ordering, and preview publication remain
  intentionally unverified while the implementation program is still active.
- 2026-07-14: added fail-closed unit coverage for release-verification
  provenance inputs: only a full lowercase source SHA, full `refs/*` source
  ref, GitHub signer identity, and workflow path are accepted. `cargo test
  --locked -p parallax-xtask` passes all 57 tests and strict clippy is clean.
- 2026-07-14: release verification now binds the signer identity exactly to the
  requested repository-owned workflow and source ref. Cross-repository
  workflows, malformed repository names, and identity/ref mismatches fail
  before any external verification command runs; the 57-test xtask suite and
  strict clippy remain green.
- 2026-07-14: the Syft CycloneDX action now sets its source name to the archive
  filename and source version to the archive's final SHA-256. This makes the
  emitted SBOM metadata match the fail-closed verifier's exact archive/digest
  contract; a release workflow fixture prevents removal of either flag.
- 2026-07-14: a local Syft 1.45.1 rehearsal against
  `parallax-0.1.0-dev+69db387-aarch64-unknown-linux-gnu.tar.gz` produced
  CycloneDX 1.6 metadata with that exact basename and
  `sha256:9d744cf76159352b3ff498e7d60558b66db816b7231f706131c025980da9c80a`.
  This proves the action's producer flags and verifier expectations agree on
  real archive bytes without publishing an artifact.
- 2026-07-15: expanded the local fail-closed archive verifier fixtures beyond
  checksum/SBOM/object corruption. Independently generated archives with a
  nested binary path, non-executable mode, wrong source epoch, or an extra
  entry now each fail `read_binary`; no fixture relies on the production
  packager to manufacture malformed input. All 10 release-focused xtask tests
  pass and strict all-target xtask clippy is clean on Linux arm64.
- 2026-07-15: completed the local checksum/SBOM syntax tamper matrix. The
  verifier now has fixtures for a wrong digest, uppercase/non-terminated
  digest, multiline checksum, wrong SBOM archive name, wrong archive digest,
  wrong CycloneDX format, malformed JSON, and a missing SBOM. The focused test
  and strict all-target xtask clippy pass locally.
- 2026-07-15: closed the producer-side identity bypass. `release-package` now
  requires target and version, rejects archive names outside the exact preview
  or stable contract, and verifies the built object's architecture, embedded
  version identity, symbol table, and resolvable line tables before writing
  archive bytes. Both preview and stable callers pass their authoritative
  identities. The release suite, CLI parser fixture, and strict all-target
  xtask clippy pass locally.
- 2026-07-15: release identity validation now parses Cargo-flavored SemVer
  through `semver::Version` instead of accepting every whitespace-free string.
  Valid preview build metadata remains supported; incomplete versions,
  `v`-prefixed tags, whitespace, and shell-like suffixes fail before packaging
  or verification. The explicit dependency is locked, all 10 release tests
  pass with `--locked`, and strict all-target xtask clippy is clean.
- 2026-07-15: bound atomic verification to the deterministic outer gzip header,
  not only the inner tar contract. Nonzero flags/mtime, changed compression
  metadata, and a host-specific OS byte now fail even if an alternate producer
  recomputes sidecars. All 11 release-focused tests and strict all-target xtask
  clippy pass locally.
- 2026-07-15: made cosign and GitHub provenance bindings directly testable
  without contacting either service. Pure argument builders now have an exact
  regression fixture covering the finalized archive/bundle, certificate
  identity and OIDC issuer, repository/workflow, source commit/ref, and denial
  of self-hosted-runner attestations. All 12 release-focused tests and strict
  all-target xtask clippy pass locally.
- 2026-07-15: hardened the verified macOS SDK input boundary before cache use.
  The composite action now rejects malformed versions and any digest that is
  not exactly 64 lowercase hex characters before constructing a cache key or
  download URL; restored bytes remain checksum-verified before extraction.
  The release caller fixture locks validation/cache/checksum/extraction order,
  and the focused test plus strict all-target xtask clippy pass locally.
- 2026-07-15: closed the remaining whole-archive canonicality gap in local
  verification. After bounded extraction and metadata validation, the verifier
  now rebuilds the archive with the sole repository packager and requires exact
  compressed-byte equality. Recomputed sidecars can therefore no longer bless
  appended bytes or a different deflate/tar encoding that happens to expose the
  same inner file. Archive input is rejected above 512 MiB before allocation;
  sparse oversized, trailing-byte, and changed-stream fixtures fail closed.
  All 14 release-focused tests and strict all-target xtask clippy pass locally.
- 2026-07-15: unified release-object validation across production packaging and
  local rehearsal. Rehearsal now rejects unsupported architecture/object bytes,
  missing symbols or resolvable line tables, wrong embedded version identity,
  and binaries larger than 512 MiB before its two archive passes. The archive
  determinism primitive remains separately fixture-tested without compressing
  a large debug test executable. All 15 release-focused tests and strict
  all-target xtask clippy pass locally.
- 2026-07-15: strengthened SBOM verification beyond two self-asserted metadata
  strings. The verifier now requires a versioned CycloneDX 1.6 document with a
  UUID serial, generation timestamp, file root component, exact archive
  name/digest, and an application-tool inventory naming the mise-pinned Syft
  1.45.1 producer; a fixture locks the mise/verifier version together and a
  hollow but name/digest-matching SBOM fails closed. A local Syft 1.45.1 scan
  confirmed this exact metadata/tool shape on Linux arm64. All 15
  release-focused tests and strict all-target xtask clippy pass locally.
- 2026-07-15: removed archive-channel ambiguity from packaging and
  verification. Callers must select `preview`, `stable`, or the explicitly
  non-publishable `rehearsal` channel. Preview requires a
  `<version>-preview.<ordinal>+<source>` version and
  `parallax-<target>.tar.gz`; stable requires a release version without
  prerelease/build metadata and `parallax-<version>-<target>.tar.gz`;
  rehearsal permits development SemVer but keeps the versioned name.
  Verification derives only preview from `refs/heads/main` or stable from the
  exact `refs/tags/v<version>` ref before checking signer identity, so no
  rehearsal identity can pass the publication boundary. Both workflows and
  the local rehearsal pass their explicit channel, and fixtures reject crossed
  names, versions, channels, and source refs. All 15 focused release tests, all
  three CLI parser tests, strict xtask Clippy, formatting, and
  `git diff --check` pass locally.

## Why

Parallax already has strong SHA pins, Zig builds, signatures, SBOMs,
attestations, and preview Homebrew automation, but preview, stable, and local
packaging construct archives separately. Tar/gzip metadata is not one proven
contract, release jobs can start independently of complete version validation,
and there is no operator command that verifies the artifact set atomically.

## Scope

- One repository-owned Rust helper/xtask archive implementation used by every
  release mode; composite actions are callers, never independent byte producers.
- Digest-pinned and verified external build inputs, including the macOS SDK,
  with caches keyed by the verified digest.
- Deterministic ordering, paths, timestamps, ownership, permissions, and gzip
  metadata; isolated target/Zig caches.
- Plan 096's source-line/backtrace decision: shipped binaries retain line
  tables (`debug = "line-tables-only"`, `strip = "none"`), so no symbol
  companion is required. Verify this on final archive bytes.
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
expectations. Include debug/line-table presence, strip settings, build IDs,
panic/backtrace source-line fidelity, symbol companion names when selected, and
binary/archive size. Add fixtures that characterize the current public contract.
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

Implement one repository-owned deterministic Rust library/xtask function shared
by preview, stable, and local rehearsal. Composite actions and workflows pass
validated inputs to that same function; they contain no alternate tar/gzip
construction. Normalize file order, relative paths,
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

When plan 096 selects symbol companions, treat each companion as a first-class
version/target/build-ID-bound release asset with checksum, SBOM inventory,
signature/attestation, completeness checks, and retention policy. When line
tables remain in the binary, verify the final published stripped/unstripped
bytes still resolve the representative backtrace.

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

Remove the current cross-repository Homebrew tap write-token path. Prefer a
pull model: a workflow in `tailrocks/homebrew-parallax` uses its own narrowly
scoped repository authority to fetch and independently verify published Parallax
preview assets before updating the formula. If the tap cannot use that model,
the operator must approve a narrowly scoped GitHub App installation token with
repository allowlist, short expiry, rotation/revocation evidence, and audited
workflow identity. OIDC/cosign permissions do not authorize writes to another
repository. A long-lived PAT or unreviewed secret blocks retirement.

## Test Plan

- Golden archive layout/mode/name tests per supported target.
- Representative release panic/backtrace resolution and build-ID/symbol
  mismatch/tamper tests.
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
- Cross-repository credential fixtures proving no long-lived PAT, pull-model or
  approved short-lived App authorization, narrow repository scope, expiry, and
  failure before verified assets.

## Done Criteria

- [ ] Preview, stable, and rehearsal use one archive implementation.
- [ ] Archive construction is deterministic across two clean identical inputs.
- [ ] Tag, workspace, binary, archive, and release versions agree before build.
- [ ] External SDK/tool inputs are digest-verified before extraction and cache use.
- [ ] Published artifacts preserve the approved source-line/backtrace contract,
  and any symbol companions match exact target/version/build ID.
- [ ] Manual stable publication is disabled until explicitly opened and tagged.
- [ ] Stable publication requires protected `v*` tag authorization and approval
  through the protected stable-release environment.
- [ ] Every sidecar identifies the same finalized archive and source commit.
- [ ] Xtask rehearsal cannot publish and verification fails every tamper fixture.
- [ ] Release jobs are ordered behind validation with least-privilege permissions.
- [ ] One published preview asset set verifies end to end.
- [ ] Only the rolling preview formula mutates before stable readiness.
- [ ] The tap update uses its own pull workflow or an operator-approved
  short-lived narrowly scoped GitHub App token; the old long-lived write secret
  and workflow path are absent.

## STOP Conditions

- The shared path changes public archive names/layout or Homebrew expectations
  without a reviewed migration.
- Stripping or symbol separation makes representative captured backtraces
  unresolvable or produces unattested companion bytes.
- Two identical clean rehearsals differ and the source of nondeterminism is
  not understood.
- Signing/attestation would require a long-lived secret or broad permissions.
- The rolling formula can be updated only by retaining an unreviewed long-lived
  PAT or cross-repository write secret.
- A sidecar refers to bytes other than the published archive.
- Any external build input is downloaded/extracted without digest verification.
- Stable publishing or formula mutation is opened implicitly.
- Stable publication can bypass tag protection or operator environment approval.

## Remove When

Delete this plan and index row after all release modes share deterministic
construction, atomic verification rejects tampering, and a preview proves the
published contract.

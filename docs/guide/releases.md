# Rehearse and verify releases

Parallax uses one repository-owned Rust archive implementation for local
rehearsal, rolling previews, and future stable releases. Every archive contains
one top-level `parallax` executable with normalized metadata and retains source
line tables; there are no separate symbol archives.

## Local rehearsal

Run the build and deterministic two-pass packaging path for the host target:

```bash
scripts/release.sh
```

Pass a supported target triple to rehearse a cross build. The script builds the
embedded UI, uses Zig/cargo-zigbuild with the native-TLS vendored OpenSSL path,
packages the same binary twice, and fails unless both archive digests match. It
writes the archive and bare-hash `.sha256` under `target/dist/` and never
publishes. The script explicitly selects the non-publishable `rehearsal`
channel and produces the versioned archive shape. Every package caller must
select exactly one `--channel preview|stable|rehearsal`. Preview requires a
`<version>-preview.<ordinal>+<source>` version and
`parallax-<target>.tar.gz`; stable requires a release version without
prerelease or build metadata and `parallax-<version>-<target>.tar.gz`.
Rehearsal accepts development SemVer and the versioned name, but published
verification derives only preview or stable from the trusted source ref, so a
rehearsal can never satisfy the publication contract.

Release workflows run `cargo xtask release-validate` in their metadata gate,
before any test or cross-build matrix starts. This rejects a preview identity
without its ordinal/source metadata and rejects stable versions containing any
prerelease or build metadata before release resources are spent.

## Verify published preview assets

Install repository tools through mise, download one archive with its three
sidecars, and resolve the preview source identity:

```bash
mise install
mkdir -p target/verify-preview
gh release download preview \
  --repo tailrocks/parallax \
  --dir target/verify-preview \
  --pattern 'parallax-x86_64-unknown-linux-gnu.tar.gz*'
source_sha=$(gh release view preview --repo tailrocks/parallax \
  --json targetCommitish --jq .targetCommitish)
version=$(gh release view preview --repo tailrocks/parallax \
  --json name --jq '.name | sub("^Preview "; "")')
source_epoch=$(git show -s --format=%ct "$source_sha")
```

Then run atomic verification from this repository checkout:

```bash
mise exec -- cargo xtask release-verify \
  --archive target/verify-preview/parallax-x86_64-unknown-linux-gnu.tar.gz \
  --target x86_64-unknown-linux-gnu \
  --version "$version" \
  --source-epoch "$source_epoch" \
  --source-commit "$source_sha" \
  --source-ref refs/heads/main \
  --signer-identity \
    https://github.com/tailrocks/parallax/.github/workflows/preview.yml@refs/heads/main \
  --signer-workflow tailrocks/parallax/.github/workflows/preview.yml
```

The command rejects an unexpected archive name/layout/mode/owner/mtime,
target format or architecture, missing line or symbol tables, mismatched
embedded version identity, checksum or CycloneDX digest drift, a missing
Sigstore bundle, the wrong workflow certificate identity, self-hosted
provenance, or a different source commit/ref.
Preview verification accepts only `refs/heads/main` plus the unversioned
preview name. Stable verification accepts only the exact
`refs/tags/v<version>` plus its versioned archive name.

## Publication authority

The Parallax preview workflow can write only the rolling GitHub prerelease. The
`tailrocks/homebrew-parallax` repository independently pulls and verifies all
four archives and sidecars with its own scheduled/manual workflow before its
own token updates `parallax-preview.rb`. Parallax has no tap write token or
cross-repository checkout path.

Stable release readiness is closed. The stable workflow has no manual dispatch
and fails before build unless the operator explicitly sets
`STABLE_RELEASE_ENABLED=true`, creates a reviewer-protected `stable-release`
environment, and activates a `refs/tags/v*` ruleset that restricts tag creation,
update, and deletion. The tag, workspace version, binary identity, archive
name, signature, provenance, and release title must all agree after readiness
is opened.

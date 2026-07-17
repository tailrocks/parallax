# Plan 102 release artifact and authorization baseline

Validation date: 2026-07-13
Baseline: `00e3b10`

## Public artifact contract

Both GitHub release workflows build four targets with Zig/cargo-zigbuild:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `aarch64-unknown-linux-gnu` (glibc 2.17 floor)
- `x86_64-unknown-linux-gnu` (glibc 2.17 floor)

The rolling `preview` release publishes
`parallax-<target>.tar.gz`; the stable workflow publishes
`parallax-<version>-<target>.tar.gz`. Each archive contains exactly one
top-level executable named `parallax`. The Homebrew preview formula depends on
those unversioned preview names and installs that top-level executable. Its
alias is `parallax@preview` and the disabled stable formula remains separate.

Every archive has three uploaded sidecars:

- `.sha256`: lowercase SHA-256 hex plus newline, without a filename;
- `.bundle`: keyless Sigstore bundle for the exact archive bytes; and
- `.sbom.json`: CycloneDX 1.6 document whose root component version is the
  archive SHA-256.

GitHub build provenance is an attestation service record for the archive, not
an uploaded fourth sidecar. The current preview signature identity is
`https://github.com/tailrocks/parallax/.github/workflows/preview.yml@refs/heads/main`,
issued by `https://token.actions.githubusercontent.com`.

## Published preview characterization

The independently downloaded preview at source
`4e8edfa5f92cd8060dfdd46dccb82a0fa26613f8` identifies itself as
`0.1.0-preview.958+4e8edfa`. Its asset set is complete across all four targets.

| Target | Archive bytes | SHA-256 |
| --- | ---: | --- |
| `aarch64-apple-darwin` | 14,922,192 | `aa82a8313e7b3bf6d03ea0d8a6bcdf580ae2ff7d3c5bf673d7d6682e166c193e` |
| `x86_64-apple-darwin` | 16,014,983 | `299d4132e69f121d9dd919028fa48cd1a397ef8b8ecd150b2604a9d68639298c` |
| `aarch64-unknown-linux-gnu` | 64,521,549 | `f711f80ab8d9b037c2f26f720ef956f25cb5d13fa0fd90656c3fb3b4893a2821` |
| `x86_64-unknown-linux-gnu` | 64,606,502 | `07bb6011c6f0e9c7a8a93c365c9bd9834dc993ec911066b09e4b74f51c458c9a` |

The downloaded x86-64 Linux archive passes its checksum, exact cosign identity
verification, and GitHub provenance verification. Its binary is 282,342,264
bytes and retains `.debug_info`, `.debug_line`, and `.symtab`, matching the
`line-tables-only`/`strip = "none"` contract. No symbol companion exists or is
required.

Current tar metadata is not reproducible: the entry is mode `0755`, but carries
the GitHub runner owner/group and build wall-clock mtime. GNU gzip already emits
mtime zero and no filename, but its OS byte is platform-derived. Preview and
stable each call system `tar` independently, while `scripts/release.sh` calls a
third system-tar path and also adds a conflicting `v` to its local filename.

## External inputs and authorization

Both release workflows download and pipe-extract
`MacOSX26.1.sdk.tar.xz` without checking it. The immutable GitHub release asset
API reports SHA-256
`beee7212d265a6d2867d0236cc069314b38d5fb3486a6515734e76fa210c784c`;
cache keys currently omit that digest.

Live repository evidence on 2026-07-13:

- the only Parallax ruleset protects `refs/heads/main`; it does not protect
  `v*` tags;
- there is no `stable-release` environment or any other environment;
- no stable tag/release exists;
- the stable workflow accepted arbitrary manual dispatch from `main`; and
- the preview workflow checks out `tailrocks/homebrew-parallax` with
  `GH_PARALLAX_HOMEBREW_TAP_TOKEN`, creates a branch and PR there, then merges
  it. The tap has no pull workflow of its own.

Stable readiness is not open. This change therefore removes manual stable
dispatch immediately. Tag publication remains subject to Plan 102's subsequent
fail-closed authorization work; it must not be exercised until protected-tag
and environment evidence exists.

No research prompt changed: this is implementation/authorization evidence for
an existing plan, not a new research direction.

## Stable readiness protections configured live (2026-07-17)

Per the 2026-07-17 unblock directive, the operator-delegated protections were
created via the GitHub API and verified by read-back:

- Repository Actions variable `STABLE_RELEASE_ENABLED=true` (created; read-back
  confirmed name and value).
- Environment `stable-release` (id 18300879032) with a `required_reviewers`
  protection rule listing user `donbeave` (id 139017),
  `prevent_self_review=false`.
- Repository ruleset `stable tag protection` (id 19090444): `target=tag`,
  `enforcement=active`, conditions include `refs/tags/v*`, rules
  `creation`, `update`, `deletion`, `non_fast_forward`; bypass restricted to
  the repository admin role (`actor_id 5, RepositoryRole, always`).

Remaining for plan 102: one complete four-target preview asset set published
by the current implementation from a green `main` push (CI on `main` was red
at configuration time from in-flight Wave 2 alerting work, so the
`workflow_run`-gated preview publish is pending a green head), then
per-target `cargo xtask release-verify` plus tap pull-workflow acceptance.

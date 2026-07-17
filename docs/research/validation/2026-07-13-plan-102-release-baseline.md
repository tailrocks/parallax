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

## External proof attempt (2026-07-17T08:30Z UTC) — BLOCKED

Protections re-verified live:

| Control | Evidence |
| --- | --- |
| `STABLE_RELEASE_ENABLED` | `true` (Actions variable) |
| Environment | `stable-release` present |
| Tag ruleset | `stable tag protection` active, `target=tag` |
| Branch ruleset | `main protection` still active |

Rolling preview still points at pre-verifier SHA
`4e8edfa5f92cd8060dfdd46dccb82a0fa26613f8`
(`0.1.0-preview.958+4e8edfa`). Last **successful** `Publish Homebrew Preview`
run is still that SHA (run `29223792389`, 2026-07-13). Homebrew formula
`# source-sha:` matches it.

### Why a current-implementation preview cannot publish

After the packaging/verifier unification landed, every post-fix preview that
actually built failed at **Package** on Apple targets with:

```text
error: release binary is missing line tables
```

Reproduced on CI:

| Run | Source SHA | Failure |
| ---: | --- | --- |
| `29548131177` | `ba85f86cc544134a0dd50be1702b855e4aef98bc` | `build-preview (aarch64-apple-darwin)` Package |
| `29546944724` | `882dcb9cda114f60400bfe8617ec65c2de8a1c67` | same |
| `29545963394` | `d8e7a192f6f9ea81dfec216bb4a8146a8ae0b3e4` | same |

`release-package` calls `verify_object`, which requires a section named
`.debug_line` / `.zdebug_line` / `__debug_line` / `__zdebug_line` inside the
final linked binary. That check fails for Mach-O release artifacts produced
by the current toolchain (`rustc 1.97` + `cargo zigbuild` / Apple `ld` and
Zig's Mach-O linker).

Local and Linux-container reproduction (minimal `debug = "line-tables-only"`,
`strip = "none"`, `split-debuginfo = "off"` crate, target
`aarch64-apple-darwin`):

1. Object files **do** contain `__TEXT,__debug_line` and `__DWARF,*` before
   link (confirmed with a fake linker that inspects `.rcgu.o` inputs).
2. The **final linked executable has no `__DWARF` / `__debug_line` sections**
   — Apple `ld` and Zig's Mach-O link path keep a debug map / dSYM path, not
   embedded DWARF segments. Confirmed on host macOS and in
   `rust:1.87-bookworm` + `cargo-zigbuild 0.21.8` + Zig 0.14 producing a
   454 440-byte binary with `has __debug_line False`.
3. `split-debuginfo = "packed"` yields a `.dSYM` companion whose DWARF
   payload has `__DWARF,__debug_line`, but plan 102 forbids a symbol
   companion; the shipped archive is the executable alone.
4. The **old** published `parallax-aarch64-apple-darwin.tar.gz` from
   `4e8edfa` also has no DWARF sections. Current
   `cargo xtask release-package` rejects it with the same
   `release binary is missing line tables` error — so the last green preview
   cannot prove the current verifier.

Linux ELF targets were cancelled on the failing runs after the Apple matrix
leg failed; they are not independently proven green under the new package
gate either for a complete four-target publish.

### Path-filter / CI thrash (secondary)

Many `Publish Homebrew Preview` runs on 2026-07-17 conclude `skipped` because
the triggering CI run was `cancelled` or docs-only (job `if` requires
`workflow_run.conclusion == 'success'`). That is expected gating, not the
primary blocker: when preview **did** build after green CI, Package failed
on macOS line tables as above.

### Unblock required before retirement

Do **not** retire plan 102 until:

1. Mach-O release binaries retain embeddable line tables that
   `verify_object` accepts (structural fix in the build/link/package path —
   not a companion sidecar, not a verifier weaken-only patch without a real
   symbolication surface), and
2. One preview from that fixed implementation publishes all four targets, and
3. Each target passes `cargo xtask release-verify` at the exact source SHA/ref,
   and
4. The tap pull workflow accepts the set (sanitized evidence only).

No stable tag was cut. No tap write credential was restored. No older preview
was treated as proof of the current byte-producing implementation.

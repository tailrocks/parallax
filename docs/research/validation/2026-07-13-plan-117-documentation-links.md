# Plan 117 documentation link-integrity validation

Validation date: 2026-07-13
Implementation range: `714cb1e..ae890b0`
Final validation candidate: `ae890b0`

## Contract

`cargo xtask docs links` uses exact `pulldown-cmark@0.13.4` with GFM options
and source-offset events. It selects tracked Markdown through `git ls-files`,
fails on an empty selection or unreadable file, and structurally observes link
and image events. Fenced code and literal examples therefore do not become
targets.

Internal destinations resolve from the owning document, percent-decode paths,
normalize `.`/`..`, reject repository-root escape, and validate files or
directories. File fragments resolve against parsed GitHub-style heading slugs,
including duplicate suffixes and Unicode. External HTTP(S), mail, data, and app
links remain network-independent and outside the required gate.

Every failure uses the common schema-versioned diagnostic renderer with file,
source line, target/reason, remediation, and rerun command in human, JSON, or
GitHub format. Findings aggregate rather than stopping at the first broken
target.

## Fixture and repository evidence

The parser/resolver fixture covers inline and reference links, images,
directories, percent-encoded spaces, Unicode and duplicate headings, fenced
code, missing paths, missing fragments, malformed percent encoding, and root
escape. CLI parsing and fast/full partition inventories prove the command is
not a placeholder. Existing common-renderer tests prove human/JSON/GitHub
schema parity.

The first repository scan found ten real stale links to retired Plans 092, 093,
094, 099, and 101. They now point to durable validation packets or the Plan 092
closure commit. The final scan passes all 277 tracked Markdown files.

The path classifier selects the gate for every Markdown add/delete/rename, its
parser/config/source/test inputs, the Cargo lock/toolchain, and CI workflow.
`docs-links` is a separate required GitHub Actions job and an explicit
`ci-required` dependency. `cargo xtask ci --full` invokes the same function;
the required check performs no network request.

## Final gates

- 46 `parallax-xtask` tests: passed.
- Workspace strict Clippy: passed with zero warnings.
- `cargo xtask policy`: passed without adding a structural ratchet.
- `cargo xtask dependencies --all`: audit, deny, shear, hack, feature trees,
  frozen Bun install/audit/lifecycle checks passed.
- actionlint, shellcheck, 11 path-classifier fixtures, and workflow-policy
  fixtures: passed.
- `cargo xtask docs links`: 277 tracked Markdown files passed.
- GitHub Actions run
  [29223433009](https://github.com/tailrocks/parallax/actions/runs/29223433009):
  exact-candidate `docs-links` job and every other selected job passed; the
  stable `ci-required` aggregate concluded success.

No research prompt changed: Plan 117 implemented an existing documentation
quality requirement and did not change product research direction or criteria.

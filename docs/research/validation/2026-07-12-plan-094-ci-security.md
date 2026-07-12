# Plan 094 CI and repository-security validation

Validation date: 2026-07-12

## Advisory baseline

- `cargo-audit` 0.22.2 is pinned through mise.
- RustSec loaded 1,160 advisories and scanned 510 locked dependencies.
- Result: clean; no vulnerability or warning exception is configured.
- `anyhow` is 1.0.103 and `crossbeam-epoch` is 0.9.20 in `Cargo.lock`.

## Repository ruleset

Read-back from the GitHub repository-rules API after configuration:

- repository visibility: public;
- ruleset: `main protection`, repository-scoped, active, targeting only
  `refs/heads/main`;
- ref safety: deletion and non-fast-forward updates blocked;
- review: one approval, last-pusher approval prohibited, stale reviews
  dismissed, and review threads resolved;
- strict required checks: `ci-required` and `DCO`;
- bypass: organization administrators, always, reserved by policy for
  explicitly authorized operations.

Before configuration the ruleset list was empty and the legacy branch-
protection endpoint returned `404 Branch not protected`. The active ruleset was
created and immediately read back through GitHub's API; no token, actor identity,
or private reporting detail is retained here.

## Local fixture and UI evidence

- Path classifier: 9 table-driven cases.
- Event-range resolver: 4 cases, including initial and missing-base behavior.
- Aggregate result policy: 4 success/skipped/failure/cancelled cases.
- Source hygiene: 6 PR/push/zero-base/missing-base/staged/unstaged cases.
- Workflow policy: full-SHA action pins, explicit aggregate inputs, parallel
  check/Clippy lanes, and no `write-all` permission.
- Bun execution contract: 9 scripts, lock-local executables, auto-install off,
  and a Node-shebang probe executing under Bun.
- Forced-Bun Vitest: two clean consecutive runs, 41 files and 175 tests each,
  with no unexpected `scrollTo` diagnostic.
- UI format, ESLint, strict TypeScript, production build, and generated route-
  tree drift checks passed. The existing large-client-chunk warning remains
  owned by Plan 148.

## Hosted required-check evidence

GitHub Actions run `29200131016` on commit `0dc78a2` completed successfully.
The stable `ci-required` aggregate passed after actionlint/policy fixtures,
source hygiene, Rust format, advisory audit, check, Clippy, 195 nextest tests,
UI, and embedded-UI lanes all succeeded. Check and Clippy ran as siblings.

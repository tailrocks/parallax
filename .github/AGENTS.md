# GitHub Actions runner policy

Every Linux workflow uses one YAML shape on all lanes:

- `velnor` is the default and runs on
  `self-hosted,velnor-target-mvp`.
- `github` is the comparison/recovery lane and runs on
  `ubuntu-26.04`; never use `ubuntu-latest` or an unpinned Ubuntu label.
- `both` executes identical jobs and steps on both lanes.

Sunday parity selects `both`; other automatic events remain Velnor-default.

Use the `lanes` choice input and canonical inline `matrix.config` expression.
Only `matrix.config.writer` may gate mutations and must select exactly one
writer. Never branch step semantics by lane.

Rust compile jobs use mold and local-only sccache v0.16.0 with a 20 GiB bound.
The adapter owns cache reporting. Persist content-compatible target-directory
generations with a ref-independent restore prefix; do not compile CI tooling or
enable a remote cache backend.
Install tools through mise on both lanes; do not add ad hoc installers that
bypass the shared mise store.

Every job has measured `timeout-minutes`; every workflow has concurrency and
intentional cancellation. Checkouts are shallow and disable
credential persistence unless a documented writer step requires otherwise.

The GitHub lane stays so releases work when Velnor is unavailable. Native
Darwin release builders are the documented
exception: Apple `dsymutil`, codesign, and linker semantics require macOS until
a byte-valid Linux cross-build path is proven. Changes to runner labels, lane
matrices, actions, or cache behavior must pass `velnor`, `github`, and `both`
verification.

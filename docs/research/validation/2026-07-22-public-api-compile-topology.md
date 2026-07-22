# Public API compile topology

Date: 2026-07-22

## Failure class

Exact-head GitHub run `29885434328` compiled the workspace normally, then the
`parallax-server::public_api` nextest case launched a second Cargo graph through
trybuild. On the cold GitHub host that nested graph spent more than five minutes
recompiling dependencies. Nextest terminated attempt one at the unchanged
five-minute bound; the now-warm second attempt passed in 110 seconds. The
evidence gate correctly rejected that flaky pass.

Full nextest-pool reservation removed scheduler competition but could not remove
the nested compiler graph. A larger timeout or retry would preserve the
enabling structure and make cold correctness depend on machine speed.

## Structural correction

The supported lifecycle imports now compile as an ordinary Rust integration
test. Cargo therefore proves them while building the normal test graph, before
nextest runs. Private-module exclusion uses fixed assertions against the
committed syntax-derived facade manifest; `cargo xtask facade check` separately
proves that manifest matches current Rust visibility. Deliberately refreshing a
widened manifest cannot hide an accidental `pub mod worker` or
`pub mod self_telemetry` because the fixed exclusion assertions still fail.

The server no longer needs trybuild, its UI fixture graph, full-pool override,
or special five-minute allowance. `parallax-api` retains trybuild for its
different compile-fail contracts.

## Verification

- `cargo nextest run -p parallax-server --test public_api --locked`
- `cargo xtask facade check`
- strict package clippy and formatting
- exact-head GitHub and Velnor lane proof remain required

No assertion, timeout, retry policy, or public/private API expectation was
weakened.

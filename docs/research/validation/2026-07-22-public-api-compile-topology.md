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

Exact-head GitHub run `29887096900` then exposed the same failure class in
`parallax-api::public_api`: its compile-fail trybuild graph timed out at 60
seconds on both attempts. This proved nested compiler ownership was the shared
architectural defect, not a server-specific test-size problem.

## Structural correction

The supported lifecycle imports now compile as an ordinary Rust integration
test. Cargo therefore proves them while building the normal test graph, before
nextest runs. Private-module exclusions use source predicates plus committed
syntax-derived facade manifests; `cargo xtask facade check` separately proves
those manifests match current Rust visibility. Deliberately refreshing a
widened manifest cannot hide accidental visibility because the source
predicates still fail.

Neither server nor API now launches trybuild. Their UI fixture graphs, full-pool
override, special five-minute allowance, and now-unused trybuild dependency are
removed.

## Verification

- `cargo nextest run -p parallax-server --test public_api --locked`
- `cargo nextest run -p parallax-api --test public_api --locked`
- `cargo xtask facade check`
- strict package clippy and formatting
- exact-head GitHub and Velnor lane proof remain required

No assertion, timeout, retry policy, or public/private API expectation was
weakened.

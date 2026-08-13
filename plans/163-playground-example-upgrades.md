# Plan 163: Upgrade every playground example to current-latest instrumentation and re-verify emission

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md` (parallax repo).
>
> **Drift check (run first)**:
> `git -C ../parallax-telemetry-playground diff --stat 6e0a0d5..HEAD -- Cargo.toml services web libs mise.toml`
> and `git diff --stat f6208070..HEAD -- bench/otlp-fanout/lab.env`.
> On mismatch with "Current state" excerpts, STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED (major-version SDK bumps can change emitted telemetry shape)
- **Depends on**: plans/162-fanout-lab-backend-pins.md
- **Category**: migration
- **Planned at**: parallax `f6208070`, playground `6e0a0d5`, 2026-08-13

## Why this matters

The playground is the payload every backend is judged on. Its value claim is
"max-fidelity, current-generation instrumentation"; stale SDKs measure
yesterday's ecosystem. The README verified matrix is dated 2026-06-23 —
~7 weeks behind code — and one known delivery snag (Java agent → Rotel →
OpenObserve) is still marked unresolved there. After this plan, every example
uses the latest mutually-compatible stable instrumentation, the whole stack
builds and runs green, and the verified matrix is re-dated from a live run.

## Current state

Repo: `../parallax-telemetry-playground` (sibling checkout of
github.com/tailrocks/parallax-telemetry-playground). Structure: Cargo
workspace (8 Rust services + `libs/playground-telemetry` + `proto` + `cli`),
3 Java Spring Boot services (`services/catalog`, `services/payment`,
`services/fulfillment`, each with own `build.gradle.kts`), `web/` (TanStack
Start, Bun-only), `deploy/` Dockerfiles, toolchain in `mise.toml`.

Version anchors as of `6e0a0d5` (resolve latest at execution; these are the
"from" values):

Rust (`Cargo.toml` workspace deps): `opentelemetry 0.32` family
(`opentelemetry_sdk`, `-otlp`, `-appender-tracing`, `-proto`, `-http`,
`-semantic-conventions` all 0.32), `tracing-opentelemetry 0.33`,
`sentry 0.48` + `sentry-tracing 0.48`, `axum 0.8`, `tonic 0.14`,
`sqlx 0.9.0` (native-tls comment at lines 38–40 — never rustls),
`juniper 0.17.1`, `open-feature 0.3.0`, `open-feature-flagd 0.2.1`.

Java (`services/catalog/build.gradle.kts`): Boot plugin `4.1.0` (line 6),
`com.atkinsondev.opentelemetry-build 4.6.2` (line 8),
`io.sentry:sentry-spring-boot-4-starter:8.46.0` (line 31),
`dev.openfeature:sdk:1.21.0` (32), `dev.openfeature.contrib.providers:flagd:0.14.0` (33),
`io.opentelemetry.javaagent:opentelemetry-javaagent:2.29.0` (line 43).
`payment` and `fulfillment` build files mirror these — change all three.

Web (`web/package.json`): OTel JS stable SDKs `^2.8.0`
(`sdk-trace-web`, `core`, `resources`, `context-zone`), experimental line
`0.220.0` (`api-logs`, `sdk-logs`, `exporter-*-otlp-proto`), instrumentations
`^0.219.0`/`^0.63`/`^0.64`, `@sentry/tanstackstart-react ^10.0.0`,
`@sentry/vite-plugin 5.3.0`, react `^19`, vite `^8`, typescript `^6`,
`@playwright/test ^1.61.1`, `vitest 4.1.6`, `nitro ^3`.

Toolchain (`mise.toml`): rust 1.97.0, bun 1.3.14, java oracle-graalvm-25.0.3,
cargo-nextest 0.9.140, actionlint 1.7.12, protoc 35.1.

Known-issue anchors to re-test after upgrading:
- `deploy/docker-compose.yml:129-131` — Java tier forced to OTLP/HTTP because
  "the agent's okhttp gRPC sender fails to read Rotel's gRPC response". After
  the agent + Rotel bumps, re-test gRPC; keep HTTP if still broken and update
  the comment with the retested versions.
- Playground `VERIFICATION.md` "Known version blocker — Rust
  `sentry-opentelemetry` (shared trace_id)" — re-check whether the crate now
  supports the current sentry/opentelemetry pair; adopt it only if it does.
- Playground `README.md` verified matrix — dated 2026-06-23, including the
  unresolved "Java-agent→Rotel→OpenObserve delivery" row.
- parallax `bench/otlp-fanout/lab.env:3` — stale text `parallax run start`;
  the live CLI verb is `parallax invocation start` (the `run` alias is
  rejected by the CLI). Fix the comment text.

Repo constraints (playground `AGENTS`-equivalent conventions + parallax
`AGENTS.md`): Bun only (never npm/pnpm/node), `bun.lock` sole JS lockfile,
native TLS only (sqlx/reqwest native-tls features — never rustls), Rust
edition 2024, cargo-nextest as runner, fmt+clippy zero warnings, latest
stable everything with version tables updated in the same commit.

## Commands you will need

All run in `../parallax-telemetry-playground` unless noted.

| Purpose | Command | Expected on success |
|---|---|---|
| Toolchain | `mise install` | exit 0 |
| Rust latest resolution | `cargo upgrade --dry-run` (cargo-edit) or edit + `cargo update` | lockfile resolves |
| Rust check | `mise run ci` (`cargo check --locked --workspace --all-targets --all-features`) | exit 0 |
| Rust tests | `mise run test` (nextest, `--no-tests=fail`) | all pass |
| Lint | `mise run lint` && `mise run fmt` | zero warnings |
| Java build+test | `cd services/catalog && ./gradlew build` (repeat payment, fulfillment) | BUILD SUCCESSFUL |
| Web install | `cd web && bun install` | `bun.lock` updated, exit 0 |
| Web typecheck/tests | `cd web && bun run typecheck && bun run test` (see `web/package.json` scripts) | exit 0 |
| Full stack | `docker compose -f deploy/docker-compose.yml build && docker compose -f deploy/docker-compose.yml up -d` | all services up |
| Machine verification | `cargo run -p playground-cli -- test-verify` (see `VERIFICATION.md` for the exact invocation used there) | acceptance checks pass |

## Scope

**In scope** (playground repo): `Cargo.toml`, `Cargo.lock`, `mise.toml`,
`mise.lock`, `services/*/build.gradle.kts`, `web/package.json`, `bun.lock`,
`deploy/Dockerfile.*`, `deploy/docker-compose.yml` (comment/env updates tied
to retests only), `libs/playground-telemetry/src/**` and service source files
*only where upgraded SDK APIs force changes*, `README.md` (verified matrix),
`VERIFICATION.md` (re-dated results).
**In scope** (parallax repo): `bench/otlp-fanout/lab.env` (comment text only).

**Out of scope**:
- New scenarios or features (plan 164).
- Backend image pins (plan 162 — must already be merged).
- Parallax product code `crates/`, `ui/`.
- Switching Java to `sentry-opentelemetry-agent` — deliberately NOT used
  (it would hijack the OTLP fan-out; documented in `deploy/Dockerfile.java`).
  Keep upstream OTel javaagent + Sentry Spring starter split.

## Git workflow

PR-only `main` in both repos; one branch + one PR per repo; `git commit -s`;
Conventional Commits; agent trailer per `COMMITS.md`.

## Steps

### Step 1: Toolchain floor

Bump `mise.toml` versions to latest stable (rust, bun, GraalVM LTS line,
nextest, protoc, actionlint): check each with its upstream release page or
`mise ls-remote <tool> | tail`. Run `mise install`.

**Verify**: `mise run ci` → exit 0 on the *unchanged* deps (toolchain-only
bump builds clean).

### Step 2: Rust dependency upgrades

Raise every `[workspace.dependencies]` line to latest stable. The
OpenTelemetry Rust family must move in lockstep (all `opentelemetry*` +
`tracing-opentelemetry` from the same release train — check the
open-telemetry/opentelemetry-rust release notes for the paired
`tracing-opentelemetry` version). Bump `sentry`/`sentry-tracing` together.
Preserve feature flags exactly (`native-tls`, no rustls anywhere:
`grep -rn rustls Cargo.toml` must stay comment-only).

**Verify**: `mise run ci && mise run lint && mise run fmt && mise run test`
→ all exit 0, zero warnings.

### Step 3: Java upgrades

In all three `build.gradle.kts`: Boot plugin latest stable (4.1.x+),
`opentelemetry-javaagent` latest 2.x, `sentry-spring-boot-4-starter` latest
8.x, OpenFeature sdk/flagd provider latest, `com.atkinsondev.opentelemetry-build`
latest. Keep the Java 25 toolchain line unless a newer LTS is in `mise.toml`.

**Verify**: `./gradlew build` in each of the three service dirs →
BUILD SUCCESSFUL, tests pass.

### Step 4: Web upgrades

`cd web && bun update` then raise pinned/experimental OTel lines by hand to
the current stable (2.x) + matching experimental (0.2xx) pair — the two
trains must match per the OTel JS compatibility table in their release notes.
Bump `@sentry/tanstackstart-react` and `@sentry/vite-plugin` to latest.
TypeScript sources stay `.ts`/`.tsx`, strict flags untouched.

**Verify**: `bun install && bun run typecheck && bun run test` → exit 0.
`git status` shows only `web/package.json` + `bun.lock` (+ source edits
forced by API changes).

### Step 5: Rebuild the stack and re-run the emission contract

`docker compose -f deploy/docker-compose.yml build` then `up -d` against a
running fan-out lab (plan 162 pins). Drive baseline traffic
(`scenarios/a1-checkout.sh`, `scenarios/run.sh` catalog per its README) and
run the machine verification path documented in `VERIFICATION.md`
(`playground test-verify` + `smoke.sh` in parallax `bench/otlp-fanout/`).

Re-test the two known issues:
- Java agent → Rotel over **gRPC** (flip `OTEL_EXPORTER_OTLP_PROTOCOL` back
  to `grpc` for catalog only, restart, check delivery). If fixed: adopt gRPC
  and update the compose comment. If still broken: keep HTTP and update the
  comment with "retested at agent <ver> / rotel <tag>, still broken".
- Java-agent→Rotel→OpenObserve delivery row in `README.md`.

**Verify**: every service exports traces+logs+metrics to all lab backends
(per-backend smoke counts match); Sentry envelopes still arrive from Rust,
Java, and web SDK paths (Sentry UI shows events from all three platforms).

### Step 6: Refresh the paper trail

- Playground `README.md`: verified matrix re-dated with today's results.
- `VERIFICATION.md`: append the re-run results; update or close the
  `sentry-opentelemetry` version-blocker note per Step 2 findings.
- parallax `bench/otlp-fanout/lab.env:3`: change the comment text
  `parallax run start` → `parallax invocation start` (two occurrences check:
  `grep -rn "run start" bench/otlp-fanout/` → none after).

**Verify**: `grep -n "2026-06-23" README.md` → no stale matrix date;
parallax `cargo xtask docs links` → passes.

## Test plan

The playground's own test suites are the regression net: Rust nextest
workspace suite, three Gradle test suites (which also emit build/test traces
via the opentelemetry-build plugin), web `bun run test` + Playwright if
configured. All must pass at the new versions. The live Step 5 run is the
integration test; record per-backend results in the PR.

## Done criteria

- [ ] Every dependency family listed in "Current state" is at latest stable
      resolved at execution date; no rustls feature anywhere
      (`grep -rn "rustls" ../parallax-telemetry-playground/Cargo.toml` →
      comments only).
- [ ] `mise run ci|lint|fmt|test`, 3× `./gradlew build`, `bun run typecheck`
      + `bun run test` all exit 0.
- [ ] Full compose stack runs; smoke + `test-verify` pass against the pinned
      lab; Sentry receives envelopes from Rust+Java+web.
- [ ] README verified matrix re-dated; Java-gRPC and sentry-opentelemetry
      notes updated with retest evidence.
- [ ] `bench/otlp-fanout/lab.env` says `invocation start`.
- [ ] `plans/README.md` (parallax) row updated.

## STOP conditions

1. Drift check fails against the excerpts above.
2. An OTel Rust upgrade forces an emitted-schema change that breaks
   Parallax ingest asserts (`test-verify` fails on Parallax arm) — report
   the exact signal + field; do not "fix" by patching Parallax in this plan.
3. Boot 4.x latest breaks `spring-grpc` payment or GraphQL catalog beyond a
   one-line dependency alignment — report versions and the failing module.
4. OTel JS stable/experimental pair mismatch (typecheck or runtime export
   errors) that the documented compatibility table doesn't resolve.
5. Any fix appears to require a rustls feature or a non-Bun package manager.

## Maintenance notes

- Renovate keeps proposing bumps after this lands; the lockstep rules above
  (OTel Rust train, OTel JS stable+experimental pair, Boot+Sentry starter)
  are the review checklist for those PRs.
- If Step 5 adopts gRPC for the Java tier, plan-164 scenarios that assert
  protocol-specific behavior must use the new protocol.
- Deferred: `io.sentry.jvm.gradle` source-context upload (commented out in
  build.gradle.kts line 9) — needs a DSN/org decision, not a version issue.

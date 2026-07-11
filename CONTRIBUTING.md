# Contributing

This is a public Tailrocks Pte. Ltd. repository, licensed under the
[Apache License, Version 2.0](LICENSE).

Contributions are welcome during pre-release and must be submitted under the
normal project process.

Contributions made with pre-release access are accepted under the Apache
License, Version 2.0: per Section 5 of the license, any contribution
intentionally submitted for inclusion is licensed under Apache-2.0, with no
additional terms or conditions (inbound = outbound).

Do not submit confidential, proprietary, customer, employer, or third-party
material unless you have written authority to provide it under Apache-2.0.

External pre-release reviews remain governed by the reviewer-agreement process
in [REPOSITORY_PROTECTION.md](REPOSITORY_PROTECTION.md).

## Development

### Prerequisites

From the repository root:

```bash
mise install
```

That installs Bun and the other tools pinned in `mise.toml`. The Rust
toolchain resolves via `rust-toolchain.toml` (stable) when you run Cargo.

### Backend

```bash
cargo build --workspace
cargo nextest run --workspace
cargo clippy --workspace --all-targets   # zero warnings expected
cargo fmt --all -- --check
```

Serve the local product (OTLP + GraphQL + managed GreptimeDB):

```bash
cargo run -p parallax-cli -- serve
```

`parallax doctor` diagnoses a running install (server reachability, engine
health, spool segments, data sizes).

### UI

**Bun only** — never `npm`, `pnpm`, `yarn`, or Node as the runtime (see the
JS/TS tooling rule in [AGENTS.md](AGENTS.md)).

```bash
cd ui
bun install
bun run dev          # http://127.0.0.1:3000 ; proxies /graphql → :4000
```

The Vite dev proxy expects `parallax serve` on port 4000. Gate commands:

```bash
bun run typecheck
bun run lint
bun run test         # local; CI uses bun run test:ci
bun run build
```

### Full local gate (mirrors CI)

From repo root:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings` (CI uses `-D warnings`)
3. `cargo nextest run --workspace --all-targets`
4. From `ui/`: `bun run typecheck && bun run lint && bun run test:ci && bun run build`

Add UI components with Bun's shadcn runner, not `npx` — see [ui/AGENTS.md](ui/AGENTS.md).

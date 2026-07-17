# Plan 109 evidence — minimal local-first token auth (2026-07-17)

## Decision

ADR: [`docs/research/decisions/v2-auth-and-context-contract.md`](../../decisions/v2-auth-and-context-contract.md)

Authority: operator unblock directive 2026-07-17 (minimal recommended shape).

## Implemented surface

| Piece | Location |
| --- | --- |
| Optional API bearer | `[server] api_token` + env `PARALLAX_API_TOKEN` |
| Middleware | `POST /graphql`, live SSE streams |
| Open probes | `/health`, `/version`, OTLP, UI static |
| Non-loopback without token | config validate fails |
| CLI contexts | `parallax context add/list/use/show/remove` |
| Client bearer injection | `parallax-cli` GraphQL + SSE |
| Ready banner | `auth off` / `auth bearer-token` (no secret) |

## Verification

```text
cargo test -p parallax-server --lib -- config::tests auth_tests
cargo test -p parallax-server --test m109_api_auth
cargo test -p parallax-cli --bin parallax -- client::
```

Observed green on the implementer host (2026-07-17): unit auth matrix +
`m109_api_auth` open-mode and bearer deny/allow paths.

## Residual (deferred; plan 109 retired 2026-07-17)

Minimal contract v1 done criteria are met; plan file deleted. Later-scope items
remain recorded only on the ADR
([`v2-auth-and-context-contract.md`](../../decisions/v2-auth-and-context-contract.md)):

- OS keyring backend (file mode 0600 is the v1 storage).
- OTLP ingest tokens → plan 115 server profile.
- Multi-capability RBAC / revocation list / expiry → future auth contract rev.
- Browser session path for UI when API token is required.
- Remote MCP OAuth → plan 112 after local stdio.

These residuals do **not** keep plan 109 open: they were out of scope for the
approved minimal slice.

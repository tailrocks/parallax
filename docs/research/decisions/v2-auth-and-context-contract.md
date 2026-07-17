# V2 authentication and named remote contexts (minimal)

- **Status:** Approved (minimal local-first slice)
- **Contract version:** 1
- **Decision date:** 2026-07-17
- **Approved by:** alexey@chainargos.com
- **Authority:** Operator unblock directive, 2026-07-17 — Plans 109/115 opened
  with the plan's recommended minimal shape; this record is the executable ADR
  for the first shippable auth surface.
- **Plan owner:** plan 109 (retired 2026-07-17; minimal slice shipped —
  [evidence](../validation/2026-07-plan-109-v2-auth/README.md)). Later auth
  expansions stay on this ADR / plan 115 / plan 112 as noted below.

## Decision

Parallax V1 remains **local-first and open by default on loopback**. The first
V2 authentication slice adds an optional **shared API bearer token** that
protects developer-facing API surfaces when configured. Named CLI contexts store
URL + token pairs so agents and humans can address a local or remote Parallax
without putting secrets on the command line.

This is deliberately smaller than multi-tenant RBAC, OAuth, or per-project
ingest tokens (those remain later plan-109 expansions and plan 115 server
profile work). The central rule is one: **one shared operator token, checked
once at the HTTP boundary**, with default-deny when the token is configured.

## Threat model (minimal slice)

| Actor | Goal we prevent |
| --- | --- |
| Local process on same host | Accidental use of a non-loopback bind without a token |
| Network peer on a shared LAN | Unauthenticated GraphQL/SSE when an API token is set |
| Log aggregator / crash dump / shell history | Token plaintext in argv, URL, ready banner, errors, telemetry |
| Malicious MCP/client | Not covered here for remote MCP; plan 112 stays local-stdio until this contract is used for remote |

Out of scope for contract v1:

- Multi-user RBAC, project-scoped capabilities, billing roles.
- Browser cookie/session persistence designs.
- OTLP ingest tokens (`x-parallax-project-token`) — plan 115.
- Remote MCP OAuth/PKCE/resource-indicators — plan 112 after this lands.
- rustls, custom root bundles, or plaintext remote credential transport over the public Internet without an operator-controlled TLS terminator.

## Credential format and lifecycle

| Field | Rule |
| --- | --- |
| Format | Opaque UTF-8 secret, 16–256 bytes after trim. Recommended generation: 32 random bytes, base64url or hex. |
| Presentation | HTTP `Authorization: Bearer <token>` only. No query string, no path, no GraphQL variable. |
| Issuer | Operator-configured. Sources (first match wins): process env `PARALLAX_API_TOKEN`, then `config.toml` `[server] api_token`. Empty/absent = auth disabled. |
| Storage (server) | Config file or environment. Config files under `~/.parallax/` should be mode `0600` on Unix. |
| Storage (CLI contexts) | `~/.parallax/contexts.toml` holds context `name`, `url`, and optional `token`. File created/updated atomically (temp + rename) with mode `0600`. Tokens are never written to the repository, argv, or default human list output (show masks). |
| Rotation | Replace the token in config/env and every context that holds it. No server-side revocation list in v1 — possession of the current token is authorization. |
| Expiry | None in v1. Operators rotate out-of-band. |
| Capability model | One capability: `operator` (full read/write of the protected developer API). Missing/invalid token → 401 with a secret-free body. |

## Protected surfaces

When an API token **is configured**:

| Surface | Auth |
| --- | --- |
| `POST /graphql` | Required bearer |
| `GET /v1/logs/stream`, `GET /v1/traces/stream` | Required bearer |
| `GET /health`, `GET /version` | Open (probes) |
| OTLP gRPC/HTTP | Open in v1 (ingest tokens deferred) |
| UI static assets | Open (browser will need a later session path) |

When an API token **is not configured**:

- Protected surfaces stay open (V1 local compatibility).
- **Hard stop:** if `server.bind` is not a loopback address (`127.0.0.1`, `::1`, or equivalent) and no API token is configured, configuration validation fails. Non-loopback exposure without a token is not a supported product mode.

## Named contexts (CLI)

Commands:

```text
parallax context add <name> --url <url> [--token <token> | --token-env <VAR>]
parallax context list
parallax context use <name>
parallax context show [<name>]
parallax context remove <name>
```

Rules:

- Names are non-empty, not `local` (reserved implicit open context → `http://127.0.0.1:4000` with no token unless env supplies one).
- `use` sets `current` atomically with the contexts file.
- `--token-env` stores the environment variable **name** as `token_env` and never materializes the secret into the file; runtime resolves `std::env::var`.
- `--token` writes the secret into `token` (file mode 0600). Prefer `--token-env` for shared machines and CI.
- Global `--context <name>` selects the named entry; absent → file `current` → implicit `local`.
- Client injects `Authorization: Bearer …` only when a resolved token exists.
- Errors never echo the token. GraphQL/SSE failures keep existing secret-free messaging.

## Transport and TLS

- Local loopback may use plaintext HTTP (current product default).
- Remote contexts should use `https://` terminated by the operator's reverse proxy or native TLS edge. Parallax clients keep **native TLS only** (`reqwest` `native-tls` / vendored OpenSSL on cross builds) — never rustls.
- Plan 115 owns server profile bind/TLS placement; this ADR only requires clients not enable a rustls backend and not put tokens in URLs.

## Audit (minimal)

- Successful and failed auth at the middleware boundary emit tracing events with `auth.result=ok|deny` and **no token material**.
- No durable audit table in contract v1 (may grow with server profile work).

## Compatibility

| Mode | Behavior |
| --- | --- |
| Existing local installs | Unchanged: no token, loopback, open GraphQL/SSE. |
| CI non-interactive | Set `PARALLAX_API_TOKEN` on server and clients, or context `token_env`. |
| MCP local stdio (plan 112) | Continues credential-free loopback until a product MCP remote path is authorized; then it consumes this bearer contract. |

## Acceptance matrix (machine-checkable intent)

1. No token + loopback → GraphQL 200 without `Authorization`.
2. Token configured + no header → GraphQL 401, body has no secret.
3. Token configured + wrong bearer → 401, constant-time compare path.
4. Token configured + correct bearer → GraphQL proceeds.
5. Non-loopback bind + no token → config load/validate fails.
6. Context add/list/use/show/remove atomic; file mode 0600 when created.
7. `context list` / `show` never print raw tokens (mask or omit).
8. CLI with context token sends `Authorization: Bearer`.
9. Ready banner names `auth off` or `auth bearer-token` without the secret.

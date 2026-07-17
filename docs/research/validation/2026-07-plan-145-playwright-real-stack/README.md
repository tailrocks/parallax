# Plan 145 — managed Greptime + Turso browser stack (2026-07-17)

## Evidence (live, this host)

| Item | Result |
| --- | --- |
| QA stack reuse | Greptime stayed on 24000–24003; `parallax serve` restarted with fixed binary; attach mode used |
| `PARALLAX_FULL_STACK_MODE=attach` | Seeds OTLP to `:4318`, GraphQL readiness on `:4000`, UI proxy on `:4175` |
| SSE proxy | `/v1/*/stream` forwarded as byte stream (not buffered) |
| `bun run test:browser:full` | **3 passed** (telemetry-discovery, storage-composition, live-transport) ~19s |
| Product bug fixed | `service_names` / overview metric buckets no longer use open-ended range width as `date_bin` step (clamped to 1h / max 1d) — unblocked IssuesList `services` field |

## Commands run

```bash
# after rebuilding parallax with greptime step clamp
./target/debug/parallax serve --config /tmp/parallax-qa/config.toml

cd ui && PARALLAX_FULL_STACK_MODE=attach \
  PARALLAX_FULL_STACK_BASE_URL=http://127.0.0.1:4000 \
  PARALLAX_FULL_STACK_OTLP_HTTP=http://127.0.0.1:4318 \
  bun run test:browser:full
# → 3 passed
```

## Landed

- `cargo xtask browser-full-stack-serve` + example harness (attach | managed)
- OTLP seed builders in `parallax-test-support::browser::real_stack`
- Playwright project `full-stack-chromium`, fixtures, three foundation specs
- Matrix foundation + reserved rows (134–143, 150)
- Policy `ui.browser-full-stack`

## Residual (plan stays active)

- Managed (non-attach) cold/warm lifecycle lifecycle tests when ports free
- Path-aware `browser-full-stack` CI job + scheduled storage workflow
- Duration ratchets / repeated-run harness
- Feature-owned reserved full-stack specs materialization (134–143/150)
- Full command table twice from clean state

Do **not** kill foreign Greptime by port alone; attach is the approved host path when 24000–24003 are occupied.

# Plan 112: Product MCP residual ship gates

> **Executor instructions**: Local-stdio GO only. Do not rename the spike as
> product without graduation, expose raw GraphQL/storage, add mutating tools,
> or enable repository auto-trust. Remote MCP waits on auth/TLS edge integration.

## Status

- **Priority**: P1
- **Effort**: L remaining
- **Risk**: CRITICAL
- **Depends on**: 099, 104, 111 (done); remote needs auth contract + TLS edge
- **Category**: agent surface / MCP / security
- **Status**: IN PROGRESS — local-stdio product GO; residual ship gates open
- **Evidence**:
  [`docs/research/validation/2026-07-plan-112-product-mcp/README.md`](../docs/research/validation/2026-07-plan-112-product-mcp/README.md)
  and
  [`docs/research/decisions/agent-access-surface.md`](../docs/research/decisions/agent-access-surface.md)

## Landed (do not replay)

Spike local-stdio: two read-only tools, closed schemas, loopback-only GraphQL,
`--allow-local-stdio` trust, `evidence:read`, secret-free errors, 128 KiB result
cap, 1 MiB GraphQL body, redaction/hash/schema fail-closed, audit row +
`parallax.mcp.audit` span (in-memory capture), wire init/`tools/list`/negative
capability fixtures, protocol pin, no rustls. Full crate tests + Clippy green
at evidence time.

## Residual only

1. **Claimed-client fixtures**: real Codex + Claude Code registration, trust,
   discovery, invocation, retention (not only in-process wire mock).
2. ~~**Oversized path**~~: bounded summary + approved `parallax://evidence/…`
   resource refs when wire JSON exceeds 128 KiB (landed).
3. **Independent OTel verification**: exporter/subscriber integration of audit
   spans outside unit capture.
4. **Spike disposition**: graduate deliberate product crate **or**
   delete/quarantine spike; remove comparison-only paths permanently.
5. Remote transport remains out of residual until a separate GO wires auth +
   native TLS + PKCE/audience/no-passthrough.

## Done Criteria

- [ ] Client fixtures pass for every claimed client.
- [x] Oversized output → bounded summary + approved resource refs.
- [ ] Every call produces safe audit + independently verified OTel evidence.
- [ ] Spike graduated or deleted/quarantined deliberately.
- [ ] Negative tool/capability/protocol fixtures remain permanently empty/fail-closed.

## STOP / Remove When

STOP if a client requires auto-trust, raw GraphQL, unbounded text, token
passthrough, rustls, or mutating tools. Delete after GO gates pass and spike
has terminal disposition, or NO-GO removes/quarantines the spike.

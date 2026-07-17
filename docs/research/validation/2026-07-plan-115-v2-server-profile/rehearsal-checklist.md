# Plan 115 — V2 single-node rehearsal checklist (partial 2026-07-17)

Contract: `docs/research/decisions/v2-server-profile.md`  
Example config: `example-config.toml` (loopback validates without secrets).

## Offline backup / restore (file-level)

1. Stop `parallax serve`.
2. Copy as one coherent set:
   - Greptime data dir under `storage.data_dir`
   - Turso `meta.db`
   - `config.toml` / contexts
3. Restore = stop → replace dirs → start; doctor until ready banner names all surfaces.

## Graceful drain

- SIGTERM / Ctrl-C: listeners abort, ingest workers drain (single-worker path).
- Ready banner must list API/UI, GraphQL, OTLP ports, storage mode, data dir.

## Non-loopback (operator TLS edge)

1. `server.bind = "0.0.0.0"` + `PARALLAX_API_TOKEN` (≥16 bytes).
2. Terminate TLS at OS/reverse proxy (native TLS only; never rustls product path).
3. Remote CLI: `--context` with URL + token; exercise `issue context`, `trace inspect`, `import-claude` against live API.

## Upgrade / rollback

- Upgrade: replace binary + config; managed Greptime via supervisor pin.
- Rollback: prior binary + restored data dirs from backup.

## Plan 110 gate

Do **not** open multi-worker concurrency until a load packet on this profile
proves the single worker is the bottleneck (not disk/network/Greptime).

## Residual for full plan 115 retirement

- [ ] Live non-loopback rehearsal evidence packet (operator TLS edge)
- [ ] Measured backup/restore + upgrade/rollback run log
- [ ] Disk-pressure + retention prune rehearsal log
- [ ] Four-target release artifact install dogfood
- [ ] Load packet for plan 110

# Resource-footprint harness

Measures `parallax serve` RSS (binary + Greptime child), CPU%, and data-dir
bytes at idle, light-steady ingest, and post-ingest idle. Scratch `HOME`
only — never the operator's `~/.parallax`.

```bash
./bench/footprint/measure.sh
./bench/footprint/check.sh
```

`PARALLAX_BIN` skips the release rebuild. `FOOTPRINT_IDLE_SECS`,
`FOOTPRINT_STEADY_SECS`, and `FOOTPRINT_POST_SECS` override phase lengths.
The contract is `contract.toml`; a tampered ceiling must fail `check.sh`.

Published numbers live in [`docs/guide/footprint.md`](../../docs/guide/footprint.md).

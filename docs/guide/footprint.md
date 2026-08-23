# Resource footprint

Measured 2026-08-13T23:16:01Z on **Darwin 25.5.0 arm64 Apple M5 Max** by
`bench/footprint/measure.sh`. Source report:
[`bench/footprint/report.json`](../../bench/footprint/report.json).

`parallax serve` is one Rust process plus one supervised GreptimeDB
child. The data-dir byte count includes the cached `greptime` binary.

| Phase | Parallax RSS | Greptime RSS | Combined | Data dir | CPU snapshot |
| --- | ---: | ---: | ---: | ---: | ---: |
| Idle after start (60s, no traffic) | 24 MiB | 139 MiB | 163 MiB | 465 MiB | 0% |
| Light steady (telemetrygen 5 traces/s × 120s) | 28 MiB | 183 MiB | 211 MiB | 473 MiB | 0% |
| Post-ingest idle (60s) | 28 MiB | 183 MiB | 211 MiB | 473 MiB | 0% |

CPU% is a single `ps` sample at the end of each phase, not an average.
These numbers are a laptop-class local profile, not a CI runner and not
a multi-tenant server.

Contract ceilings (2× this run) live in
[`bench/footprint/contract.toml`](../../bench/footprint/contract.toml).
Ceilings are lane-aware: `check.sh` applies `[<phase>.<lane>]` overrides
(from `FOOTPRINT_LANE`, set by CI to the runner lane) before the baseline
`[<phase>]` caps, because absolute RSS scales with the host (malloc arenas
and Greptime's Go heap grow with core count). The Velnor lane baseline is
[`bench/footprint/report.velnor.json`](../../bench/footprint/report.velnor.json).
The GitHub Actions job is warn-only until **2026-08-28**.

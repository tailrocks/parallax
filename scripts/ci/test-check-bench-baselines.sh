#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cat >"$tmp/samples.txt" <<'EOF'
Benchmarking arrow_decode_10k_rows_zstd: Analyzing
                        time:   [800.00 µs 810.00 µs 820.00 µs]
spool_append_4k         time:   [10.00 µs 11.00 µs 12.00 µs]
arrow_decode_10k_rows   time:   [600.00 µs 610.00 µs 620.00 µs]
Benchmarking normalize_metrics_1k_points: Analyzing
                        time:   [130.00 µs 140.00 µs 150.00 µs]
EOF

cat >"$tmp/allocation.txt" <<'EOF'
allocation-profile: 7011 allocations/call, 1022357 bytes/call
EOF

output="$(scripts/ci/check-bench-baselines.sh "$tmp/samples.txt" "$tmp/allocation.txt")"
grep -Fq 'ok normalize_metrics_1k_points: 140.000000us' <<<"$output"
grep -Fq 'ok arrow_decode_10k_rows_zstd: 810.000000us' <<<"$output"

cat >"$tmp/unnamed.txt" <<'EOF'
                        time:   [1.00 µs 2.00 µs 3.00 µs]
EOF
if scripts/ci/check-bench-baselines.sh "$tmp/unnamed.txt" >/dev/null 2>&1; then
  echo "unnamed Criterion time line unexpectedly accepted" >&2
  exit 1
fi

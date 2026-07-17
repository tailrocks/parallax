#!/usr/bin/env bash
# Plan 103: fail-closed bench + allocation ratchets.
# Compares criterion output and allocation-profile lines against committed
# ceilings. A regression FAILS — this script never rewrites baselines.
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <samples.txt> [allocation-profile.txt]" >&2
  exit 2
fi

samples_file="$1"
allocation_file="${2:-}"
baselines_file="docs/research/testing/bench-baselines.toml"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

if [[ ! -f "$baselines_file" ]]; then
  echo "missing baselines: $baselines_file" >&2
  exit 2
fi
if [[ ! -f "$samples_file" ]]; then
  echo "missing samples: $samples_file" >&2
  exit 2
fi

fail=0

# Parse criterion lines → "name|observed_us" (max mid per name across file).
# Criterion often omits the name on the first/last bench of a group:
# bare µs → normalize_metrics_1k_points; bare ms → arrow_decode_10k_rows_zstd.
parsed="$(mktemp)"
python3 - "$samples_file" >"$parsed" <<'PY'
import re, sys
from collections import defaultdict

time_re = re.compile(
    r"^\s*(?:(\S+)\s+)?time:\s*\[\s*[0-9.]+\s+\S+\s+([0-9.]+)\s+(\S+)",
    re.I,
)

def to_us(value: str, unit: str) -> float:
    unit = unit.replace("μ", "µ")
    if unit in ("µs", "us"):
        return float(value)
    if unit == "ms":
        return float(value) * 1000.0
    if unit == "ns":
        return float(value) / 1000.0
    if unit == "s":
        return float(value) * 1_000_000.0
    raise SystemExit(f"unknown unit: {unit!r}")

mids: dict[str, list[float]] = defaultdict(list)

for raw in open(sys.argv[1], encoding="utf-8", errors="replace"):
    m = time_re.search(raw)
    if not m:
        continue
    name, mid, unit = m.group(1), m.group(2), m.group(3)
    unit_norm = unit.replace("μ", "µ")
    if not name:
        name = (
            "normalize_metrics_1k_points"
            if unit_norm in ("µs", "us")
            else "arrow_decode_10k_rows_zstd"
        )
    mids[name].append(to_us(mid, unit_norm))

for name, values in sorted(mids.items()):
    print(f"{name}|{max(values):.6f}")
PY

while IFS='|' read -r name ceiling_us; do
  [ -z "$name" ] && continue
  observed="$(awk -F'|' -v n="$name" '$1==n {print $2; found=1} END{if(!found) exit 1}' "$parsed" 2>/dev/null || true)"
  if [ -z "$observed" ]; then
    echo "MISSING $name: no sample line found" >&2
    fail=1
    continue
  fi
  over=$(awk -v o="$observed" -v c="$ceiling_us" 'BEGIN { print (o > c + 0) ? 1 : 0 }')
  if [ "$over" = "1" ]; then
    echo "REGRESSION $name: observed ${observed}us exceeds ceiling ${ceiling_us}us — investigate; never refresh the baseline in this job" >&2
    fail=1
  else
    echo "ok $name: ${observed}us <= ${ceiling_us}us"
  fi
done < <(python3 - "$baselines_file" <<'EOF'
import sys, tomllib
data = tomllib.load(open(sys.argv[1], "rb"))
for row in data.get("bench", []):
    print(f"{row['name']}|{row['ceiling_us']}")
EOF
)

if [[ -n "$allocation_file" ]]; then
  if [[ ! -f "$allocation_file" ]]; then
    echo "MISSING allocation profile: $allocation_file" >&2
    fail=1
  else
    read -r max_allocs max_bytes < <(python3 - "$baselines_file" <<'EOF'
import sys, tomllib
data = tomllib.load(open(sys.argv[1], "rb"))
a = data["allocation"]
print(a["max_allocations_per_call"], a["max_bytes_per_call"])
EOF
)
    alloc_line="$(grep -E 'allocation-profile:' "$allocation_file" | tail -1 || true)"
    if [[ -z "$alloc_line" ]]; then
      echo "MISSING allocation-profile line in $allocation_file" >&2
      fail=1
    else
      observed_allocs="$(printf '%s\n' "$alloc_line" | sed -E 's/.*: ([0-9]+) allocations\/call.*/\1/')"
      observed_bytes="$(printf '%s\n' "$alloc_line" | sed -E 's/.*allocations\/call, ([0-9]+) bytes\/call.*/\1/')"
      if [[ "$observed_allocs" -gt "$max_allocs" ]]; then
        echo "REGRESSION allocation count: ${observed_allocs} > ${max_allocs} — investigate; never refresh the baseline in this job" >&2
        fail=1
      else
        echo "ok allocation count: ${observed_allocs} <= ${max_allocs}"
      fi
      if [[ "$observed_bytes" -gt "$max_bytes" ]]; then
        echo "REGRESSION allocation bytes: ${observed_bytes} > ${max_bytes} — investigate; never refresh the baseline in this job" >&2
        fail=1
      else
        echo "ok allocation bytes: ${observed_bytes} <= ${max_bytes}"
      fi
    fi
  fi
fi

rm -f "$parsed"
exit "$fail"

#!/usr/bin/env bash
# Compare bench/footprint/report.json to contract.toml (plan 175).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
REPORT="${1:-$ROOT/report.json}"
CONTRACT="${2:-$ROOT/contract.toml}"

[[ -f "$REPORT" ]] || { echo "missing report: $REPORT" >&2; exit 2; }
[[ -f "$CONTRACT" ]] || { echo "missing contract: $CONTRACT" >&2; exit 2; }

python3 - "$REPORT" "$CONTRACT" <<'PY'
import json, re, sys

report_path, contract_path = sys.argv[1], sys.argv[2]
report = json.loads(open(report_path, encoding="utf-8").read())
text = open(contract_path, encoding="utf-8").read()
section = None
ceilings = {}
for raw in text.splitlines():
    line = raw.split("#", 1)[0].strip()
    if not line:
        continue
    if line.startswith("[") and line.endswith("]"):
        section = line[1:-1]
        continue
    match = re.match(r"([A-Za-z0-9_]+)\s*=\s*([0-9.]+)", line)
    if match and section:
        ceilings[(section, match.group(1))] = float(match.group(2))

failed = []
for phase, metrics in report["phases"].items():
    for key, observed in metrics.items():
        ceiling = ceilings.get((phase, key))
        if ceiling is None:
            failed.append(f"{phase}.{key}: no ceiling")
            continue
        if float(observed) > ceiling:
            failed.append(
                f"{phase}.{key}: observed {observed} exceeds ceiling {ceiling:g}"
            )

if failed:
    print("footprint contract BREACH")
    for row in failed:
        print(f"  {row}")
    sys.exit(1)
print("footprint contract ok")
PY

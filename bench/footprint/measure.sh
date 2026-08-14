#!/usr/bin/env bash
# Measure Parallax serve RSS / CPU / data-dir (plan 175). Scratch HOME only.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT_DIR="${FOOTPRINT_OUT:-$ROOT/bench/footprint}"
REPORT="${OUT_DIR}/report.json"
IDLE_SECS="${FOOTPRINT_IDLE_SECS:-60}"
STEADY_SECS="${FOOTPRINT_STEADY_SECS:-120}"
POST_SECS="${FOOTPRINT_POST_SECS:-60}"
API_PORT="${FOOTPRINT_API_PORT:-18400}"
OTLP_HTTP="${FOOTPRINT_OTLP_HTTP:-18418}"
OTLP_GRPC="${FOOTPRINT_OTLP_GRPC:-18417}"

narrate() { printf '==> %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

sample_rss_kb() {
  local pid="$1"
  ps -o rss= -p "$pid" 2>/dev/null | awk '{print $1+0}'
}

sample_cpu() {
  local pid="$1"
  ps -o %cpu= -p "$pid" 2>/dev/null | awk '{print $1+0}'
}

dir_bytes() {
  du -sk "$1" 2>/dev/null | awk '{print $1*1024}'
}

wait_ready() {
  local url="$1" timeout="$2" start now
  start="$(date +%s)"
  while true; do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    now="$(date +%s)"
    if (( now - start > timeout )); then
      return 1
    fi
    sleep 0.5
  done
}

drive_otlp() {
  local secs="$1" end
  end=$((SECONDS + secs))
  if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    narrate "steady load: telemetrygen traces rate=5 duration=${secs}s"
    docker run --rm --network host \
      ghcr.io/open-telemetry/opentelemetry-collector-contrib/telemetrygen:v0.158.0 \
      traces --otlp-endpoint="127.0.0.1:${OTLP_GRPC}" --otlp-insecure \
      --rate=5 --duration="${secs}s" --service=footprint \
      --otlp-attributes=parallax.lab="1" || true
    return
  fi
  narrate "steady load: curl OTLP/HTTP (no docker telemetrygen)"
  while (( SECONDS < end )); do
    curl -sS -o /dev/null -X POST "http://127.0.0.1:${OTLP_HTTP}/v1/traces" \
      -H 'content-type: application/x-protobuf' --data-binary '' || true
    sleep 0.2
  done
}

sample_phase() {
  local name="$1"
  local parallax_pid="$2"
  local greptime_pid="$3"
  local data_dir="$4"
  local rss_p rss_g cpu bytes
  rss_p="$(sample_rss_kb "$parallax_pid")"
  if [[ -n "$greptime_pid" ]]; then
    rss_g="$(sample_rss_kb "$greptime_pid")"
  else
    rss_g=0
  fi
  cpu="$(sample_cpu "$parallax_pid")"
  bytes="$(dir_bytes "$data_dir")"
  printf '    %s: parallax_rss=%sKiB greptime_rss=%sKiB cpu=%s%% data_dir=%sB\n' \
    "$name" "$rss_p" "$rss_g" "$cpu" "$bytes" >&2
  printf '%s %s %s %s %s\n' "$name" "$rss_p" "$rss_g" "$cpu" "$bytes"
}

WORKDIR="$(mktemp -d "${TMPDIR:-/tmp}/parallax-footprint.XXXXXX")"
HOME_DIR="${WORKDIR}/home"
DATA_DIR="${HOME_DIR}/.parallax"
mkdir -p "$DATA_DIR" "$OUT_DIR"
cleanup() {
  if [[ -n "${SERVE_PID:-}" ]] && kill -0 "$SERVE_PID" 2>/dev/null; then
    narrate "stopping serve pid=${SERVE_PID}"
    kill "$SERVE_PID" 2>/dev/null || true
    wait "$SERVE_PID" 2>/dev/null || true
  fi
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

if [[ -x "${HOME}/.parallax/bin/greptime" ]]; then
  mkdir -p "${DATA_DIR}/bin"
  cp "${HOME}/.parallax/bin/greptime" "${DATA_DIR}/bin/greptime"
  chmod +x "${DATA_DIR}/bin/greptime"
  narrate "seeded greptime binary from operator cache (data dir still scratch)"
fi

narrate "scratch HOME=${HOME_DIR}"
if [[ -n "${PARALLAX_BIN:-}" ]]; then
  BIN="$PARALLAX_BIN"
else
  narrate "building release binary (cargo build --release -p parallax-cli)"
  (cd "$ROOT" && cargo build --release -p parallax-cli)
  BIN="${ROOT}/target/release/parallax"
fi
[[ -x "$BIN" ]] || die "binary not executable: $BIN"

cat >"${DATA_DIR}/config.toml" <<EOF
[server]
bind = "127.0.0.1"
api_port = ${API_PORT}
otlp_grpc_port = ${OTLP_GRPC}
otlp_http_port = ${OTLP_HTTP}

[storage]
mode = "managed"
data_dir = "${DATA_DIR}"
EOF

narrate "starting ${BIN} serve"
HOME="$HOME_DIR" "$BIN" serve --config "${DATA_DIR}/config.toml" \
  >"${WORKDIR}/serve.log" 2>&1 &
SERVE_PID=$!
if ! wait_ready "http://127.0.0.1:${API_PORT}/health" 180; then
  tail -n 80 "${WORKDIR}/serve.log" >&2 || true
  die "serve never became ready"
fi
narrate "ready pid=${SERVE_PID}"

GREPTIME_PID=""
if [[ -f "${DATA_DIR}/greptime.pid" ]]; then
  GREPTIME_PID="$(tr -d '[:space:]' <"${DATA_DIR}/greptime.pid")"
  narrate "greptime pid=${GREPTIME_PID}"
else
  narrate "greptime pidfile missing (engine may still be starting)"
fi

narrate "phase idle-after-start (${IDLE_SECS}s, no traffic)"
sleep "$IDLE_SECS"
if [[ -z "$GREPTIME_PID" && -f "${DATA_DIR}/greptime.pid" ]]; then
  GREPTIME_PID="$(tr -d '[:space:]' <"${DATA_DIR}/greptime.pid")"
fi
IDLE_LINE="$(sample_phase idle "$SERVE_PID" "$GREPTIME_PID" "$DATA_DIR")"

narrate "phase light-steady (${STEADY_SECS}s)"
drive_otlp "$STEADY_SECS"
STEADY_LINE="$(sample_phase steady "$SERVE_PID" "$GREPTIME_PID" "$DATA_DIR")"

narrate "phase post-ingest idle (${POST_SECS}s)"
sleep "$POST_SECS"
POST_LINE="$(sample_phase post_idle "$SERVE_PID" "$GREPTIME_PID" "$DATA_DIR")"

hw="$(uname -srm)"
if command -v sysctl >/dev/null 2>&1; then
  hw="${hw} $(sysctl -n machdep.cpu.brand_string 2>/dev/null || true)"
fi
measured_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

python3 - "$REPORT" "$measured_at" "$hw" "$IDLE_LINE" "$STEADY_LINE" "$POST_LINE" <<'PY'
import json, sys
path, measured_at, hardware, idle, steady, post = sys.argv[1:7]

def parse(line):
    name, rss_p, rss_g, cpu, disk = line.split()
    return {
        "parallax_rss_kb": int(float(rss_p)),
        "greptime_rss_kb": int(float(rss_g)),
        "cpu_pct": float(cpu),
        "data_dir_bytes": int(float(disk)),
    }

report = {
    "measured_at": measured_at,
    "hardware": hardware,
    "phases": {
        "idle": parse(idle),
        "steady": parse(steady),
        "post_idle": parse(post),
    },
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(report, fh, indent=2)
    fh.write("\n")
print(json.dumps(report, indent=2))
PY

narrate "wrote ${REPORT}"

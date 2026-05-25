#!/usr/bin/env bash
# Generate the standing 4-way benchmark dataset on ALL FOUR builds, identically.
# Data is generated natively in-engine (GreptimeDB range(), ClickHouse numbers()) — no CSV transport,
# fully reproducible. Tables: spans1m, m2m, logs1m, sj, errs, tsr (see docs/.../four-way-version-comparison.md).
#
# Usage:   bench/four-way/gen.sh            # default N=1_000_000 rows
#          N=5000000 bench/four-way/gen.sh  # bigger tier
# Prereq:  docker compose -f bench/compose.yml up -d   (all four builds healthy)
# Row-size policy (operator): keep N meaningful — minimum 50_000; default 1_000_000.
set -euo pipefail

N="${N:-1000000}"
(( N >= 50000 )) || { echo "N must be >= 50000 (meaningful tier); got $N" >&2; exit 1; }

GT_STABLE="${GT_STABLE:-parallax-bench-greptimedb-1}"
GT_NIGHTLY="${GT_NIGHTLY:-parallax-bench-greptimedb-nightly-1}"
CH_STABLE="${CH_STABLE:-parallax-bench-clickhouse-1}"
CH_HEAD="${CH_HEAD:-parallax-bench-clickhouse-head-1}"

gt(){ docker exec "$1" curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode "sql=$2" >/dev/null; }
ch(){ docker exec "$1" clickhouse-client -q "$2"; }

gen_gt(){ local c=$1; echo "  [GT] $c ..."
  # spans1m — event signal: low-card PK(service) + trace_id INVERTED + append_mode (Run 114 rule)
  gt "$c" "DROP TABLE IF EXISTS spans1m"
  gt "$c" "CREATE TABLE spans1m (\"ts\" TIMESTAMP(3) TIME INDEX,\"trace_id\" STRING INVERTED INDEX,\"span_id\" STRING,\"service\" STRING,\"duration_ms\" DOUBLE,\"status\" STRING,PRIMARY KEY(\"service\")) ENGINE=mito WITH (append_mode='true')"
  gt "$c" "INSERT INTO spans1m (\"ts\",\"trace_id\",\"span_id\",\"service\",\"duration_ms\",\"status\") SELECT (1716000000000+\"value\")::timestamp_ms, concat('t',cast(\"value\"%70000 as string)), concat('sp',cast(\"value\" as string)), concat('s',cast(\"value\"%12 as string)), (\"value\"%300)::double, CASE WHEN \"value\"%33=0 THEN 'error' ELSE 'ok' END FROM range(0,$N)"
  # m2m — metric signal: PK(service,instance) dedup (default), 40k series
  gt "$c" "DROP TABLE IF EXISTS m2m"
  gt "$c" "CREATE TABLE m2m (\"ts\" TIMESTAMP(3) TIME INDEX,\"service\" STRING,\"instance\" STRING,\"val\" DOUBLE,\"counter\" BIGINT,PRIMARY KEY(\"service\",\"instance\")) ENGINE=mito"
  gt "$c" "INSERT INTO m2m (\"ts\",\"service\",\"instance\",\"val\",\"counter\") SELECT (1716000000000+\"value\")::timestamp_ms, concat('s',cast(\"value\"%40 as string)), concat('i',cast(\"value\"%1000 as string)), (\"value\"%100)::double, \"value\" FROM range(0,$N)"
  # logs1m — log signal: PK(service) + message FULLTEXT(bloom) + append
  gt "$c" "DROP TABLE IF EXISTS logs1m"
  gt "$c" "CREATE TABLE logs1m (\"ts\" TIMESTAMP(3) TIME INDEX,\"service\" STRING,\"level\" STRING,\"message\" STRING FULLTEXT INDEX WITH(backend='bloom',analyzer='English',case_sensitive='false',false_positive_rate='0.01'),\"trace_id\" STRING,PRIMARY KEY(\"service\")) ENGINE=mito WITH (append_mode='true')"
  gt "$c" "INSERT INTO logs1m (\"ts\",\"service\",\"level\",\"message\",\"trace_id\") SELECT (1716000000000+\"value\")::timestamp_ms, concat('s',cast(\"value\"%12 as string)), CASE WHEN \"value\"%7=0 THEN 'ERROR' ELSE 'INFO' END, concat('request id=',cast(\"value\" as string),' path=/api/r',cast(\"value\"%50 as string),CASE WHEN \"value\"%7=0 THEN ' timeout error' ELSE ' ok' END), concat('t',cast(\"value\"%70000 as string)) FROM range(0,$N)"
  # sj — dynamic-attribute JSON
  gt "$c" "DROP TABLE IF EXISTS sj"
  gt "$c" "CREATE TABLE sj (\"ts\" TIMESTAMP(3) TIME INDEX,\"svc\" STRING,\"attributes\" JSON,PRIMARY KEY(\"svc\")) ENGINE=mito WITH (append_mode='true')"
  gt "$c" "INSERT INTO sj (\"ts\",\"svc\",\"attributes\") SELECT (1716000000000+\"value\")::timestamp_ms, concat('s',cast(\"value\"%12 as string)), parse_json(concat('{\"http\":{\"status_code\":',cast(200+(\"value\"%5)*100 as string),'}}')) FROM range(0,$N)"
  # errs — for cross-tier join (trace_id-keyed)
  gt "$c" "DROP TABLE IF EXISTS errs"
  gt "$c" "CREATE TABLE errs (\"ts\" TIMESTAMP(3) TIME INDEX,\"trace_id\" STRING INVERTED INDEX,\"fingerprint\" STRING,\"service\" STRING,PRIMARY KEY(\"service\")) ENGINE=mito WITH (append_mode='true')"
  gt "$c" "INSERT INTO errs (\"ts\",\"trace_id\",\"fingerprint\",\"service\") SELECT (1716000000000+\"value\")::timestamp_ms, concat('t',cast(\"value\"%70000 as string)), concat('fp',cast(\"value\"%300 as string)), concat('s',cast(\"value\"%12 as string)) FROM range(0,$N)"
  # tsr — time-range scan (varying ts)
  gt "$c" "DROP TABLE IF EXISTS tsr"
  gt "$c" "CREATE TABLE tsr (\"ts\" TIMESTAMP(3) TIME INDEX,\"svc\" STRING,\"v\" DOUBLE,PRIMARY KEY(\"svc\")) ENGINE=mito WITH (append_mode='true')"
  gt "$c" "INSERT INTO tsr (\"ts\",\"svc\",\"v\") SELECT (1716000000000+\"value\")::timestamp_ms, concat('s',cast(\"value\"%12 as string)), (\"value\"%100)::double FROM range(0,$N)"
  # flush so reads come from SST (settled state), not the memtable
  for t in spans1m m2m logs1m sj errs tsr; do gt "$c" "ADMIN flush_table('$t')"; done
}

gen_ch(){ local c=$1; echo "  [CH] $c ..."
  ch "$c" "DROP TABLE IF EXISTS spans1m"
  ch "$c" "CREATE TABLE spans1m (ts DateTime64(3),trace_id String,span_id String,service String,duration_ms Float64,status String) ENGINE=MergeTree ORDER BY (trace_id,ts)"
  ch "$c" "INSERT INTO spans1m SELECT now(),concat('t',toString(number%70000)),concat('sp',toString(number)),concat('s',toString(number%12)),(number%300)::Float64,if(number%33=0,'error','ok') FROM numbers($N)"
  ch "$c" "DROP TABLE IF EXISTS m2m"
  ch "$c" "CREATE TABLE m2m (ts DateTime64(3),service String,instance String,val Float64,counter Int64) ENGINE=MergeTree ORDER BY (service,instance)"
  ch "$c" "INSERT INTO m2m SELECT now(),concat('s',toString(number%40)),concat('i',toString(number%1000)),(number%100)::Float64,number FROM numbers($N)"
  ch "$c" "DROP TABLE IF EXISTS logs1m"
  ch "$c" "CREATE TABLE logs1m (ts DateTime64(3),service String,level String,message String,trace_id String, INDEX idx_msg message TYPE tokenbf_v1(32768,3,0) GRANULARITY 1) ENGINE=MergeTree ORDER BY (service,ts)"
  ch "$c" "INSERT INTO logs1m SELECT now(),concat('s',toString(number%12)),if(number%7=0,'ERROR','INFO'),concat('request id=',toString(number),' path=/api/r',toString(number%50),if(number%7=0,' timeout error',' ok')),concat('t',toString(number%70000)) FROM numbers($N)"
  ch "$c" "DROP TABLE IF EXISTS sj"
  ch "$c" "SET allow_experimental_json_type=1; CREATE TABLE sj (ts DateTime64(3),svc String,attributes JSON) ENGINE=MergeTree ORDER BY ts"
  ch "$c" "SET allow_experimental_json_type=1; INSERT INTO sj SELECT now(),concat('s',toString(number%12)),concat('{\"http\":{\"status_code\":',toString(200+(number%5)*100),'}}') FROM numbers($N)"
  ch "$c" "DROP TABLE IF EXISTS errs"
  ch "$c" "CREATE TABLE errs (ts DateTime64(3),trace_id String,fingerprint String,service String) ENGINE=MergeTree ORDER BY (trace_id,ts)"
  ch "$c" "INSERT INTO errs SELECT now(),concat('t',toString(number%70000)),concat('fp',toString(number%300)),concat('s',toString(number%12)) FROM numbers($N)"
  ch "$c" "DROP TABLE IF EXISTS tsr"
  ch "$c" "CREATE TABLE tsr (ts DateTime64(3),svc String,v Float64) ENGINE=MergeTree ORDER BY ts"
  ch "$c" "INSERT INTO tsr SELECT fromUnixTimestamp64Milli(toInt64(1716000000000+number)),concat('s',toString(number%12)),(number%100)::Float64 FROM numbers($N)"
}

echo "Generating 4-way dataset (N=$N rows) on all four builds..."
gen_gt "$GT_STABLE"; gen_gt "$GT_NIGHTLY"; gen_ch "$CH_STABLE"; gen_ch "$CH_HEAD"
echo "Done. Verify: bench/four-way/bench.sh"

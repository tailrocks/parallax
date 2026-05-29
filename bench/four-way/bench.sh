#!/usr/bin/env bash
# Run the 4-way benchmark suite across ALL FOUR builds and print the matrix (median ms per cell).
# Encodes every query in docs/.../four-way-version-comparison.md. Per-engine SQL where dialects differ
# (count(*) vs count(); date_bin vs toStartOfInterval; last_value vs argMax; approx_percentile_cont vs
# quantile; matches_term vs hasToken; json_get_int vs .:Int64 cast).
#
# Usage:  bench/four-way/bench.sh            # REPS=6 warm reps, median
#         REPS=10 bench/four-way/bench.sh
# Prereq: docker compose -f bench/compose.yml up -d  &&  bench/four-way/gen.sh
set -euo pipefail

GT_STABLE="${GT_STABLE:-parallax-bench-greptimedb-1}"
GT_NIGHTLY="${GT_NIGHTLY:-parallax-bench-greptimedb-nightly-1}"
CH_STABLE="${CH_STABLE:-parallax-bench-clickhouse-1}"
CH_HEAD="${CH_HEAD:-parallax-bench-clickhouse-head-1}"
REPS="${REPS:-6}"

gtmed(){ local c=$1 sql=$2; { for ((i=0;i<REPS;i++)); do docker exec "$c" curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode "sql=$sql" 2>/dev/null | python3 -c "import sys,json;print(json.load(sys.stdin).get('execution_time_ms',-1))" 2>/dev/null; done; } | python3 -c "import sys;v=sorted(float(x) for x in sys.stdin.read().split() if x and float(x)>=0);print(round(v[len(v)//2]) if v else 'ERR')"; }
chmed(){ local c=$1 sql=$2; { for ((i=0;i<REPS;i++)); do docker exec "$c" clickhouse-client -q "$sql FORMAT Null" --time 2>&1 | grep -E '^[0-9]'; done; } | python3 -c "import sys;v=sorted(float(x)*1000 for x in sys.stdin.read().split() if x);print(round(v[len(v)//2]) if v else 'ERR')"; }

row(){ # 1=name 2=GT_SQL 3=CH_SQL
  printf "%-40s | GT-stable %5sms | GT-nightly %5sms | CH-stable %5sms | CH-head %5sms\n" \
    "$1" "$(gtmed "$GT_STABLE" "$2")" "$(gtmed "$GT_NIGHTLY" "$2")" "$(chmed "$CH_STABLE" "$3")" "$(chmed "$CH_HEAD" "$3")"
}

echo "4-way benchmark matrix (median of $REPS warm reps, ms). All queries < 300ms gate = interactive."
echo "GT=GreptimeDB v1.0.2/v1.1-nightly  CH=ClickHouse 26.5/26.6-head"
echo "-------------------------------------------------------------------------------------------------"
# --- spans (anchored, scan, topk, trace-explorer) ---
row "anchored-lookup(trace_id)"          "SELECT count(*) FROM spans1m WHERE trace_id='t12345'"                                                  "SELECT count() FROM spans1m WHERE trace_id='t12345'"
row "unindexed-scan(span_id)"            "SELECT count(*) FROM spans1m WHERE span_id='sp500000'"                                                 "SELECT count() FROM spans1m WHERE span_id='sp500000'"
row "topk(order-by-limit)"               "SELECT trace_id,duration_ms FROM spans1m ORDER BY duration_ms DESC LIMIT 10"                           "SELECT trace_id,duration_ms FROM spans1m ORDER BY duration_ms DESC LIMIT 10"
row "trace-explorer(err+dur sort)"       "SELECT trace_id,service,duration_ms FROM spans1m WHERE status='error' AND duration_ms>250 ORDER BY duration_ms DESC LIMIT 50" "SELECT trace_id,service,duration_ms FROM spans1m WHERE status='error' AND duration_ms>250 ORDER BY duration_ms DESC LIMIT 50"
row "high-group-agg(GROUP BY trace_id)"  "SELECT trace_id,count(*) c,avg(duration_ms) FROM spans1m GROUP BY trace_id ORDER BY c DESC LIMIT 50"  "SELECT trace_id,count() c,avg(duration_ms) FROM spans1m GROUP BY trace_id ORDER BY c DESC LIMIT 50"
row "count-distinct(trace_id 70k)"       "SELECT count(distinct trace_id) FROM spans1m"                                                         "SELECT count(distinct trace_id) FROM spans1m"
row "count-distinct-highcard(span_id 1M)" "SELECT count(distinct span_id) FROM spans1m"                                                         "SELECT count(distinct span_id) FROM spans1m"
row "latency-histogram(30 buckets)"      "SELECT floor(duration_ms/10)*10 b,count(*) FROM spans1m GROUP BY b ORDER BY b"                         "SELECT floor(duration_ms/10)*10 b,count() FROM spans1m GROUP BY b ORDER BY b"
# --- metrics (m2m) ---
row "metric-agg-flat"                    "SELECT service,avg(val) FROM m2m GROUP BY service"                                                    "SELECT service,avg(val) FROM m2m GROUP BY service"
row "metric-bucketed-line"               "SELECT date_bin('1 minute'::INTERVAL,ts) m,service,avg(val) FROM m2m GROUP BY m,service"              "SELECT toStartOfInterval(ts,INTERVAL 1 MINUTE) m,service,avg(val) FROM m2m GROUP BY m,service"
row "counter-rate-panel"                 "SELECT date_bin('5 minutes'::INTERVAL,ts) m,service,max(counter)-min(counter) FROM m2m GROUP BY m,service" "SELECT toStartOfInterval(ts,INTERVAL 5 MINUTE) m,service,max(counter)-min(counter) FROM m2m GROUP BY m,service"
row "last-value(current)"                "SELECT service,last_value(val ORDER BY ts) FROM m2m GROUP BY service"                                  "SELECT service,argMax(val,ts) FROM m2m GROUP BY service"
row "latency-p99-by-service"             "SELECT service,approx_percentile_cont(duration_ms,0.99) FROM spans1m GROUP BY service"                 "SELECT service,quantile(0.99)(duration_ms) FROM spans1m GROUP BY service"
# --- logs (logs1m) ---
row "fulltext-selective(1 row)"          "SELECT count(*) FROM logs1m WHERE matches_term(message,'777777')"                                      "SELECT count() FROM logs1m WHERE hasToken(message,'777777')"
row "fulltext-broad(~14%)"               "SELECT count(*) FROM logs1m WHERE matches_term(message,'timeout')"                                     "SELECT count() FROM logs1m WHERE hasToken(message,'timeout')"
row "log-tail(service,ts DESC)"          "SELECT ts,level,message FROM logs1m WHERE service='s3' ORDER BY ts DESC LIMIT 100"                     "SELECT ts,level,message FROM logs1m WHERE service='s3' ORDER BY ts DESC LIMIT 100"
# --- errors / json / join / time-range ---
row "issue-list(GROUP BY fingerprint)"   "SELECT fingerprint,count(*) c,max(ts) FROM errs GROUP BY fingerprint ORDER BY c DESC LIMIT 50"        "SELECT fingerprint,count() c,max(ts) FROM errs GROUP BY fingerprint ORDER BY c DESC LIMIT 50"
row "dynamic-attr-json(cast)"            "SELECT json_get_int(\"attributes\",'http.status_code') sc,count(*) FROM sj GROUP BY sc"                "SELECT attributes.http.status_code.:Int64 sc,count() FROM sj GROUP BY sc"
row "cross-tier-join(anchored)"          "SELECT count(*) FROM spans1m s JOIN errs e ON s.trace_id=e.trace_id WHERE s.trace_id='t12345'"        "SELECT count() FROM spans1m s JOIN errs e ON s.trace_id=e.trace_id WHERE s.trace_id='t12345'"
row "time-range-scan(100k window)"       "SELECT count(*) FROM tsr WHERE ts BETWEEN 1716000900000::timestamp_ms AND 1716001000000::timestamp_ms" "SELECT count() FROM tsr WHERE ts BETWEEN fromUnixTimestamp64Milli(1716000900000) AND fromUnixTimestamp64Milli(1716001000000)"
echo "-------------------------------------------------------------------------------------------------"
echo "Update docs/research/storage/greptimedb-vs-clickhouse/four-way-version-comparison.md with these medians."

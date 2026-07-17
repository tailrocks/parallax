#!/usr/bin/env bash
# Plan 159 machine assertions over the live Parallax GraphQL surface.
#
# Usage:
#   DRIVE_ID=… CRON_ID=… CONSOLE_ID=… DAEMON_ID=… \
#   JHAPPY_ID=… JERROR_ID=… JOUTSIDE_ID=… JREATTACH_ID=… \
#   JPAR_IDS="id1 id2 id3" WRAPPED_ID=… bash assert.sh
#
# Every assertion prints PASS/FAIL and stores its raw JSON next to this
# script under assert-outputs/. Exit code = number of failures.
set -u

GQL="${GQL:-http://127.0.0.1:4000/graphql}"
OUT="$(dirname "$0")/assert-outputs"
mkdir -p "$OUT"
FAILURES=0
NOW_NS="$(($(date +%s) * 1000000000))"
FROM_NS="$((NOW_NS - 4 * 3600 * 1000000000))"

q() { # name, query
  local name="$1" query="$2"
  curl -sf "$GQL" -H 'content-type: application/json' \
    -d "$(jq -n --arg q "$query" '{query: $q}')" | tee "$OUT/$name.json"
}

check() { # name, jq-predicate, json
  local name="$1" pred="$2" json="$3"
  if jq -e "$pred" >/dev/null 2>&1 <<<"$json"; then
    echo "PASS $name"
  else
    echo "FAIL $name (predicate: $pred)"
    FAILURES=$((FAILURES + 1))
  fi
}

# ── 1. Four CLI modes with appMode + terminal outcome ──────────────────────
for pair in "one_shot:$DRIVE_ID:drive" "one_shot:$CRON_ID:cron" \
            "interactive:$CONSOLE_ID:console" "daemon:$DAEMON_ID:daemon"; do
  mode="${pair%%:*}"; rest="${pair#*:}"; id="${rest%%:*}"; label="${rest##*:}"
  json=$(q "1-mode-$label" "{ invocation(invocationId: \"$id\") { invocationId appMode status outcome } observedInvocations(limit: 200) { invocationId appMode } }")
  check "1.$label observed appMode=$mode" \
    ".data.observedInvocations[] | select(.invocationId == \"$id\") | select(.appMode == \"$mode\")" "$json"
  if [ "$mode" != "daemon" ]; then
    check "1.$label terminal outcome" \
      '.data.invocation | .status == "finished" and .outcome == "success"' "$json"
  fi
done

# ── 2. Console run: sessions, screens, actions, cross-service trace ────────
json=$(q "2-console" "{ sessions(invocationId: \"$CONSOLE_ID\") { sessionId startNanos endNanos } screenVisits(invocationId: \"$CONSOLE_ID\") { screenId navigationSequence enteredNanos exitedNanos } uiActions(invocationId: \"$CONSOLE_ID\") { name traceId outcome } }")
check "2.sessions one closed pair" \
  '.data.sessions | length == 1 and .[0].endNanos != null' "$json"
check "2.screenVisits >=2 strictly increasing" \
  '[.data.screenVisits[].navigationSequence] | length >= 2 and . == (sort | unique)' "$json"
check "2.uiActions >=2 incl checkout.submit" \
  '(.data.uiActions | length >= 2) and ([.data.uiActions[] | select(.name == "checkout.submit")] | length >= 1)' "$json"
SUBMIT_TRACE=$(jq -r '[.data.uiActions[] | select(.name == "checkout.submit")][0].traceId' <<<"$json")
json=$(q "2-console-trace" "{ trace(traceId: \"$SUBMIT_TRACE\") { spans { service } } }")
check "2.checkout.submit trace crosses into checkout service" \
  '[.data.trace.spans[].service] | index("checkout") != null' "$json"

# ── 3. Daemon background cycles ─────────────────────────────────────────────
json=$(q "3-cycles" "{ backgroundCycles(invocationId: \"$DAEMON_ID\", fromNanos: \"$FROM_NS\", toNanos: \"$NOW_NS\") { name count } }")
check "3.daemon cycles >=1 name with count>=1" \
  '.data.backgroundCycles | length >= 1 and (.[0].count >= 1)' "$json"

# ── 4. Jobs: order_dispatch + fulfillment_shipment with shared jobId ───────
json=$(q "4-jobs" "{ jobs(fromNanos: \"$FROM_NS\", toNanos: \"$NOW_NS\") { jobId jobType attempts { traceId outcome } } }")
check "4.order_dispatch job present" \
  '[.data.jobs[] | select(.jobType == "order_dispatch")] | length >= 1' "$json"
check "4.fulfillment_shipment job present" \
  '[.data.jobs[] | select(.jobType == "fulfillment_shipment")] | length >= 1' "$json"
FJOB=$(jq -r '[.data.jobs[] | select(.jobType == "fulfillment_shipment")][0].jobId // empty' <<<"$json")
if [ -n "$FJOB" ]; then
  # The job id must ride the Kafka hop: the producing request span (SERVER)
  # and the CONSUMER attempt span carry the same job.id on both sides of the
  # broker — two distinct span kinds sharing one job id prove the handoff.
  sqlq="SELECT DISTINCT span_kind FROM opentelemetry_traces WHERE \"span_attributes.job.id\" = '$FJOB'"
  json=$(curl -sf "$GQL" -H 'content-type: application/json' \
    -d "$(jq -n --arg q "$sqlq" '{query: ("{ sql(query: " + ($q | tojson) + ") { rows } }")}')")
  echo "$json" > "$OUT/4-jobs-cross.json"
  check "4.shipment jobId crosses the Kafka hop (two span kinds share it)" \
    '.data.sql.rows | length >= 2' "$json"
fi

# ── 5. Daemon conversations ─────────────────────────────────────────────────
json=$(q "5-conversations" "{ conversations(invocationId: \"$DAEMON_ID\") { conversationId agentName providerName } }")
check "5.daemon conversation with agent+provider" \
  '.data.conversations | length >= 1 and (.[0].agentName != null) and (.[0].providerName != null)' "$json"

# ── 6. Drive run signals + service map kinds ────────────────────────────────
json=$(q "6-drive" "{ logsByInvocation(invocationId: \"$DRIVE_ID\", limit: 5) { body } tracesByInvocation(invocationId: \"$DRIVE_ID\") { traceId } serviceMap(fromNanos: \"$FROM_NS\", toNanos: \"$NOW_NS\") { nodes { kind } } }")
check "6.drive logs non-empty" '.data.logsByInvocation | length >= 1' "$json"
check "6.drive traces non-empty" '.data.tracesByInvocation | length >= 1' "$json"
check "6.serviceMap kinds cli+browser+service" \
  '[.data.serviceMap.nodes[].kind] | (index("cli") != null) and (index("browser") != null) and (index("service") != null)' "$json"

# ── 7. Journeys ──────────────────────────────────────────────────────────────
json=$(q "7-jerror" "{ screenVisits(invocationId: \"$JERROR_ID\") { screenId enteredNanos exitedNanos } invocation(invocationId: \"$JERROR_ID\") { errorEvents { tsNanos title } } uiActions(invocationId: \"$JERROR_ID\") { name screenId widgetName outcome } }")
check "7.j-error failure inside checkout visit with widget context" \
  '(.data.uiActions[] | select(.name == "checkout.submit")) as $a
   | $a.outcome == "error" and $a.screenId == "checkout" and $a.widgetName == "checkout.submit.button"' "$json"
check "7.j-error error event timestamp inside checkout visit" \
  '(.data.screenVisits[] | select(.screenId == "checkout")) as $v
   | [.data.invocation.errorEvents[]
      | select((.tsNanos | tonumber) >= ($v.enteredNanos | tonumber)
           and (.tsNanos | tonumber) <= ($v.exitedNanos | tonumber))]
   | length >= 1' "$json"

json=$(q "7-joutside" "{ screenVisits(invocationId: \"$JOUTSIDE_ID\") { screenId enteredNanos exitedNanos } invocation(invocationId: \"$JOUTSIDE_ID\") { errorEvents { tsNanos title } } }")
check "7.j-outside error resolves to no visit" \
  '([.data.invocation.errorEvents[] | select(.title | test("BetweenScreens"))][0].tsNanos | tonumber) as $t
   | [.data.screenVisits[]
      | select(($t >= (.enteredNanos | tonumber))
           and (.exitedNanos != null and $t <= (.exitedNanos | tonumber)))]
   | length == 0' "$json"

json=$(q "7-jreattach" "{ sessions(invocationId: \"$JREATTACH_ID\") { sessionId previousSessionId } }")
check "7.j-reattach >=3-link previous chain" \
  '(.data.sessions | length >= 3) and ([.data.sessions[] | select(.previousSessionId != null)] | length >= 2)' "$json"

PAR_OK=1
for id in $JPAR_IDS; do
  json=$(q "7-jpar-$id" "{ logsByInvocation(invocationId: \"$id\", limit: 200) { invocationId } }")
  jq -e --arg id "$id" '[.data.logsByInvocation[] | select(.invocationId != null and .invocationId != $id)] | length == 0' >/dev/null <<<"$json" || PAR_OK=0
done
if [ "$PAR_OK" = 1 ]; then echo "PASS 7.j-parallel non-bleeding signal sets"; else echo "FAIL 7.j-parallel"; FAILURES=$((FAILURES+1)); fi

# ── 8. Negative: legacy contract rejected ───────────────────────────────────
legacy=$(curl -s "$GQL" -H 'content-type: application/json' \
  -d '{"query":"{ run(runId: \"x\") { runId } }"}')
echo "$legacy" > "$OUT/8-legacy.json"
check "8.legacy run(runId:) fails schema validation" \
  '.errors | length >= 1' "$legacy"
# No corpus signal carries the legacy keys. The hand-posted legacy probe
# (service.name=legacy-probe) may have widened the column; every row that
# carries the key must belong to that probe alone. A missing column (sql
# error "No field named") is the strongest pass.
legacy_sql="SELECT count(*) FROM opentelemetry_traces WHERE \"span_attributes.parallax.run.id\" IS NOT NULL AND service_name != 'legacy-probe'"
legacy_rows=$(curl -s "$GQL" -H 'content-type: application/json' \
  -d "$(jq -n --arg q "$legacy_sql" '{query: ("{ sql(query: " + ($q | tojson) + ") { rows } }")}')")
echo "$legacy_rows" > "$OUT/8-no-legacy-signals.json"
if jq -e '(.errors[0].message // "") | test("No field named")' >/dev/null 2>&1 <<<"$legacy_rows" \
   || jq -e '.data.sql.rows[0] | fromjson | .[0] == 0' >/dev/null 2>&1 <<<"$legacy_rows"; then
  echo "PASS 8.no corpus signal carries parallax.run.id"
else
  echo "FAIL 8.no corpus signal carries parallax.run.id"; FAILURES=$((FAILURES+1))
fi
json=$(q "8-legacy-only-invocations" "{ observedInvocations(limit: 500) { invocationId } }")
check "8.legacy-only span minted no invocation" \
  '[.data.observedInvocations[] | select(.invocationId == "legacy-run-only")] | length == 0' "$json"

echo
echo "assert.sh failures: $FAILURES"
exit "$FAILURES"

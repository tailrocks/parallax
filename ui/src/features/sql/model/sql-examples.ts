import { CLI_INVOCATION_ID } from "@/shared/semconv"

export type SqlExample = {
  readonly label: string
  readonly sql: string
}

export const SQL_EXAMPLES: readonly SqlExample[] = [
  {
    label: "Slow spans + error logs",
    sql: `SELECT s."timestamp", s.service_name, s.span_name,
       s.duration_nano / 1000000 AS ms,
       l.severity_text, l.body
FROM opentelemetry_traces s
JOIN opentelemetry_logs l ON l.trace_id = s.trace_id
WHERE s.duration_nano > 10000000 AND l.severity_number >= 17
ORDER BY s."timestamp" DESC LIMIT 50`,
  },
  {
    label: "Error events per service",
    sql: `SELECT service, error_type, count(*) AS events
FROM error_events
WHERE ts >= now() - INTERVAL '1 hour'
GROUP BY service, error_type
ORDER BY events DESC`,
  },
  {
    label: "Log volume by severity",
    sql: `SELECT severity_text, count(*) AS lines
FROM opentelemetry_logs
WHERE "timestamp" >= now() - INTERVAL '1 hour'
GROUP BY severity_text
ORDER BY lines DESC`,
  },
  {
    label: "Invocation cross-section",
    sql: `SELECT 'span linked to invocation log' AS signal, count(DISTINCT s.span_id) AS rows
FROM opentelemetry_traces s
JOIN opentelemetry_logs l ON l.trace_id = s.trace_id
WHERE l."${CLI_INVOCATION_ID}" = '<invocation-id>'
UNION ALL
SELECT 'log', count(*)
FROM opentelemetry_logs
WHERE "${CLI_INVOCATION_ID}" = '<invocation-id>'
UNION ALL
SELECT 'metric point', count(*)
FROM invocation_metric_points
WHERE invocation_id = '<invocation-id>'`,
  },
  {
    label: "Slowest root spans",
    sql: `SELECT span_name, service_name, count(*) AS calls,
       max(duration_nano) / 1000000 AS worst_ms,
       avg(duration_nano) / 1000000 AS avg_ms
FROM opentelemetry_traces
WHERE parent_span_id IS NULL OR parent_span_id = ''
GROUP BY span_name, service_name
ORDER BY max(duration_nano) DESC LIMIT 25`,
  },
]

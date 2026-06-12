import { useState } from "react"
import { createFileRoute, Link } from "@tanstack/react-router"
import { graphql } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"
import { MetricStrip } from "@/components/metric-strip"

interface TraceSpan {
  tsNanos: string
  service: string
  name: string
  kind: string
  statusCode: string
  statusMessage: string
  durationNs: string
  spanId: string
  parentSpanId: string | null
  runId: string | null
  links: string
  attributes: string
}

interface TraceLog {
  tsNanos: string
  service: string
  severityText: string
  body: string
  spanId: string
}

export const Route = createFileRoute("/traces/$traceId")({
  loader: ({ params }) =>
    graphql<{ trace: { spans: TraceSpan[] } | null; logsByTrace: TraceLog[] }>(
      `{ trace(traceId: "${params.traceId}") {
           spans { tsNanos service name kind statusCode statusMessage durationNs
                   spanId parentSpanId runId links attributes }
         }
         logsByTrace(traceId: "${params.traceId}") { tsNanos service severityText body spanId } }`
    ),
  component: TracePage,
})

function parseAttributes(json: string): [string, string][] {
  try {
    const value = JSON.parse(json) as Record<string, unknown>
    return Object.entries(value).map(([key, v]) => [
      key,
      typeof v === "string" ? v : JSON.stringify(v),
    ])
  } catch {
    return []
  }
}

interface SpanLink {
  traceId: string
  spanId: string
}

function parseLinks(json: string): SpanLink[] {
  try {
    const value: unknown = JSON.parse(json)
    return Array.isArray(value)
      ? (value as SpanLink[]).filter((link) => link.traceId)
      : []
  } catch {
    return []
  }
}

/** Waterfall with a clickable span detail pane (attributes, span logs). */
function TracePage() {
  const { trace, logsByTrace } = Route.useLoaderData()
  const { traceId } = Route.useParams()
  const [selectedId, setSelectedId] = useState<string | null>(null)
  if (!trace) {
    return <p className="text-sm text-muted-foreground">Trace not found.</p>
  }
  const spans = trace.spans
  const start = Math.min(...spans.map((s) => Number(s.tsNanos)))
  const end = Math.max(
    ...spans.map((s) => Number(s.tsNanos) + Number(s.durationNs))
  )
  const total = Math.max(1, end - start)
  const selected = spans.find((s) => s.spanId === selectedId) ?? null
  const selectedAttributes = selected
    ? parseAttributes(selected.attributes)
    : []
  const dbQuery = selectedAttributes.find(([key]) => key === "db.query.text")
  const selectedLogs = selected
    ? logsByTrace.filter((log) => log.spanId === selected.spanId)
    : []
  const selectedLinks = selected ? parseLinks(selected.links) : []
  const runId = spans.find((s) => s.runId)?.runId ?? null

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-lg font-semibold">
          Trace <code className="text-base">{traceId}</code>
        </h1>
        {runId ? (
          <Button asChild variant="outline" size="sm">
            <Link to="/runs/$runId" params={{ runId }}>
              run {runId}
            </Link>
          </Button>
        ) : null}
      </div>
      <div className={selected ? "grid gap-4 lg:grid-cols-[1fr_24rem]" : ""}>
        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">
                {spans.length} span(s) · {(total / 1e6).toFixed(1)}ms — click a
                span for attributes
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-1.5">
              {spans.map((span) => {
                const offset = ((Number(span.tsNanos) - start) / total) * 100
                const width = Math.max(
                  0.5,
                  (Number(span.durationNs) / total) * 100
                )
                const failed = span.statusCode === "STATUS_CODE_ERROR"
                const active = span.spanId === selectedId
                return (
                  <button
                    key={span.spanId}
                    type="button"
                    onClick={() => setSelectedId(active ? null : span.spanId)}
                    className={`block w-full space-y-0.5 rounded px-1 py-0.5 text-left hover:bg-muted/60 ${
                      active ? "bg-muted" : ""
                    }`}
                  >
                    <div className="flex items-center justify-between gap-2 text-xs">
                      <span className="truncate">
                        <Badge variant="outline" className="mr-1">
                          {span.service}
                        </Badge>
                        {span.name}
                        {parseLinks(span.links).length > 0 ? (
                          <Badge variant="secondary" className="ml-1">
                            ↗ {parseLinks(span.links).length} linked
                          </Badge>
                        ) : null}
                      </span>
                      <span className="shrink-0 text-muted-foreground tabular-nums">
                        {(Number(span.durationNs) / 1e6).toFixed(2)}ms
                      </span>
                    </div>
                    <div className="h-2 w-full rounded bg-muted">
                      <div
                        className={`h-2 rounded ${failed ? "bg-destructive" : "bg-primary"}`}
                        style={{ marginLeft: `${offset}%`, width: `${width}%` }}
                      />
                    </div>
                  </button>
                )
              })}
            </CardContent>
          </Card>

          {logsByTrace.length > 0 ? (
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">Correlated logs</CardTitle>
              </CardHeader>
              <CardContent>
                <ul className="space-y-1 font-mono text-xs">
                  {logsByTrace.map((log, index) => (
                    <li key={index} className="flex gap-2">
                      <span className="shrink-0 text-muted-foreground">
                        {log.severityText}
                      </span>
                      <span className="break-all">{log.body}</span>
                    </li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          ) : null}

          <MetricStrip
            title="Metrics around this trace"
            service={spans[0]?.service}
            runId={runId ?? undefined}
            fromNanos={(BigInt(start) - 300_000_000_000n).toString()}
            toNanos={(BigInt(end) + 300_000_000_000n).toString()}
            stepSeconds={30}
          />
        </div>

        {selected ? (
          <Card className="h-fit lg:sticky lg:top-4">
            <CardHeader>
              <CardTitle className="flex items-center justify-between gap-2 text-sm">
                <span className="truncate">{selected.name}</span>
                <Badge
                  variant={
                    selected.statusCode === "STATUS_CODE_ERROR"
                      ? "destructive"
                      : "secondary"
                  }
                >
                  {selected.statusCode.replace("STATUS_CODE_", "")}
                </Badge>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-xs">
              <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
                <dt className="text-muted-foreground">service</dt>
                <dd>{selected.service}</dd>
                <dt className="text-muted-foreground">kind</dt>
                <dd>{selected.kind.replace("SPAN_KIND_", "").toLowerCase()}</dd>
                <dt className="text-muted-foreground">duration</dt>
                <dd className="tabular-nums">
                  {(Number(selected.durationNs) / 1e6).toFixed(3)}ms
                </dd>
                <dt className="text-muted-foreground">span id</dt>
                <dd className="font-mono break-all">{selected.spanId}</dd>
                {selected.statusMessage ? (
                  <>
                    <dt className="text-muted-foreground">status</dt>
                    <dd className="break-all">{selected.statusMessage}</dd>
                  </>
                ) : null}
              </dl>

              {dbQuery ? (
                <div>
                  <p className="mb-1 font-medium text-muted-foreground">
                    db.query.text
                  </p>
                  <pre className="overflow-x-auto rounded bg-muted p-2 font-mono">
                    {dbQuery[1]}
                  </pre>
                </div>
              ) : null}

              {selectedAttributes.length > 0 ? (
                <div>
                  <Separator className="my-2" />
                  <p className="mb-1 font-medium text-muted-foreground">
                    Attributes
                  </p>
                  <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
                    {selectedAttributes.map(([key, value]) => (
                      <div key={key} className="contents">
                        <dt className="font-mono text-muted-foreground">
                          {key}
                        </dt>
                        <dd className="font-mono break-all">{value}</dd>
                      </div>
                    ))}
                  </dl>
                </div>
              ) : (
                <p className="text-muted-foreground">No attributes.</p>
              )}

              {selectedLinks.length > 0 ? (
                <div>
                  <Separator className="my-2" />
                  <p className="mb-1 font-medium text-muted-foreground">
                    Linked traces
                  </p>
                  <ul className="space-y-1 font-mono">
                    {selectedLinks.map((link) => (
                      <li key={`${link.traceId}-${link.spanId}`}>
                        <Link
                          to="/traces/$traceId"
                          params={{ traceId: link.traceId }}
                          className="underline underline-offset-4"
                        >
                          {link.traceId}
                        </Link>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}

              {selectedLogs.length > 0 ? (
                <div>
                  <Separator className="my-2" />
                  <p className="mb-1 font-medium text-muted-foreground">
                    Logs in this span
                  </p>
                  <ul className="space-y-1 font-mono">
                    {selectedLogs.map((log, index) => (
                      <li key={index} className="flex gap-2">
                        <span className="shrink-0 text-muted-foreground">
                          {log.severityText}
                        </span>
                        <span className="break-all">{log.body}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
            </CardContent>
          </Card>
        ) : null}
      </div>
    </div>
  )
}

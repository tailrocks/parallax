import { useEffect, useState } from "react"
import { Link } from "@tanstack/react-router"
import { IconArrowUpRight } from "@tabler/icons-react"

import { CopyButton } from "@/shared/console/copy-button"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"
import type { IssueCorrelationResult } from "@/features/issues/model/issue-detail"
import { rangeLinkSearch, type ResolvedRange } from "@/domain/range"
import { formatTimeInRange } from "@/shared/format"
import { severityColor, severityToken } from "@/shared/colors"
import { LongValue } from "@/features/issues/components/long-value"

export type IssueCorrelationState =
  | { readonly status: "loading" }
  | { readonly status: "no-trace" }
  | IssueCorrelationResult
  | { readonly status: "error"; readonly message: string }

export function CorrelationCard({
  state,
  traceId,
  range,
  onRetry,
}: {
  state: IssueCorrelationState
  traceId: string
  range: ResolvedRange
  onRetry: () => void
}) {
  const [showAllLogs, setShowAllLogs] = useState(false)

  useEffect(() => {
    setShowAllLogs(false)
  }, [state])

  const ready = state.status === "ready" ? state.correlation : null
  const resourceEntries =
    ready === null
      ? []
      : Object.entries(ready.resource).map(
          ([key, value]) =>
            [key, typeof value === "string" ? value : JSON.stringify(value)] as const
        )
  const logs = ready?.logs ?? []
  const visibleLogs = showAllLogs ? logs : logs.slice(0, 12)

  return (
    <Card>
      <CardHeader className="flex-row items-start justify-between gap-3">
        <CardTitle className="text-sm">Correlation</CardTitle>
        {ready ? (
          <div className="flex flex-wrap items-center justify-end gap-2">
            {ready.invocationId ? (
              <Link
                to="/invocations/$invocationId"
                params={{ invocationId: ready.invocationId }}
                search={rangeLinkSearch(range)}
                className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
              >
                invocation
                <IconArrowUpRight className="size-3" />
              </Link>
            ) : (
              <span className="text-xs text-muted-foreground">No invocation span</span>
            )}
            <CopyButton value={traceId} />
          </div>
        ) : null}
      </CardHeader>
      <CardContent className="space-y-3">
        {state.status === "loading" ? (
          <div className="space-y-2" aria-live="polite">
            <p className="text-sm text-muted-foreground">Loading trace correlation…</p>
            <Skeleton className="h-4 w-48" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : state.status === "no-trace" || !traceId ? (
          <p className="text-sm text-muted-foreground">No trace linked to this event.</p>
        ) : state.status === "trace-unavailable" ? (
          <div className="flex flex-wrap items-center gap-2">
            <p className="text-sm text-muted-foreground">Trace is unavailable.</p>
            <Button size="xs" variant="outline" type="button" onClick={onRetry}>
              Retry
            </Button>
          </div>
        ) : state.status === "error" ? (
          <div className="flex flex-wrap items-center gap-2">
            <p className="text-sm text-destructive">{state.message}</p>
            <Button size="xs" variant="outline" type="button" onClick={onRetry}>
              Retry
            </Button>
          </div>
        ) : (
          <>
            <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
              <Link
                to="/traces/$traceId"
                params={{ traceId }}
                search={rangeLinkSearch(range)}
                className="inline-flex items-center gap-1 hover:text-foreground"
              >
                Open trace {traceId.slice(0, 16)}
                <IconArrowUpRight className="size-3" />
              </Link>
              <Link
                to="/logs"
                search={{ trace: traceId }}
                className="inline-flex items-center gap-1 hover:text-foreground"
              >
                Open in Logs
                <IconArrowUpRight className="size-3" />
              </Link>
              {ready?.releaseVersion ? (
                <Badge variant="secondary">release {ready.releaseVersion}</Badge>
              ) : null}
            </div>

            {resourceEntries.length > 0 ? (
              <dl className="grid gap-x-4 gap-y-1 text-xs md:grid-cols-[minmax(0,180px)_minmax(0,1fr)]">
                {resourceEntries.map(([key, value]) => (
                  <div key={key} className="contents">
                    <dt title={key} className="min-w-0 truncate font-mono text-muted-foreground">
                      {key}
                    </dt>
                    <dd className="min-w-0">
                      <LongValue value={value} />
                    </dd>
                  </div>
                ))}
              </dl>
            ) : null}

            {logs.length === 0 ? (
              <p className="text-sm text-muted-foreground">No logs in this trace.</p>
            ) : (
              <div className="space-y-2">
                <ul className="space-y-1 font-mono text-xs">
                  {visibleLogs.map((log, index) => {
                    const token = severityToken(log.severityText)
                    return (
                      <li
                        key={`${log.tsNanos}-${index}`}
                        className="grid gap-2 rounded-md px-2 py-1 sm:grid-cols-[100px_72px_minmax(0,1fr)]"
                      >
                        <span className="text-muted-foreground">
                          {formatTimeInRange(log.tsNanos, range)}
                        </span>
                        <span
                          style={token ? { color: severityColor(token) } : undefined}
                          className="font-medium"
                        >
                          {log.severityText || "log"}
                        </span>
                        <span className="break-words whitespace-pre-wrap">{log.body}</span>
                      </li>
                    )
                  })}
                </ul>
                {logs.length > 12 ? (
                  <Button
                    size="xs"
                    variant="ghost"
                    type="button"
                    onClick={() => setShowAllLogs((value) => !value)}
                  >
                    {showAllLogs ? "Show fewer" : `Show all (${logs.length})`}
                  </Button>
                ) : null}
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  )
}

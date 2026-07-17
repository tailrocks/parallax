import { Link } from "@tanstack/react-router"
import { IconAlertTriangleFilled } from "@tabler/icons-react"

import { EmptyState } from "@/components/console/empty-state"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { formatCount } from "@/lib/format"
import { rangeLinkSearch } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"

export interface InvocationIssue {
  fingerprint: string
  title: string
  status: string
  eventCount: number
  errorType?: string | null
  lastSeenNanos?: string | null
}

/** Pure error.type breakdown over this invocation's correlated issues. */
export function errorTypeBreakdown(
  issues: InvocationIssue[]
): Array<{ errorType: string; count: number }> {
  const counts = new Map<string, number>()
  for (const issue of issues) {
    const key = issue.errorType?.trim() || "(untyped)"
    counts.set(key, (counts.get(key) ?? 0) + issue.eventCount)
  }
  return [...counts.entries()]
    .map(([errorType, count]) => ({ errorType, count }))
    .sort((a, b) => b.count - a.count)
}

export function InvocationErrorsTab({
  issues,
  range,
}: {
  issues: InvocationIssue[]
  range: ResolvedRange
}) {
  if (issues.length === 0) {
    return (
      <EmptyState
        icon={IconAlertTriangleFilled}
        title="No errors"
        description="Nothing yet — this invocation's traces produced no grouped errors."
      />
    )
  }
  const breakdown = errorTypeBreakdown(issues)
  const max = breakdown[0]?.count ?? 1
  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-sm">Errors by type</CardTitle>
        </CardHeader>
        <CardContent>
          <ul className="space-y-1.5">
            {breakdown.map((row) => (
              <li
                key={row.errorType}
                className="grid grid-cols-[minmax(0,14rem)_minmax(0,1fr)_4rem] items-center gap-2 text-sm"
              >
                <code className="truncate text-xs" title={row.errorType}>
                  {row.errorType}
                </code>
                <div className="h-3 rounded bg-muted/40">
                  <div
                    className="h-full rounded bg-rose-500/70"
                    style={{
                      width: `${Math.max((row.count / max) * 100, 2)}%`,
                    }}
                  />
                </div>
                <span className="text-right text-muted-foreground tabular-nums">
                  {formatCount(row.count)}
                </span>
              </li>
            ))}
          </ul>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">Correlated issues</CardTitle>
        </CardHeader>
        <CardContent>
          <ul className="space-y-2 text-sm">
            {issues.map((issue) => (
              <li
                key={issue.fingerprint}
                className="grid gap-2 rounded-lg border bg-muted/20 px-3 py-2 md:grid-cols-[minmax(0,1fr)_auto]"
              >
                <Link
                  to="/issues/$fingerprint"
                  params={{ fingerprint: issue.fingerprint }}
                  search={rangeLinkSearch(range)}
                  className="min-w-0 truncate font-medium hover:underline"
                >
                  {/* issue.title already leads with the error type. */}
                  {issue.title}
                </Link>
                <span className="flex items-center gap-2 text-xs text-muted-foreground">
                  <Badge variant={issue.status === "open" ? "rose" : "emerald"}>
                    {issue.status}
                  </Badge>
                  {formatCount(issue.eventCount)} events
                </span>
              </li>
            ))}
          </ul>
        </CardContent>
      </Card>
    </div>
  )
}

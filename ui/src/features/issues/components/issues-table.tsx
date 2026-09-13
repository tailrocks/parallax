import { Link } from "@tanstack/react-router"
import { useVirtualizer } from "@tanstack/react-virtual"
import { memo, useRef } from "react"

import { SortableHead } from "@/shared/console/data-table"
import { RelativeTime } from "@/shared/console/relative-time"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { topTags, trendEvents, type IssueRow } from "@/features/issues/model/issue-summary"
import {
  issueNeedsAttention,
  issueStatusBadgeVariant,
} from "@/features/issues/model/issue-status"
import type { IssueSort, IssuesSearchPatch } from "@/features/issues/model/issues-search"
import type { ResolvedRange } from "@/domain/time-range/range"
import { rangeLinkSearch } from "@/domain/time-range/range"
import { formatCount } from "@/shared/format"
import { cn } from "@/lib/utils"
import { rowKeyboardAttrs, useRowKeyboardNav } from "@/lib/row-keyboard-nav"

const VIRTUALIZE_THRESHOLD = 30
const ISSUE_ROW_ESTIMATE = 72
const ISSUE_COLUMN_COUNT = 8

export const MiniSparkline = memo(function MiniSparkline({
  data,
  className,
}: {
  data: Array<{ value: number }>
  className?: string
}) {
  if (data.length === 0) {
    return <span className={cn("block h-9 w-full", className)} />
  }
  const values = data.map((point) => point.value)
  const min = Math.min(...values)
  const max = Math.max(...values)
  const span = max - min || 1
  const width = 100
  const height = 36
  const pad = 2
  const points = values
    .map((value, index) => {
      const x = values.length === 1 ? width / 2 : (index / (values.length - 1)) * width
      const y = height - pad - ((value - min) / span) * (height - pad * 2)
      return `${x},${y}`
    })
    .join(" ")
  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      className={cn("h-9 w-full text-current", className)}
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      <polyline
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinejoin="round"
        strokeLinecap="round"
        points={points}
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  )
})

export function IssuesTable({
  items,
  range,
  sort,
  onSearch,
  onIssue,
}: {
  items: readonly IssueRow[]
  range: ResolvedRange
  sort: IssueSort
  onSearch: (patch: IssuesSearchPatch) => void
  onIssue: (issue: IssueRow) => void
}) {
  const parentRef = useRef<HTMLDivElement | null>(null)
  const virtualize = items.length > VIRTUALIZE_THRESHOLD
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ISSUE_ROW_ESTIMATE,
    overscan: 8,
    enabled: virtualize,
  })
  const virtualItems = virtualize ? virtualizer.getVirtualItems() : null
  const firstVirtual = virtualItems?.[0]
  const lastVirtual = virtualItems?.at(-1)
  const paddingTop = firstVirtual?.start ?? 0
  const paddingBottom =
    virtualize && lastVirtual ? Math.max(0, virtualizer.getTotalSize() - lastVirtual.end) : 0
  const rows = virtualize
    ? (virtualItems ?? []).map((item) => items[item.index]!).filter(Boolean)
    : items
  const activeRow = useRowKeyboardNav({
    scope: "issues",
    count: rows.length,
    onOpen: (index) => {
      const issue = rows[index]
      if (issue) onIssue(issue)
    },
  })

  const header = (
    <TableHeader className={virtualize ? "sticky top-0 z-10 bg-card" : undefined}>
      <TableRow>
        <TableHead>Issue</TableHead>
        <TableHead className="w-36">Service</TableHead>
        <TableHead className="w-24">Trend</TableHead>
        <TableHead className="w-24 text-right">
          <SortableHead
            {...(sort === "EVENTS" ? { sort: "events:desc" } : {})}
            sortKey="events"
            onSort={() => onSearch({ sort: "EVENTS" })}
          >
            Events
          </SortableHead>
        </TableHead>
        <TableHead className="w-28 text-right">Age</TableHead>
        <TableHead className="w-32 text-right">
          <SortableHead
            {...(sort === "LAST_SEEN" ? { sort: "lastSeen:desc" } : {})}
            sortKey="lastSeen"
            onSort={() => onSearch({ sort: "LAST_SEEN" })}
          >
            Last seen
          </SortableHead>
        </TableHead>
        <TableHead className="w-56">Tags</TableHead>
        <TableHead className="w-24">Status</TableHead>
      </TableRow>
    </TableHeader>
  )

  const body = (
    <TableBody>
      {paddingTop > 0 ? (
        <tr aria-hidden="true">
          <td
            colSpan={ISSUE_COLUMN_COUNT}
            className="border-0 p-0"
            style={{ height: paddingTop }}
          />
        </tr>
      ) : null}
      {rows.map((issue, index) => {
        const recentOpen = issueNeedsAttention(issue.status) && trendEvents(issue) > 0
        const tags = topTags(issue.tags)
        return (
          <TableRow
            key={issue.fingerprint}
            {...rowKeyboardAttrs("issues", index)}
            className={cn(
              "cursor-pointer",
              recentOpen && "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]",
              activeRow === index && "bg-accent/60"
            )}
            onClick={() => onIssue(issue)}
          >
            <TableCell className="max-w-xl">
              <div className="min-w-0 space-y-1">
                <Link
                  to="/issues/$service/$fingerprint"
                  params={{ service: issue.service, fingerprint: issue.fingerprint }}
                  search={rangeLinkSearch(range)}
                  className="block font-medium hover:underline"
                  onClick={(event) => event.stopPropagation()}
                >
                  {issue.errorType || issue.title}
                </Link>
                <div className="flex min-w-0 flex-wrap items-center gap-2 text-sm text-muted-foreground">
                  <span className="min-w-0 break-words">{issue.title}</span>
                  {issue.lastTraceId ? (
                    <Link
                      to="/traces/$traceId"
                      params={{ traceId: issue.lastTraceId }}
                      search={rangeLinkSearch(range)}
                      aria-label={`trace ${issue.lastTraceId}`}
                      className="shrink-0"
                      onClick={(event) => event.stopPropagation()}
                    >
                      <Badge variant="secondary" className="font-mono">
                        trace {issue.lastTraceId.slice(0, 8)}
                      </Badge>
                    </Link>
                  ) : null}
                </div>
                {issue.culprit ? (
                  <div className="truncate font-mono text-xs text-muted-foreground/70">
                    {issue.culprit}
                  </div>
                ) : null}
              </div>
            </TableCell>
            <TableCell>
              <Link
                to="/services/$service"
                params={{ service: issue.service }}
                search={rangeLinkSearch(range)}
                className="inline-flex max-w-36"
                onClick={(event) => event.stopPropagation()}
              >
                <Badge variant="outline" className="truncate">
                  {issue.service}
                </Badge>
              </Link>
            </TableCell>
            <TableCell>
              <Link
                to="/issues/$service/$fingerprint"
                params={{ service: issue.service, fingerprint: issue.fingerprint }}
                search={rangeLinkSearch(range)}
                className="block text-rose-500"
                onClick={(event) => event.stopPropagation()}
              >
                <MiniSparkline
                  data={issue.trend.map((point) => ({
                    value: point.count,
                  }))}
                  className="h-9"
                />
              </Link>
            </TableCell>
            <TableCell className="text-right tabular-nums">
              <Link
                to="/issues/$service/$fingerprint"
                params={{ service: issue.service, fingerprint: issue.fingerprint }}
                search={rangeLinkSearch(range)}
                className="hover:underline"
                onClick={(event) => event.stopPropagation()}
              >
                {formatCount(issue.eventCount)}
              </Link>
            </TableCell>
            <TableCell className="text-right text-muted-foreground">
              <RelativeTime nanos={issue.firstSeenNanos} />
            </TableCell>
            <TableCell className="text-right text-muted-foreground">
              <RelativeTime nanos={issue.lastSeenNanos} />
            </TableCell>
            <TableCell>
              <div className="flex flex-wrap gap-1">
                {tags.length === 0 ? (
                  <span className="text-muted-foreground">-</span>
                ) : (
                  tags.map((tag) => (
                    <Badge
                      key={tag.label}
                      variant="secondary"
                      className="max-w-36 truncate font-mono"
                    >
                      {tag.label}
                      {tag.rest > 0 ? (
                        <span className="ml-1 text-muted-foreground">+{tag.rest}</span>
                      ) : null}
                    </Badge>
                  ))
                )}
              </div>
            </TableCell>
            <TableCell>
              <Badge variant={issueStatusBadgeVariant(issue.status)}>{issue.status}</Badge>
            </TableCell>
          </TableRow>
        )
      })}
      {paddingBottom > 0 ? (
        <tr aria-hidden="true">
          <td
            colSpan={ISSUE_COLUMN_COUNT}
            className="border-0 p-0"
            style={{ height: paddingBottom }}
          />
        </tr>
      ) : null}
    </TableBody>
  )

  if (!virtualize) {
    return (
      <div className="overflow-hidden rounded-lg border bg-card">
        <Table>
          {header}
          {body}
        </Table>
      </div>
    )
  }

  return (
    <div
      ref={parentRef}
      data-virtualized="issues"
      className="relative max-h-[min(70vh,720px)] w-full overflow-auto rounded-lg border bg-card"
    >
      <Table>
        {header}
        {body}
      </Table>
    </div>
  )
}

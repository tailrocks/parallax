import { Link, createFileRoute, useNavigate } from "@tanstack/react-router"
import { IconTerminal2 } from "@tabler/icons-react"
import { z } from "zod"

import { CopyButton } from "@/components/console/copy-button"
import { EmptyState } from "@/components/console/empty-state"
import {
  FilterSelect,
  SearchInput,
  Toolbar,
} from "@/components/console/data-table"
import { RangePicker } from "@/components/console/range-picker"
import { RelativeTime } from "@/components/console/relative-time"
import { PageHeader } from "@/components/page-header"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { graphqlCached } from "@/lib/api"
import type { ObservedRun } from "@/lib/api"
import { formatCount, formatDurationNs } from "@/lib/format"
import {
  rangeLinkSearch,
  rangeSearchSchema,
  resolveRangeSearch,
  updateRangeSearch,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

interface RunRecord {
  invocationId: string
  command: string | null
  status: string
  exitCode: number | null
  startedAtNanos: string
  endedAtNanos: string | null
  errorCount: number
  traceCount: number
}

export interface RunRow {
  invocationId: string
  source: "cli" | "external"
  command: string | null
  service: string | null
  status: "running" | "finished" | "external"
  exitCode: number | null
  startedAtNanos: string
  endedAtNanos: string | null
  lastNanos: string
  errorCount: number | null
  traceCount: number | null
  spanCount: number
  logCount: number
}

export interface RunsData {
  rows: RunRow[]
}

export interface RunsSearch {
  q?: string
  status?: "running" | "finished" | "external"
  range?: string
  from?: string
  to?: string
}

type RunsSearchPatch = {
  [K in keyof RunsSearch]?: RunsSearch[K] | undefined
}

export function durationNs(row: RunRow, now = Date.now()): string | null {
  const start = BigInt(row.startedAtNanos)
  const end =
    row.status === "running"
      ? BigInt(now) * 1_000_000n
      : row.endedAtNanos
        ? BigInt(row.endedAtNanos)
        : BigInt(row.lastNanos)
  if (end <= start) return null
  return (end - start).toString()
}

export function statusTone(
  row: RunRow
): "sky" | "emerald" | "rose" | "secondary" {
  if (row.status === "running") return "sky"
  if (row.status === "external") return "secondary"
  return row.exitCode === 0 ? "emerald" : "rose"
}

export function mergeRuns(
  runs: RunRecord[],
  observedInvocations: ObservedRun[]
): RunRow[] {
  const rows = new Map<string, RunRow>()
  for (const observed of observedInvocations) {
    rows.set(observed.invocationId, {
      invocationId: observed.invocationId,
      source: "external",
      command: null,
      service: observed.service,
      status: "external",
      exitCode: null,
      startedAtNanos: observed.firstNanos,
      endedAtNanos: observed.lastNanos,
      lastNanos: observed.lastNanos,
      errorCount: null,
      traceCount: null,
      spanCount: observed.spanCount,
      logCount: observed.logCount,
    })
  }
  for (const run of runs) {
    const observed = rows.get(run.invocationId)
    rows.set(run.invocationId, {
      invocationId: run.invocationId,
      source: "cli",
      command: run.command,
      service: observed?.service ?? null,
      status: run.status === "running" ? "running" : "finished",
      exitCode: run.exitCode,
      startedAtNanos: run.startedAtNanos,
      endedAtNanos: run.endedAtNanos,
      lastNanos: observed?.lastNanos ?? run.endedAtNanos ?? run.startedAtNanos,
      errorCount: run.errorCount,
      traceCount: run.traceCount,
      spanCount: observed?.spanCount ?? 0,
      logCount: observed?.logCount ?? 0,
    })
  }
  return [...rows.values()].sort((a, b) =>
    BigInt(a.lastNanos) < BigInt(b.lastNanos) ? 1 : -1
  )
}

export function filterRunsByRange(
  rows: RunRow[],
  range: ResolvedRange,
  nowNanos = (BigInt(Date.now()) * 1_000_000n).toString()
): RunRow[] {
  const windowStart = BigInt(range.fromNanos)
  const windowEnd = BigInt(range.toNanos)
  const openEnd = BigInt(nowNanos)
  return rows.filter((row) => {
    const start = BigInt(row.startedAtNanos)
    const end =
      row.status === "running"
        ? openEnd
        : BigInt(row.endedAtNanos ?? row.lastNanos)
    return start <= windowEnd && end >= windowStart
  })
}

const runsSearchSchema = rangeSearchSchema.extend({
  q: z.unknown().optional(),
  status: z.unknown().optional(),
})

export const Route = createFileRoute("/runs/")({
  validateSearch: (search: Record<string, unknown>): RunsSearch => {
    const parsed = runsSearchSchema.parse(search)
    const result: RunsSearch = {}
    if (typeof parsed.q === "string" && parsed.q) result.q = parsed.q
    if (
      parsed.status === "running" ||
      parsed.status === "finished" ||
      parsed.status === "external"
    ) {
      result.status = parsed.status
    }
    if (parsed.range) result.range = parsed.range
    if (parsed.from) result.from = parsed.from
    if (parsed.to) result.to = parsed.to
    return result
  },
  loader: async () => {
    const { invocations, observedInvocations } = await graphqlCached<{
      invocations: RunRecord[]
      observedInvocations: ObservedRun[]
    }>(`
      {
        invocations {
          invocationId
          command
          status
          exitCode
          startedAtNanos
          endedAtNanos
          errorCount
          traceCount
        }
        observedInvocations {
          invocationId
          service
          firstNanos
          lastNanos
          spanCount
          logCount
        }
      }
    `)
    return {
      rows: mergeRuns(invocations, observedInvocations),
    } satisfies RunsData
  },
  component: RunsPage,
})

function RunsPage() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const range = resolveRangeSearch(search)
  const setSearch = (patch: RunsSearchPatch) => {
    const raw = { ...search, ...patch }
    const next: RunsSearch = {}
    for (const key of Object.keys(raw) as Array<keyof RunsSearch>) {
      const value = raw[key]
      if (value != null && value !== "") Object.assign(next, { [key]: value })
    }
    void navigate({ search: next })
  }
  return (
    <RunsContent
      data={data}
      search={search}
      range={range}
      onSearch={setSearch}
      onRun={(invocationId) =>
        void navigate({
          to: "/runs/$runId",
          params: { runId: invocationId },
          search: rangeLinkSearch(range),
        })
      }
    />
  )
}

export function RunsContent({
  data,
  search,
  range,
  onSearch,
  onRun,
}: {
  data: RunsData
  search: RunsSearch
  range: ResolvedRange
  onSearch: (patch: RunsSearchPatch) => void
  onRun: (invocationId: string) => void
}) {
  const query = search.q?.toLowerCase() ?? ""
  const rowsInWindow = filterRunsByRange(data.rows, range)
  const rows = rowsInWindow.filter((row) => {
    const matchesStatus = !search.status || row.status === search.status
    const haystack =
      `${row.invocationId} ${row.command ?? ""} ${row.service ?? ""}`.toLowerCase()
    return matchesStatus && haystack.includes(query)
  })
  const detailSearch = rangeLinkSearch(range)

  return (
    <div className="space-y-4">
      <PageHeader
        icon={IconTerminal2}
        iconClassName="text-violet-500"
        title="Runs"
        description="Command and externally observed run ids with errors, duration, and last activity."
        actions={
          <RangePicker
            value={range}
            onChange={(next) => onSearch(updateRangeSearch(next))}
          />
        }
      />

      <Toolbar className="justify-between">
        <div className="flex flex-wrap items-center gap-2">
          <SearchInput
            value={search.q ?? ""}
            onChange={(q) => onSearch({ q })}
            placeholder="Search runs"
          />
          <FilterSelect
            {...(search.status ? { value: search.status } : {})}
            onChange={(status) =>
              onSearch({
                status:
                  status === "running" ||
                  status === "finished" ||
                  status === "external"
                    ? status
                    : undefined,
              })
            }
            placeholder="Any status"
            options={[
              { value: "running", label: "Running" },
              { value: "finished", label: "Finished" },
              { value: "external", label: "External" },
            ]}
          />
        </div>
        <span className="text-xs text-muted-foreground">
          {formatCount(rows.length)} of {formatCount(rowsInWindow.length)}
          {rowsInWindow.length !== data.rows.length ? (
            <>
              {" "}
              shown · {formatCount(rowsInWindow.length)} of{" "}
              {formatCount(data.rows.length)} runs in window
            </>
          ) : null}
        </span>
      </Toolbar>

      {rows.length === 0 ? (
        <EmptyState
          icon={IconTerminal2}
          title="No runs captured yet"
          description={
            <span className="inline-flex items-center gap-2">
              <code>parallax run &lt;your command&gt;</code>
              <CopyButton value="parallax run <your command>" />
            </span>
          }
        />
      ) : (
        <div className="overflow-hidden rounded-lg border bg-card">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Run</TableHead>
                <TableHead>Command</TableHead>
                <TableHead className="w-32">Status</TableHead>
                <TableHead className="w-24 text-right">Traces</TableHead>
                <TableHead className="w-24 text-right">Errors</TableHead>
                <TableHead className="w-28 text-right">Duration</TableHead>
                <TableHead className="w-32 text-right">Last seen</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rows.map((row) => {
                const errors = row.errorCount ?? 0
                return (
                  <TableRow
                    key={row.invocationId}
                    className={cn(
                      "cursor-pointer",
                      errors > 0 &&
                        "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
                    )}
                    onClick={() => onRun(row.invocationId)}
                  >
                    <TableCell>
                      <div className="flex min-w-0 items-center gap-2">
                        <Link
                          to="/runs/$runId"
                          params={{ runId: row.invocationId }}
                          search={detailSearch}
                          className="min-w-0 hover:underline"
                          onClick={(event) => event.stopPropagation()}
                        >
                          <code className="block max-w-44 truncate text-xs">
                            {row.invocationId}
                          </code>
                        </Link>
                        <CopyButton value={row.invocationId} />
                        <Badge variant="secondary">{row.source}</Badge>
                      </div>
                    </TableCell>
                    <TableCell className="max-w-md truncate font-mono text-xs text-muted-foreground">
                      {row.command ?? (
                        <span className="italic">
                          {row.service ?? "external telemetry"}
                        </span>
                      )}
                    </TableCell>
                    <TableCell>
                      <RunStatusBadge row={row} />
                    </TableCell>
                    <TableCell className="text-right tabular-nums">
                      {row.traceCount == null ? (
                        "-"
                      ) : (
                        <Link
                          to="/runs/$runId"
                          params={{ runId: row.invocationId }}
                          search={detailSearch}
                          className="hover:underline"
                          onClick={(event) => event.stopPropagation()}
                        >
                          {formatCount(row.traceCount)}
                        </Link>
                      )}
                    </TableCell>
                    <TableCell
                      className={cn(
                        "text-right tabular-nums",
                        errors > 0
                          ? "text-rose-600 dark:text-rose-400"
                          : "text-muted-foreground/40"
                      )}
                    >
                      {row.errorCount == null ? (
                        "-"
                      ) : (
                        <Link
                          to="/runs/$runId"
                          params={{ runId: row.invocationId }}
                          search={detailSearch}
                          className="hover:underline"
                          onClick={(event) => event.stopPropagation()}
                        >
                          {formatCount(errors)}
                        </Link>
                      )}
                    </TableCell>
                    <TableCell className="text-right tabular-nums">
                      {row.status === "running"
                        ? "..."
                        : durationNs(row)
                          ? formatDurationNs(durationNs(row)!)
                          : "-"}
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      <RelativeTime nanos={row.lastNanos} />
                    </TableCell>
                  </TableRow>
                )
              })}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  )
}

export function RunStatusBadge({ row }: { row: RunRow }) {
  if (row.status === "running") {
    return (
      <Badge variant="blue">
        <span className="size-1.5 animate-pulse rounded-full bg-current" />
        running
      </Badge>
    )
  }
  if (row.status === "external")
    return <Badge variant="secondary">external</Badge>
  if (row.exitCode === 0) return <Badge variant="emerald">finished</Badge>
  return <Badge variant="rose">exit {row.exitCode ?? "?"}</Badge>
}

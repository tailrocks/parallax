import { useEffect, useState } from "react"
import { createFileRoute, useNavigate, useRouter } from "@tanstack/react-router"
import { IconRefresh, IconTerminal2 } from "@tabler/icons-react"
import { z } from "zod"

import { CopyButton } from "@/components/console/copy-button"
import { EmptyState } from "@/components/console/empty-state"
import {
  FilterSelect,
  SearchInput,
  Toolbar,
} from "@/components/console/data-table"
import { InvocationsTable } from "@/components/console/invocations/invocations-table"
import { RangePicker } from "@/components/console/range-picker"
import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { graphql, graphqlCached } from "@/lib/api"
import type { Invocation, ObservedInvocation } from "@/lib/api"
import { formatCount } from "@/lib/format"
import { invocationStatus, mergeInvocations } from "@/lib/invocation"
import type { InvocationRow, InvocationStatus } from "@/lib/invocation"
import {
  rangeLinkSearch,
  rangeSearchSchema,
  resolveRangeSearch,
  updateRangeSearch,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { usePageVisible } from "@/lib/use-visible"

const MODES = ["one_shot", "interactive", "daemon", "capsule"] as const
const STATUSES = ["running", "finished", "failed", "stale"] as const
const OUTCOMES = [
  "success",
  "failure",
  "error",
  "timeout",
  "skip",
  "cancellation",
] as const

export interface InvocationsSearch {
  q?: string
  mode?: (typeof MODES)[number]
  status?: InvocationStatus
  outcome?: (typeof OUTCOMES)[number]
  live?: boolean
  range?: string
  from?: string
  to?: string
}

type InvocationsSearchPatch = {
  [K in keyof InvocationsSearch]?: InvocationsSearch[K] | undefined
}

const INVOCATIONS_QUERY = `
  {
    invocations {
      invocationId
      command
      appMode
      outcome
      status
      exitCode
      startedAtNanos
      endedAtNanos
      errorCount
      traceCount
      sessionCount
    }
    observedInvocations {
      invocationId
      service
      lastCommand
      appMode
      firstNanos
      lastNanos
      spanCount
      logCount
    }
  }
`

interface InvocationsQueryData {
  invocations: Invocation[]
  observedInvocations: ObservedInvocation[]
}

export function filterInvocationsByRange(
  rows: InvocationRow[],
  range: ResolvedRange,
  nowNanos = (BigInt(Date.now()) * 1_000_000n).toString()
): InvocationRow[] {
  const windowStart = BigInt(range.fromNanos)
  const windowEnd = BigInt(range.toNanos)
  const openEnd = BigInt(nowNanos)
  return rows.filter((row) => {
    const start = BigInt(row.startedAtNanos)
    const end = row.endedAtNanos != null ? BigInt(row.endedAtNanos) : openEnd
    return start <= windowEnd && end >= windowStart
  })
}

const invocationsSearchSchema = rangeSearchSchema.extend({
  q: z.unknown().optional(),
  mode: z.unknown().optional(),
  status: z.unknown().optional(),
  outcome: z.unknown().optional(),
  live: z.unknown().optional(),
})

export const Route = createFileRoute("/invocations/")({
  validateSearch: (search: Record<string, unknown>): InvocationsSearch => {
    const parsed = invocationsSearchSchema.parse(search)
    const result: InvocationsSearch = {}
    if (typeof parsed.q === "string" && parsed.q) result.q = parsed.q
    const mode = MODES.find((value) => value === parsed.mode)
    if (mode) result.mode = mode
    const status = STATUSES.find((value) => value === parsed.status)
    if (status) result.status = status
    const outcome = OUTCOMES.find((value) => value === parsed.outcome)
    if (outcome) result.outcome = outcome
    if (parsed.live === true || parsed.live === "true") result.live = true
    if (parsed.range) result.range = parsed.range
    if (parsed.from) result.from = parsed.from
    if (parsed.to) result.to = parsed.to
    return result
  },
  loader: async () => {
    const data = await graphqlCached<InvocationsQueryData>(INVOCATIONS_QUERY)
    return {
      rows: mergeInvocations(data.invocations, data.observedInvocations),
    }
  },
  component: InvocationsPage,
})

function InvocationsPage() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const router = useRouter()
  const range = resolveRangeSearch(search)
  const pageVisible = usePageVisible()
  const live = search.live === true
  const [polledRows, setPolledRows] = useState<InvocationRow[] | null>(null)

  useEffect(() => {
    if (!live || !pageVisible) return
    const timer = setInterval(() => {
      void graphql<InvocationsQueryData>(INVOCATIONS_QUERY)
        .then((next) =>
          setPolledRows(
            mergeInvocations(next.invocations, next.observedInvocations)
          )
        )
        // Live polling tolerates transient API failures; next tick retries.
        .catch(() => {})
    }, 5_000)
    return () => clearInterval(timer)
  }, [live, pageVisible])

  const setSearch = (patch: InvocationsSearchPatch) => {
    const raw = { ...search, ...patch }
    const next: InvocationsSearch = {}
    for (const key of Object.keys(raw) as Array<keyof InvocationsSearch>) {
      const value = raw[key]
      if (value != null && value !== "" && value !== false) {
        Object.assign(next, { [key]: value })
      }
    }
    void navigate({ search: next })
  }

  return (
    <InvocationsContent
      rows={polledRows ?? data.rows}
      search={search}
      range={range}
      live={live}
      onSearch={setSearch}
      onRefresh={() => void router.invalidate()}
      onOpen={(invocationId) =>
        void navigate({
          to: "/invocations/$invocationId",
          params: { invocationId },
          search: rangeLinkSearch(range),
        })
      }
    />
  )
}

export function InvocationsContent({
  rows: allRows,
  search,
  range,
  live,
  onSearch,
  onRefresh,
  onOpen,
}: {
  rows: InvocationRow[]
  search: InvocationsSearch
  range: ResolvedRange
  live: boolean
  onSearch: (patch: InvocationsSearchPatch) => void
  onRefresh: () => void
  onOpen: (invocationId: string) => void
}) {
  const query = search.q?.toLowerCase() ?? ""
  const rowsInWindow = filterInvocationsByRange(allRows, range)
  const rows = rowsInWindow.filter((row) => {
    if (search.mode && row.appMode !== search.mode) return false
    if (search.status && invocationStatus(row) !== search.status) return false
    if (search.outcome && row.outcome !== search.outcome) return false
    const haystack =
      `${row.invocationId} ${row.command ?? ""} ${row.service ?? ""}`.toLowerCase()
    return haystack.includes(query)
  })

  return (
    <div className="space-y-4">
      <PageHeader
        icon={IconTerminal2}
        iconClassName="text-violet-500"
        title="CLI Apps"
        description="Every observed CLI invocation — command, mode, sessions, errors, and live activity."
        actions={
          <>
            <Button
              size="sm"
              variant={live ? "secondary" : "outline"}
              onClick={() => onSearch({ live: live ? undefined : true })}
            >
              {live ? (
                <span className="size-1.5 animate-pulse rounded-full bg-emerald-500" />
              ) : null}
              {live ? "Live" : "Go live"}
            </Button>
            {!live ? (
              <Button size="sm" variant="outline" onClick={onRefresh}>
                <IconRefresh />
                Refresh
              </Button>
            ) : null}
            <RangePicker
              value={range}
              onChange={(next) => onSearch(updateRangeSearch(next))}
            />
          </>
        }
      />

      <Toolbar className="justify-between">
        <div className="flex flex-wrap items-center gap-2">
          <SearchInput
            value={search.q ?? ""}
            onChange={(q) => onSearch({ q })}
            placeholder="Search invocations"
          />
          <FilterSelect
            {...(search.mode ? { value: search.mode } : {})}
            onChange={(mode) =>
              onSearch({ mode: MODES.find((value) => value === mode) })
            }
            placeholder="Any mode"
            options={MODES.map((value) => ({
              value,
              label: value.replace("_", "-"),
            }))}
          />
          <FilterSelect
            {...(search.status ? { value: search.status } : {})}
            onChange={(status) =>
              onSearch({ status: STATUSES.find((value) => value === status) })
            }
            placeholder="Any status"
            options={STATUSES.map((value) => ({ value, label: value }))}
          />
          <FilterSelect
            {...(search.outcome ? { value: search.outcome } : {})}
            onChange={(outcome) =>
              onSearch({
                outcome: OUTCOMES.find((value) => value === outcome),
              })
            }
            placeholder="Any outcome"
            options={OUTCOMES.map((value) => ({ value, label: value }))}
          />
        </div>
        <span className="text-xs text-muted-foreground">
          {formatCount(rows.length)} of {formatCount(rowsInWindow.length)} in
          window
        </span>
      </Toolbar>

      {rows.length === 0 ? (
        <EmptyState
          icon={IconTerminal2}
          title="No CLI invocations yet"
          description={
            <span className="inline-flex items-center gap-2">
              <code>parallax invocation start -- &lt;your command&gt;</code>
              <CopyButton value="parallax invocation start -- <your command>" />
            </span>
          }
        />
      ) : (
        <InvocationsTable
          rows={rows}
          detailSearch={rangeLinkSearch(range)}
          onOpen={onOpen}
        />
      )}
    </div>
  )
}

import { useNavigate, useRouterState } from "@tanstack/react-router"
import { IconFlask, IconTerminal2 } from "@tabler/icons-react"

import {
  ClearFiltersButton,
  FilterSelect,
  SearchInput,
  SortableHead,
  Toolbar,
} from "@/shared/console/data-table"
import { EmptyState } from "@/shared/console/empty-state"
import { useDelayedLoading } from "@/shared/console/hooks"
import { RelativeTime } from "@/shared/console/relative-time"
import { TableSkeleton } from "@/shared/console/skeletons"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  flakyLabel,
  rollupLabel,
  suiteLabel,
  type TestExplorerRow,
  type TestsData,
} from "@/features/tests/model/test-summary"
import {
  patchTestsSearch,
  type TestExplorerSort,
  type TestsSearch,
  type TestsSearchPatch,
} from "@/features/tests/model/tests-search"
import { RangePicker } from "@/features/time-range"
import { formatCount } from "@/shared/format"
import {
  rangeLinkSearch,
  resolveRangeSearch,
  updateRangeSearch,
  type ResolvedRange,
} from "@/domain/time-range/range"
import { TEST_ROLLUP } from "@/shared/colors"
import { cn } from "@/lib/utils"
import { PageHeader } from "@/shared/components/page-header"

const ROLLUP_OPTIONS = [
  { value: "FAILED", label: "Failed" },
  { value: "FLAKY_PASS", label: "Flaky pass" },
  { value: "BROKEN", label: "Broken" },
  { value: "PASSED", label: "Passed" },
  { value: "SKIPPED", label: "Skipped" },
  { value: "UNKNOWN", label: "Unknown" },
] as const

const FLAKY_OPTIONS = [
  { value: "FLAKY", label: "Flaky" },
  { value: "BROKEN", label: "Broken" },
  { value: "FIXED", label: "Fixed" },
  { value: "HEALTHY", label: "Healthy" },
] as const

function rollupTone(rollup: TestExplorerRow["rollup"]): string {
  return TEST_ROLLUP[rollup].badge
}

export function TestsPage({ data, search }: { data: TestsData; search: TestsSearch }) {
  const navigate = useNavigate({ from: "/tests/" })
  const range = resolveRangeSearch(search)
  const routerLoading = useRouterState({
    select: (state) => state.status === "pending",
  })
  const loading = useDelayedLoading(routerLoading)

  const setSearch = (patch: TestsSearchPatch) =>
    void navigate({ search: patchTestsSearch(search, patch) })

  return (
    <TestsContent
      data={data}
      search={search}
      range={range}
      loading={loading}
      onSearch={setSearch}
      onCase={(caseKey) =>
        void navigate({
          to: "/tests/$caseKey",
          params: { caseKey },
          search: rangeLinkSearch(range),
        })
      }
    />
  )
}

export function TestsContent({
  data,
  search,
  range,
  loading,
  onSearch,
  onCase,
}: {
  data: TestsData
  search: TestsSearch
  range: ResolvedRange
  loading?: boolean
  onSearch: (patch: TestsSearchPatch) => void
  onCase: (caseKey: string) => void
}) {
  const hasFilters = Boolean(
    search.q || search.suite || search.service || search.status || search.flakyState
  )
  const sort: TestExplorerSort = search.sort ?? "LAST_SEEN"

  return (
    <div className="space-y-4">
      <PageHeader
        icon={IconFlask}
        iconClassName="text-violet-500"
        title="Tests"
        description="Variant-scoped test results fused with traces, issues, and flaky state."
        actions={
          <RangePicker value={range} onChange={(next) => onSearch(updateRangeSearch(next))} />
        }
      />

      <Toolbar className="justify-between">
        <div className="flex flex-wrap items-center gap-2">
          <SearchInput
            value={search.q ?? ""}
            onChange={(q) => onSearch({ q })}
            placeholder="Search name / suite"
          />
          <FilterSelect
            {...(search.service ? { value: search.service } : {})}
            onChange={(service) => onSearch({ service })}
            placeholder="All services"
            options={data.services.map((service) => ({
              value: service,
              label: service,
            }))}
          />
          <FilterSelect
            {...(search.status ? { value: search.status } : {})}
            onChange={(status) =>
              onSearch({
                status: ROLLUP_OPTIONS.some((option) => option.value === status)
                  ? (status as TestsSearch["status"])
                  : undefined,
              })
            }
            placeholder="Any rollup"
            options={[...ROLLUP_OPTIONS]}
          />
          <FilterSelect
            {...(search.flakyState ? { value: search.flakyState } : {})}
            onChange={(flakyState) =>
              onSearch({
                flakyState: FLAKY_OPTIONS.some((option) => option.value === flakyState)
                  ? (flakyState as TestsSearch["flakyState"])
                  : undefined,
              })
            }
            placeholder="Any flaky state"
            options={[...FLAKY_OPTIONS]}
          />
          {hasFilters ? (
            <ClearFiltersButton
              onClick={() =>
                onSearch({
                  q: undefined,
                  suite: undefined,
                  service: undefined,
                  serviceVersion: undefined,
                  status: undefined,
                  flakyState: undefined,
                })
              }
            />
          ) : null}
        </div>
        <div className="text-sm text-muted-foreground tabular-nums">
          {formatCount(data.items.length)}
          {data.hasMore ? "+" : ""} variants
        </div>
      </Toolbar>

      {loading ? (
        <TableSkeleton rows={8} />
      ) : data.items.length === 0 ? (
        <EmptyState
          icon={IconTerminal2}
          title="No test variants yet"
          description="Emit parented test spans with test.case.name (or run a nextest/JUnit adapter) so Parallax can derive attempt chains."
        />
      ) : (
        <div className="overflow-hidden rounded-xl border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>
                  <SortableHead
                    {...(sort === "NAME" ? { sort: "name:asc" } : {})}
                    sortKey="name"
                    onSort={() => onSearch({ sort: "NAME" })}
                  >
                    Name
                  </SortableHead>
                </TableHead>
                <TableHead>Suite</TableHead>
                <TableHead>Rollup</TableHead>
                <TableHead>Flaky</TableHead>
                <TableHead className="text-right">Attempts</TableHead>
                <TableHead>Service</TableHead>
                <TableHead className="text-right">
                  <SortableHead
                    {...(sort === "LAST_SEEN" ? { sort: "lastSeen:desc" } : {})}
                    sortKey="lastSeen"
                    onSort={() => onSearch({ sort: "LAST_SEEN" })}
                  >
                    Last seen
                  </SortableHead>
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.items.map((row) => (
                <TableRow
                  key={`${row.caseKey}:${row.variantKey}`}
                  className="cursor-pointer"
                  onClick={() => onCase(row.caseKey)}
                >
                  <TableCell className="max-w-[18rem]">
                    <div className="truncate font-medium">{row.name}</div>
                    {row.parameters.length > 0 ? (
                      <div className="truncate text-xs text-muted-foreground">
                        {row.parameters
                          .filter((parameter) => !parameter.excluded)
                          .map((parameter) => `${parameter.name}=${parameter.value}`)
                          .join(", ")}
                      </div>
                    ) : null}
                  </TableCell>
                  <TableCell className="max-w-[12rem] truncate text-sm text-muted-foreground">
                    {suiteLabel(row.suitePath)}
                  </TableCell>
                  <TableCell>
                    <Badge variant="secondary" className={cn("capitalize", rollupTone(row.rollup))}>
                      {rollupLabel(row.rollup)}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-sm capitalize">
                    {row.flaky ? flakyLabel(row.flaky.state) : "—"}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">{row.attemptCount}</TableCell>
                  <TableCell className="text-sm">{row.lastResult.service || "—"}</TableCell>
                  <TableCell className="text-right text-sm">
                    <RelativeTime nanos={row.lastSeenNanos} />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  )
}

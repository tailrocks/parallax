import { Link } from "@tanstack/react-router"

import { HeatCell } from "@/shared/console/heat-cell"
import type { HeatScale } from "@/shared/console/heat-cell"
import { ServiceDot } from "@/shared/console/service-dot"
import { RelativeTime } from "@/shared/console/relative-time"
import { SortableHead } from "@/shared/console/data-table"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { serviceErrorRate } from "@/features/services/model/service-summary"
import type { ServiceTableRow } from "@/features/services/model/service-summary"
import type {
  ServiceSort,
  ServicesSearch,
  ServicesSearchPatch,
} from "@/features/services/model/services-search"
import { formatCount, formatDurationNs, formatPercent } from "@/lib/format"
import { rangeLinkSearch, type ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

export function ServicesTable({
  rows,
  search,
  range,
  p95Scale,
  errorRateScale,
  onSearch,
}: {
  rows: ServiceTableRow[]
  search: ServicesSearch
  range: ResolvedRange
  p95Scale: HeatScale
  errorRateScale: HeatScale
  onSearch: (patch: ServicesSearchPatch) => void
}) {
  const sortProps = search.sort ? { sort: search.sort } : {}
  const onSort = (sort: string | undefined) =>
    onSearch({ sort: sort as ServiceSort | undefined })

  return (
    <div className="overflow-hidden rounded-lg border bg-card">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>
              <SortableHead {...sortProps} sortKey="name" onSort={onSort}>
                Service
              </SortableHead>
            </TableHead>
            <TableHead className="w-28">
              <SortableHead {...sortProps} sortKey="version" onSort={onSort}>
                Version
              </SortableHead>
            </TableHead>
            <TableHead className="w-24">
              <SortableHead {...sortProps} sortKey="runtime" onSort={onSort}>
                Runtime
              </SortableHead>
            </TableHead>
            <TableHead className="w-28">
              <SortableHead {...sortProps} sortKey="env" onSort={onSort}>
                Env
              </SortableHead>
            </TableHead>
            <TableHead className="w-28 text-right">
              <SortableHead {...sortProps} sortKey="spans" onSort={onSort}>
                Spans
              </SortableHead>
            </TableHead>
            <TableHead className="w-28 text-right">
              <SortableHead {...sortProps} sortKey="errors" onSort={onSort}>
                Errors
              </SortableHead>
            </TableHead>
            <TableHead className="w-28 text-right">Error rate</TableHead>
            <TableHead className="w-28 text-right">
              <SortableHead {...sortProps} sortKey="p95" onSort={onSort}>
                p95
              </SortableHead>
            </TableHead>
            <TableHead className="w-32 text-right">
              <SortableHead {...sortProps} sortKey="lastSeen" onSort={onSort}>
                Last seen
              </SortableHead>
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map((row) => {
            const errors = Number(row.errorCount)
            const rate = serviceErrorRate(row)
            return (
              <TableRow
                key={row.name}
                className={cn(
                  "cursor-pointer",
                  errors > 0 && "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
                )}
              >
                <TableCell>
                  <Link
                    to="/services/$service"
                    params={{ service: row.name }}
                    search={rangeLinkSearch(range)}
                    className="flex min-w-0 items-center gap-2 font-medium"
                  >
                    <ServiceDot name={row.name} />
                    <span className="truncate">{row.name}</span>
                  </Link>
                </TableCell>
                <TableCell
                  className="font-mono text-xs"
                  title={`namespace ${row.serviceNamespace ?? "not emitted"}; instances ${row.instanceCount ?? "0"}`}
                >
                  {row.serviceVersion ?? (
                    <span className="font-sans text-muted-foreground">-</span>
                  )}
                </TableCell>
                <TableCell
                  className="text-muted-foreground"
                  title={
                    row.telemetrySdkName
                      ? `${row.telemetrySdkName} ${row.telemetrySdkVersion ?? ""}`.trim()
                      : "SDK not emitted"
                  }
                >
                  {row.telemetrySdkLanguage ?? "-"}
                </TableCell>
                <TableCell className="text-muted-foreground">
                  {row.deploymentEnvironment ?? "-"}
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  <Link
                    to="/traces"
                    search={{ service: row.name, ...rangeLinkSearch(range) }}
                    className="hover:underline"
                  >
                    {formatCount(Number(row.spanCount))}
                  </Link>
                </TableCell>
                <TableCell
                  className={cn(
                    "text-right tabular-nums",
                    errors > 0
                      ? "text-rose-600 dark:text-rose-400"
                      : "text-muted-foreground/40"
                  )}
                >
                  <Link
                    to="/traces"
                    search={{
                      service: row.name,
                      errors: true,
                      ...rangeLinkSearch(range),
                    }}
                    className="hover:underline"
                  >
                    {formatCount(errors)}
                  </Link>
                </TableCell>
                <TableCell className="text-right">
                  <HeatCell value={rate} scale={errorRateScale}>
                    {formatPercent(rate)}
                  </HeatCell>
                </TableCell>
                <TableCell className="text-right">
                  {row.p95Ms == null ? (
                    <span className="text-muted-foreground">-</span>
                  ) : (
                    <HeatCell value={row.p95Ms} scale={p95Scale}>
                      {formatDurationNs(row.p95Ms * 1_000_000)}
                    </HeatCell>
                  )}
                </TableCell>
                <TableCell className="text-right text-muted-foreground">
                  <RelativeTime nanos={row.lastSeenNanos} />
                </TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </div>
  )
}

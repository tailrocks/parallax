import { Link } from "@tanstack/react-router"
import { useMemo, useRef, useState } from "react"
import type { ReactNode } from "react"
import { useVirtualizer } from "@tanstack/react-virtual"

import { Chip } from "@/shared/console/chip"
import { CopyButton } from "@/shared/console/copy-button"
import { ServiceDot } from "@/shared/console/service-dot"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { severityColor, severityToken } from "@/shared/colors"
import { formatDateTime, formatLogBodyPreview, formatTimeInRange, stripAnsi } from "@/shared/format"
import { rangeLinkSearch, resolvePreset } from "@/domain/time-range/range"
import type { ResolvedRange } from "@/domain/time-range/range"

/** One log row, with every field the doc viewer needs. Shared by the Logs page
 * and the run detail page so both render logs identically. */
export interface LogDoc {
  _key?: string
  tsNanos: string
  eventName: string
  observedTsNanos: string
  service: string
  severityNum: number
  severityText: string
  body: string
  traceId: string
  spanId: string
  invocationId: string | null
  scopeName: string
  attributes: string
  resource: string
}

export const OPTIONAL_LOG_COLUMNS = ["service", "event", "trace", "scope"] as const
export type OptionalLogColumn = (typeof OPTIONAL_LOG_COLUMNS)[number]
const LOG_VIRTUALIZATION_THRESHOLD = 100

export function parseLogColumns(value: string | undefined): OptionalLogColumn[] {
  if (!value) return ["service", "trace"]
  const requested = value
    .split(",")
    .map((column) => column.trim())
    .filter((column): column is OptionalLogColumn =>
      OPTIONAL_LOG_COLUMNS.includes(column as OptionalLogColumn)
    )
  return Array.from(new Set(requested))
}

export function serializeLogColumns(columns: readonly OptionalLogColumn[]) {
  return columns.length > 0 ? columns.join(",") : undefined
}

export function severityVariant(num: number): "rose" | "amber" | "secondary" | "outline" {
  if (num >= 17) return "rose"
  if (num >= 13) return "amber"
  if (num >= 9) return "secondary"
  return "outline"
}

export function severityLabel(log: LogDoc) {
  return log.severityText || (log.severityNum >= 17 ? "ERROR" : "LOG")
}

function observedSkewField(log: LogDoc): [string, string] | null {
  try {
    const observed = BigInt(log.observedTsNanos || "0")
    if (observed === 0n) return null
    const emitted = BigInt(log.tsNanos || "0")
    const delta = observed > emitted ? observed - emitted : emitted - observed
    if (delta <= 1_000_000_000n) return null
    return ["@observed", formatDateTime(log.observedTsNanos)]
  } catch {
    return null
  }
}

/** Flatten one log into ordered field/value rows for the doc viewer. */
function docFields(log: LogDoc): Array<[string, string]> {
  const rows: Array<[string, string]> = [
    ["@timestamp", formatDateTime(log.tsNanos)],
    ["severity", `${severityLabel(log)} (${log.severityNum})`],
    ["service.name", log.service],
    ["body", stripAnsi(log.body)],
  ]
  if (log.eventName) rows.splice(2, 0, ["event.name", log.eventName])
  const observed = observedSkewField(log)
  if (observed) rows.splice(log.eventName ? 3 : 2, 0, observed)
  if (log.traceId) rows.push(["trace_id", log.traceId])
  if (log.spanId) rows.push(["span_id", log.spanId])
  if (log.invocationId) rows.push(["run_id", log.invocationId])
  if (log.scopeName) rows.push(["scope.name", log.scopeName])
  for (const [prefix, json] of [
    ["", log.attributes],
    ["resource.", log.resource],
  ] as const) {
    try {
      const parsed: unknown = JSON.parse(json)
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        for (const [key, value] of Object.entries(parsed)) {
          rows.push([`${prefix}${key}`, typeof value === "string" ? value : JSON.stringify(value)])
        }
      }
    } catch {
      // non-object payloads stay out of the table
    }
  }
  return rows
}

function rawDocument(log: LogDoc) {
  return JSON.stringify(log, null, 2)
}

function SeverityBadge({ log }: { log: LogDoc }) {
  const fatal = log.severityNum >= 21 || severityLabel(log).toUpperCase() === "FATAL"
  // Severity ramp token (plan 162): dot + WORD, never color alone.
  const token =
    severityToken(severityLabel(log)) ??
    (log.severityNum >= 21
      ? "fatal"
      : log.severityNum >= 17
        ? "error"
        : log.severityNum >= 13
          ? "warn"
          : log.severityNum >= 9
            ? "info"
            : log.severityNum >= 5
              ? "debug"
              : "trace")
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className="size-1.5 rounded-full" style={{ backgroundColor: severityColor(token) }} />
      <Badge variant={severityVariant(log.severityNum)} className={fatal ? "font-bold" : undefined}>
        {severityLabel(log)}
      </Badge>
    </span>
  )
}

function logKey(log: LogDoc) {
  return (
    log._key ?? `${log.tsNanos}-${log.spanId || "no-span"}-${log.traceId || "no-trace"}-${log.body}`
  )
}

function VirtualizedLogTable({
  logs,
  columnCount,
  headerRows,
  renderRow,
}: {
  logs: LogDoc[]
  columnCount: number
  headerRows: ReactNode
  renderRow: (log: LogDoc) => ReactNode
}) {
  const parentRef = useRef<HTMLDivElement | null>(null)
  const virtualizer = useVirtualizer({
    count: logs.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 44,
    overscan: 12,
  })
  const virtualItems = virtualizer.getVirtualItems()
  const firstVirtual = virtualItems[0]
  const lastVirtual = virtualItems.at(-1)
  const paddingTop = firstVirtual?.start ?? 0
  const paddingBottom = lastVirtual ? Math.max(0, virtualizer.getTotalSize() - lastVirtual.end) : 0

  return (
    <div
      ref={parentRef}
      data-slot="table-container"
      data-virtualized="logs"
      className="relative max-h-[min(70vh,720px)] w-full overflow-auto rounded-xl shadow-(--custom-shadow) corner-squircle dark:shadow-(--custom-shadow)"
    >
      <table
        data-slot="table"
        className="w-full caption-bottom rounded-xl bg-card/40 text-sm shadow-(--custom-shadow) corner-squircle dark:shadow-(--custom-shadow)"
      >
        <TableHeader className="sticky top-0 z-10 bg-card/95 backdrop-blur supports-backdrop-filter:bg-card/75">
          {headerRows}
        </TableHeader>
        <TableBody>
          {paddingTop > 0 ? (
            <tr aria-hidden="true">
              <td colSpan={columnCount} className="border-0 p-0" style={{ height: paddingTop }} />
            </tr>
          ) : null}
          {virtualItems.map((virtualItem) => {
            const log = logs[virtualItem.index]
            return log ? renderRow(log) : null
          })}
          {paddingBottom > 0 ? (
            <tr aria-hidden="true">
              <td
                colSpan={columnCount}
                className="border-0 p-0"
                style={{ height: paddingBottom }}
              />
            </tr>
          ) : null}
        </TableBody>
      </table>
    </div>
  )
}

/** The shared logs table: compact rows whose click opens a field-level document viewer. */
export function LogsTable({
  logs,
  range = resolvePreset("24h"),
  columns = ["service", "trace"],
  anchorNanos,
  onShowContext,
}: {
  logs: LogDoc[]
  range?: ResolvedRange
  columns?: OptionalLogColumn[]
  anchorNanos?: string | undefined
  onShowContext?: (log: LogDoc) => void
}) {
  const [selected, setSelected] = useState<LogDoc | null>(null)
  const [fieldSearch, setFieldSearch] = useState("")
  const visible = new Set(columns)
  const detailSearch = rangeLinkSearch(range)
  const columnCount =
    3 +
    (visible.has("service") ? 1 : 0) +
    (visible.has("event") ? 1 : 0) +
    (visible.has("trace") ? 1 : 0) +
    (visible.has("scope") ? 1 : 0)

  const selectedFields = useMemo(() => {
    if (!selected) return []
    const all = docFields(selected)
    const needle = fieldSearch.trim().toLowerCase()
    if (!needle) return all
    return all.filter(
      ([key, value]) => key.toLowerCase().includes(needle) || value.toLowerCase().includes(needle)
    )
  }, [selected, fieldSearch])

  const headerRows = (
    <TableRow>
      <TableHead className="w-36">Time</TableHead>
      <TableHead className="w-28">Severity</TableHead>
      {visible.has("service") ? <TableHead className="w-36">Service</TableHead> : null}
      {visible.has("event") ? <TableHead className="w-40">Event</TableHead> : null}
      <TableHead>Body</TableHead>
      {visible.has("trace") ? <TableHead className="w-28">Trace</TableHead> : null}
      {visible.has("scope") ? <TableHead className="w-36">Scope</TableHead> : null}
    </TableRow>
  )
  const header = <TableHeader>{headerRows}</TableHeader>

  function openLog(log: LogDoc) {
    setSelected(log)
    setFieldSearch("")
  }

  const renderRow = (log: LogDoc) => {
    const isAnchor = String(anchorNanos ?? "") === log.tsNanos
    return (
      <TableRow
        key={logKey(log)}
        data-anchor={isAnchor ? "true" : undefined}
        data-state={isAnchor ? "selected" : undefined}
        className="cursor-pointer"
        onClick={() => openLog(log)}
      >
        <TableCell className="font-mono text-xs whitespace-nowrap">
          <button
            type="button"
            className="text-left focus-visible:ring-[1.5px] focus-visible:ring-ring/50 focus-visible:outline-none"
            onClick={(event) => {
              event.stopPropagation()
              openLog(log)
            }}
          >
            {formatTimeInRange(log.tsNanos, range)}
          </button>
        </TableCell>
        <TableCell>
          <SeverityBadge log={log} />
        </TableCell>
        {visible.has("service") ? (
          <TableCell className="max-w-36 truncate text-muted-foreground">
            <span className="inline-flex min-w-0 items-center gap-1.5">
              <ServiceDot name={log.service} />
              <span className="truncate">{log.service}</span>
            </span>
          </TableCell>
        ) : null}
        {visible.has("event") ? (
          <TableCell className="max-w-40 truncate font-mono text-xs text-muted-foreground">
            {log.eventName || "-"}
          </TableCell>
        ) : null}
        <TableCell className="max-w-xl truncate font-mono text-xs">
          {formatLogBodyPreview(log.body)}
        </TableCell>
        {visible.has("trace") ? (
          <TableCell>
            {log.traceId ? (
              <Chip
                render={
                  <Link
                    to="/traces/$traceId"
                    params={{ traceId: log.traceId }}
                    search={detailSearch}
                    aria-label={`Trace ${log.traceId}`}
                    onClick={(event) => event.stopPropagation()}
                  />
                }
                className="text-muted-foreground hover:text-foreground"
              >
                {log.traceId.slice(0, 8)}
              </Chip>
            ) : (
              <span className="text-muted-foreground">-</span>
            )}
          </TableCell>
        ) : null}
        {visible.has("scope") ? (
          <TableCell className="max-w-36 truncate text-muted-foreground">
            {log.scopeName || "-"}
          </TableCell>
        ) : null}
      </TableRow>
    )
  }

  const table =
    logs.length > LOG_VIRTUALIZATION_THRESHOLD ? (
      <VirtualizedLogTable
        logs={logs}
        columnCount={columnCount}
        headerRows={headerRows}
        renderRow={renderRow}
      />
    ) : (
      <Table>
        {header}
        <TableBody>{logs.map(renderRow)}</TableBody>
      </Table>
    )

  return (
    <>
      {table}

      <Sheet
        open={selected !== null}
        onOpenChange={(open) => {
          if (!open) setSelected(null)
        }}
      >
        <SheetContent className="w-full overflow-y-auto sm:max-w-2xl">
          <SheetHeader>
            <SheetTitle className="flex items-center justify-between gap-2">
              <span>Log document</span>
              {selected ? <CopyButton value={rawDocument(selected)} /> : null}
            </SheetTitle>
            <SheetDescription>
              {selected ? `${selected.service} · ${formatDateTime(selected.tsNanos)}` : ""}
            </SheetDescription>
          </SheetHeader>
          {selected ? (
            <div className="flex flex-col gap-4 px-4 pb-6">
              <div className="flex flex-wrap items-center gap-2">
                <SeverityBadge log={selected} />
                {onShowContext ? (
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      onShowContext(selected)
                      setSelected(null)
                    }}
                  >
                    Show context (±30s)
                  </Button>
                ) : null}
                {selected.traceId ? (
                  <Chip
                    render={
                      <Link
                        to="/traces/$traceId"
                        params={{ traceId: selected.traceId }}
                        search={detailSearch}
                        aria-label={`Trace ${selected.traceId}`}
                      />
                    }
                  >
                    trace {selected.traceId.slice(0, 12)}
                  </Chip>
                ) : null}
                {selected.invocationId ? (
                  <Chip
                    render={
                      <Link
                        to="/invocations/$invocationId"
                        params={{ invocationId: selected.invocationId }}
                        search={detailSearch}
                        aria-label={`Run ${selected.invocationId}`}
                      />
                    }
                  >
                    run {selected.invocationId.slice(0, 12)}
                  </Chip>
                ) : null}
              </div>
              <Input
                value={fieldSearch}
                onChange={(event) => setFieldSearch(event.target.value)}
                placeholder="Search field names or values"
              />
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-44">Field</TableHead>
                    <TableHead>Value</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {selectedFields.map(([key, value]) => (
                    <TableRow key={key}>
                      <TableCell className="align-top font-mono text-xs text-muted-foreground">
                        {key}
                      </TableCell>
                      <TableCell className="font-mono text-xs break-all whitespace-pre-wrap">
                        {key === "trace_id" ? (
                          <Link
                            to="/traces/$traceId"
                            params={{ traceId: value }}
                            search={detailSearch}
                            className="underline underline-offset-4"
                          >
                            {value}
                          </Link>
                        ) : key === "run_id" ? (
                          <Link
                            to="/invocations/$invocationId"
                            params={{ invocationId: value }}
                            search={detailSearch}
                            className="underline underline-offset-4"
                          >
                            {value}
                          </Link>
                        ) : (
                          value
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          ) : null}
        </SheetContent>
      </Sheet>
    </>
  )
}

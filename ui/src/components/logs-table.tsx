import { Link } from "@tanstack/react-router"
import { useMemo, useRef, useState } from "react"
import { useVirtualizer } from "@tanstack/react-virtual"

import { CopyButton } from "@/components/console/copy-button"
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
import { formatDateTime, formatTimeInRange } from "@/lib/format"
import { resolvePreset } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

/** One log row, with every field the doc viewer needs. Shared by the Logs page
 * and the run detail page so both render logs identically. */
export interface LogDoc {
  _key?: string
  tsNanos: string
  service: string
  severityNum: number
  severityText: string
  body: string
  traceId: string
  spanId: string
  runId: string | null
  scopeName: string
  attributes: string
  resource: string
}

export const OPTIONAL_LOG_COLUMNS = ["service", "trace", "scope"] as const
export type OptionalLogColumn = (typeof OPTIONAL_LOG_COLUMNS)[number]

export function parseLogColumns(
  value: string | undefined
): OptionalLogColumn[] {
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

export function severityVariant(
  num: number
): "rose" | "amber" | "secondary" | "outline" {
  if (num >= 17) return "rose"
  if (num >= 13) return "amber"
  if (num >= 9) return "secondary"
  return "outline"
}

export function severityLabel(log: LogDoc) {
  return log.severityText || (log.severityNum >= 17 ? "ERROR" : "LOG")
}

/** Flatten one log into ordered field/value rows for the doc viewer. */
function docFields(log: LogDoc): Array<[string, string]> {
  const rows: Array<[string, string]> = [
    [
      "@timestamp",
      new Date(Number(BigInt(log.tsNanos) / 1_000_000n)).toISOString(),
    ],
    ["severity", `${severityLabel(log)} (${log.severityNum})`],
    ["service.name", log.service],
    ["body", log.body],
  ]
  if (log.traceId) rows.push(["trace_id", log.traceId])
  if (log.spanId) rows.push(["span_id", log.spanId])
  if (log.runId) rows.push(["run_id", log.runId])
  if (log.scopeName) rows.push(["scope.name", log.scopeName])
  for (const [prefix, json] of [
    ["", log.attributes],
    ["resource.", log.resource],
  ] as const) {
    try {
      const parsed: unknown = JSON.parse(json)
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        for (const [key, value] of Object.entries(parsed)) {
          rows.push([
            `${prefix}${key}`,
            typeof value === "string" ? value : JSON.stringify(value),
          ])
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
  const fatal =
    log.severityNum >= 21 || severityLabel(log).toUpperCase() === "FATAL"
  return (
    <span className="inline-flex items-center gap-1.5">
      <span
        className={cn(
          "size-1.5 rounded-full",
          log.severityNum >= 17
            ? "bg-rose-500"
            : log.severityNum >= 13
              ? "bg-amber-500"
              : "bg-muted-foreground/40"
        )}
      />
      <Badge
        variant={severityVariant(log.severityNum)}
        className={fatal ? "font-bold" : undefined}
      >
        {severityLabel(log)}
      </Badge>
    </span>
  )
}

function logKey(log: LogDoc) {
  return (
    log._key ??
    `${log.tsNanos}-${log.spanId || "no-span"}-${log.traceId || "no-trace"}-${log.body}`
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
  const parentRef = useRef<HTMLDivElement | null>(null)
  const visible = new Set(columns)
  const columnCount =
    3 + (visible.has("service") ? 1 : 0) + (visible.has("trace") ? 1 : 0) +
    (visible.has("scope") ? 1 : 0)
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
  const paddingBottom = lastVirtual
    ? Math.max(0, virtualizer.getTotalSize() - lastVirtual.end)
    : 0

  const selectedFields = useMemo(() => {
    if (!selected) return []
    const all = docFields(selected)
    const needle = fieldSearch.trim().toLowerCase()
    if (!needle) return all
    return all.filter(
      ([key, value]) =>
        key.toLowerCase().includes(needle) ||
        value.toLowerCase().includes(needle)
    )
  }, [selected, fieldSearch])

  const headerRows = (
    <TableRow>
      <TableHead className="w-36">Time</TableHead>
      <TableHead className="w-28">Severity</TableHead>
      {visible.has("service") ? (
        <TableHead className="w-36">Service</TableHead>
      ) : null}
      <TableHead>Body</TableHead>
      {visible.has("trace") ? (
        <TableHead className="w-28">Trace</TableHead>
      ) : null}
      {visible.has("scope") ? (
        <TableHead className="w-36">Scope</TableHead>
      ) : null}
    </TableRow>
  )
  const header = <TableHeader>{headerRows}</TableHeader>

  const renderRow = (log: LogDoc) => {
    const isAnchor = String(anchorNanos ?? "") === log.tsNanos
    return (
    <TableRow
      key={logKey(log)}
      data-anchor={isAnchor ? "true" : undefined}
      data-state={isAnchor ? "selected" : undefined}
      className="cursor-pointer"
      onClick={() => {
        setSelected(log)
        setFieldSearch("")
      }}
    >
      <TableCell className="font-mono text-xs whitespace-nowrap">
        {formatTimeInRange(log.tsNanos, range)}
      </TableCell>
      <TableCell>
        <SeverityBadge log={log} />
      </TableCell>
      {visible.has("service") ? (
        <TableCell className="max-w-36 truncate text-muted-foreground">
          {log.service}
        </TableCell>
      ) : null}
      <TableCell className="max-w-xl truncate font-mono text-xs">
        {log.body}
      </TableCell>
      {visible.has("trace") ? (
        <TableCell>
          {log.traceId ? (
            <Link
              to="/traces/$traceId"
              params={{ traceId: log.traceId }}
              onClick={(event) => event.stopPropagation()}
              className="rounded-full border border-border/70 px-2 py-1 font-mono text-[11px] text-muted-foreground hover:text-foreground"
            >
              {log.traceId.slice(0, 8)}
            </Link>
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
    logs.length > 100 ? (
      <div
        ref={parentRef}
        data-slot="table-container"
        className="relative max-h-[min(70vh,720px)] w-full overflow-auto rounded-xl corner-squircle shadow-(--custom-shadow) dark:shadow-(--custom-shadow)"
      >
        <table
          data-slot="table"
          className="w-full caption-bottom rounded-xl corner-squircle bg-card/40 text-sm shadow-(--custom-shadow) dark:shadow-(--custom-shadow)"
        >
          <TableHeader className="sticky top-0 z-10 bg-card/95 backdrop-blur supports-backdrop-filter:bg-card/75">
            {headerRows}
          </TableHeader>
          <TableBody>
            {paddingTop > 0 ? (
              <tr aria-hidden="true">
                <td
                  colSpan={columnCount}
                  className="border-0 p-0"
                  style={{ height: paddingTop }}
                />
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
              {selected
                ? `${selected.service} · ${formatDateTime(selected.tsNanos)}`
                : ""}
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
                  <Link
                    to="/traces/$traceId"
                    params={{ traceId: selected.traceId }}
                    className="rounded-full border border-border/70 px-2 py-1 font-mono text-xs hover:bg-muted"
                  >
                    trace {selected.traceId.slice(0, 12)}
                  </Link>
                ) : null}
                {selected.runId ? (
                  <Link
                    to="/runs/$runId"
                    params={{ runId: selected.runId }}
                    className="rounded-full border border-border/70 px-2 py-1 font-mono text-xs hover:bg-muted"
                  >
                    run {selected.runId.slice(0, 12)}
                  </Link>
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
                            className="underline underline-offset-4"
                          >
                            {value}
                          </Link>
                        ) : key === "run_id" ? (
                          <Link
                            to="/runs/$runId"
                            params={{ runId: value }}
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

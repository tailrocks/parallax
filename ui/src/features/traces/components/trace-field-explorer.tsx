import { Link } from "@tanstack/react-router"
import {
  IconAlertTriangle,
  IconDatabase,
  IconExternalLink,
  IconFilter,
  IconSearch,
} from "@tabler/icons-react"
import { useEffect, useMemo, useState } from "react"
import { CopyButton } from "@/shared/console/copy-button"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet"
import { gqlString, graphql } from "@/platform/graphql/transport"
import type { FieldKey, FieldStats, FieldValueCount } from "../model/wire"
import { formatCount, formatPercent } from "@/shared/format"
import type { ResolvedRange } from "@/domain/range"
import { cn } from "@/lib/utils"

interface FieldExplorerProps {
  range: ResolvedRange
  service?: string | undefined
  onApplyService: (service: string | undefined) => void
}

function count(value: string): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : 0
}

function escapeSqlIdent(value: string): string {
  return value.replace(/"/g, '""')
}

function sqlLiteral(value: string): string {
  return `'${value.replace(/'/g, "''")}'`
}

function fieldColumn(key: string): string {
  if (key === "resource.service.name" || key === "service.name") {
    return `"service_name"`
  }
  if (key.startsWith("resource.")) {
    return `"resource_attributes.${escapeSqlIdent(key.slice("resource.".length))}"`
  }
  return `"span_attributes.${escapeSqlIdent(key)}"`
}

function fieldValueSql(
  key: string,
  value: string,
  range: ResolvedRange,
  operator: "=" | "<>"
): string {
  const column = fieldColumn(key)
  return `SELECT trace_id, span_id, service_name, span_name,
       CAST("timestamp" AS BIGINT) AS ts_nanos,
       CAST(${column} AS STRING) AS field_value
FROM opentelemetry_traces
WHERE "timestamp" >= ${range.fromNanos}
  AND "timestamp" <= ${range.toNanos}
  AND CAST(${column} AS STRING) ${operator} ${sqlLiteral(value)}
ORDER BY "timestamp" DESC
LIMIT 100`
}

function isServiceField(key: string): boolean {
  return key === "resource.service.name" || key === "service.name"
}

function CoverageBar({ value }: { value: number }) {
  return (
    <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
      <div
        className="h-full rounded-full bg-emerald-500"
        style={{ width: `${Math.max(0, Math.min(1, value)) * 100}%` }}
      />
    </div>
  )
}

function FieldValueRow({
  fieldKey,
  range,
  value,
  max,
  onApplyService,
}: {
  fieldKey: string
  range: ResolvedRange
  value: FieldValueCount
  max: number
  onApplyService: (service: string | undefined) => void
}) {
  const includeSql = fieldValueSql(fieldKey, value.value, range, "=")
  const excludeSql = fieldValueSql(fieldKey, value.value, range, "<>")
  const n = count(value.count)
  return (
    <div className="flex flex-col gap-2 rounded-md border p-3">
      <div className="flex items-center justify-between gap-3">
        <div className="min-w-0">
          <div className="truncate font-mono text-xs" title={value.value}>
            {value.value}
          </div>
          <div className="mt-1 flex items-center gap-2">
            <CoverageBar value={max > 0 ? n / max : 0} />
            <span className="w-12 shrink-0 text-right text-xs text-muted-foreground tabular-nums">
              {formatCount(n)}
            </span>
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-1">
          {isServiceField(fieldKey) ? (
            <Button
              type="button"
              variant="outline"
              size="xs"
              onClick={() => onApplyService(value.value)}
            >
              <IconFilter />
              Include
            </Button>
          ) : null}
          <CopyButton value={includeSql} />
          <Button
            variant="ghost"
            size="icon-xs"
            render={
              <Link
                to="/sql"
                search={{ query: includeSql }}
                aria-label={`Open SQL for ${value.value}`}
              />
            }
          >
            <IconExternalLink />
          </Button>
          <Button
            variant="ghost"
            size="icon-xs"
            render={
              <Link
                to="/sql"
                search={{ query: excludeSql }}
                aria-label={`Open exclusion SQL for ${value.value}`}
              />
            }
          >
            <IconSearch />
          </Button>
        </div>
      </div>
    </div>
  )
}

export function FieldExplorer({ range, service, onApplyService }: FieldExplorerProps) {
  const [open, setOpen] = useState(false)
  const [keys, setKeys] = useState<FieldKey[]>([])
  const [selectedKey, setSelectedKey] = useState<string | null>(null)
  const [stats, setStats] = useState<FieldStats | null>(null)
  const [loadingKeys, setLoadingKeys] = useState(false)
  const [loadingStats, setLoadingStats] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!open) return
    const controller = new AbortController()
    setLoadingKeys(true)
    setError(null)
    void graphql<{ fieldKeys: FieldKey[] }>(
      `{
        fieldKeys(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") {
          key namespace source rowCount nonNullCount coverage isIdentifier
        }
      }`,
      { signal: controller.signal }
    )
      .then((data) => {
        setKeys(data.fieldKeys)
        setSelectedKey((current) =>
          current && data.fieldKeys.some((field) => field.key === current)
            ? current
            : (data.fieldKeys[0]?.key ?? null)
        )
      })
      .catch((err: unknown) => {
        if (!controller.signal.aborted) {
          setError(err instanceof Error ? err.message : String(err))
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoadingKeys(false)
      })
    return () => controller.abort()
  }, [open, range.fromNanos, range.toNanos])

  useEffect(() => {
    if (!open || !selectedKey) {
      setStats(null)
      return
    }
    const controller = new AbortController()
    setLoadingStats(true)
    setError(null)
    const serviceArg = service ? `, service: "${gqlString(service)}"` : ""
    void graphql<{ fieldStats: FieldStats }>(
      `{
        fieldStats(
          key: "${gqlString(selectedKey)}",
          fromNanos: "${range.fromNanos}",
          toNanos: "${range.toNanos}"${serviceArg}
        ) {
          key namespace source rowCount nonNullCount distinctCount coverage capped isIdentifier
          topValues { value count }
        }
      }`,
      { signal: controller.signal }
    )
      .then((data) => setStats(data.fieldStats))
      .catch((err: unknown) => {
        if (!controller.signal.aborted) {
          setError(err instanceof Error ? err.message : String(err))
          setStats(null)
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoadingStats(false)
      })
    return () => controller.abort()
  }, [open, selectedKey, range.fromNanos, range.toNanos, service])

  const grouped = useMemo(() => {
    const groups = new Map<string, FieldKey[]>()
    for (const key of keys) {
      const group = groups.get(key.namespace) ?? []
      group.push(key)
      groups.set(key.namespace, group)
    }
    return [...groups.entries()].map(([namespace, fields]) => ({
      namespace,
      fields: fields.sort((a, b) => b.coverage - a.coverage || a.key.localeCompare(b.key)),
    }))
  }, [keys])

  const maxValueCount = Math.max(0, ...(stats?.topValues ?? []).map((value) => count(value.count)))

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger render={<Button type="button" variant="outline" size="sm" />}>
        <IconDatabase />
        Fields
      </SheetTrigger>
      <SheetContent className="w-full gap-0 p-0 sm:max-w-3xl">
        <SheetHeader className="border-b">
          <SheetTitle>Fields</SheetTitle>
          <SheetDescription>{service ? `service:${service}` : "all services"}</SheetDescription>
        </SheetHeader>
        <div className="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[18rem_1fr]">
          <div className="min-h-0 border-b md:border-r md:border-b-0">
            <ScrollArea className="h-[36vh] md:h-full">
              <div className="flex flex-col gap-3 p-4">
                {loadingKeys ? (
                  <p className="text-sm text-muted-foreground">Loading...</p>
                ) : grouped.length === 0 ? (
                  <p className="text-sm text-muted-foreground">No fields</p>
                ) : (
                  grouped.map((group) => (
                    <div key={group.namespace} className="flex flex-col gap-1">
                      <div className="px-1 text-xs font-medium text-muted-foreground uppercase">
                        {group.namespace}
                      </div>
                      {group.fields.map((field) => (
                        <button
                          key={field.key}
                          type="button"
                          className={cn(
                            "flex flex-col gap-1 rounded-md px-2 py-2 text-left transition outline-none hover:bg-muted focus-visible:ring-[1.5px] focus-visible:ring-ring/50",
                            selectedKey === field.key && "bg-muted"
                          )}
                          onClick={() => setSelectedKey(field.key)}
                        >
                          <div className="flex w-full items-center gap-2">
                            <span className="min-w-0 flex-1 truncate font-mono text-xs">
                              {field.key}
                            </span>
                            {field.source === "RESOURCE" ? (
                              <Badge variant="outline">res</Badge>
                            ) : null}
                            {field.isIdentifier ? <Badge variant="amber">id</Badge> : null}
                          </div>
                          <div className="flex items-center gap-2">
                            <CoverageBar value={field.coverage} />
                            <span className="w-10 text-right text-xs text-muted-foreground tabular-nums">
                              {formatPercent(field.coverage)}
                            </span>
                          </div>
                        </button>
                      ))}
                    </div>
                  ))
                )}
              </div>
            </ScrollArea>
          </div>
          <ScrollArea className="min-h-0">
            <div className="flex flex-col gap-4 p-4">
              {error ? (
                <div className="flex items-center gap-2 rounded-md border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
                  <IconAlertTriangle className="size-4" />
                  {error}
                </div>
              ) : null}
              {loadingStats ? (
                <p className="text-sm text-muted-foreground">Loading...</p>
              ) : stats ? (
                <>
                  <div className="flex flex-wrap items-center gap-2">
                    <h3 className="min-w-0 flex-1 truncate font-mono text-sm">{stats.key}</h3>
                    <Badge variant="outline">{stats.source.toLowerCase()}</Badge>
                    {stats.isIdentifier ? <Badge variant="amber">identifier</Badge> : null}
                    {stats.capped ? <Badge variant="blue">capped</Badge> : null}
                  </div>
                  <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
                    <div className="rounded-md border p-3">
                      <div className="text-xs text-muted-foreground">Coverage</div>
                      <div className="mt-1 text-lg font-medium tabular-nums">
                        {formatPercent(stats.coverage)}
                      </div>
                    </div>
                    <div className="rounded-md border p-3">
                      <div className="text-xs text-muted-foreground">Rows</div>
                      <div className="mt-1 text-lg font-medium tabular-nums">
                        {formatCount(count(stats.nonNullCount))}
                      </div>
                    </div>
                    <div className="rounded-md border p-3">
                      <div className="text-xs text-muted-foreground">Distinct</div>
                      <div className="mt-1 text-lg font-medium tabular-nums">
                        {formatCount(count(stats.distinctCount))}
                      </div>
                    </div>
                    <div className="rounded-md border p-3">
                      <div className="text-xs text-muted-foreground">Window</div>
                      <div className="mt-1 text-lg font-medium tabular-nums">
                        {formatCount(count(stats.rowCount))}
                      </div>
                    </div>
                  </div>
                  <div className="flex flex-col gap-2">
                    <div className="text-sm font-medium">Top values</div>
                    {stats.topValues.length > 0 ? (
                      stats.topValues.map((value) => (
                        <FieldValueRow
                          key={value.value}
                          fieldKey={stats.key}
                          range={range}
                          value={value}
                          max={maxValueCount}
                          onApplyService={onApplyService}
                        />
                      ))
                    ) : (
                      <p className="text-sm text-muted-foreground">No values</p>
                    )}
                  </div>
                </>
              ) : (
                <p className="text-sm text-muted-foreground">Select a field</p>
              )}
            </div>
          </ScrollArea>
        </div>
      </SheetContent>
    </Sheet>
  )
}

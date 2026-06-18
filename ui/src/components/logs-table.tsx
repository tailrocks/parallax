import { Link } from "@tanstack/react-router"
import { useMemo, useState } from "react"
import { Badge } from "@/components/ui/badge"
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

/** One log row, with every field the doc viewer needs. Shared by the Logs page
 * and the run detail page so both render logs identically. */
export interface LogDoc {
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

export function severityVariant(
  num: number
): "destructive" | "secondary" | "outline" {
  if (num >= 17) return "destructive"
  if (num >= 13) return "secondary"
  return "outline"
}

export function formatTime(tsNanos: string): string {
  return new Date(Number(BigInt(tsNanos) / 1_000_000n)).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  })
}

/** Flatten one log into ordered field/value rows for the doc viewer. */
function docFields(log: LogDoc): Array<[string, string]> {
  const rows: Array<[string, string]> = [
    [
      "@timestamp",
      new Date(Number(BigInt(log.tsNanos) / 1_000_000n)).toISOString(),
    ],
    ["severity", `${log.severityText} (${log.severityNum})`],
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

/** The shared logs table: a Time/Severity/Service/Body grid whose rows open a
 * searchable field-level document viewer. Render order is the caller's — pass
 * rows already sorted (newest first on every surface). */
export function LogsTable({ logs }: { logs: LogDoc[] }) {
  const [selected, setSelected] = useState<LogDoc | null>(null)
  const [fieldSearch, setFieldSearch] = useState("")

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

  return (
    <>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-28">Time</TableHead>
            <TableHead className="w-24">Severity</TableHead>
            <TableHead className="w-36">Service</TableHead>
            <TableHead>Body</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {logs.map((log, index) => (
            <TableRow
              key={`${log.tsNanos}-${index}`}
              className="cursor-pointer"
              onClick={() => {
                setSelected(log)
                setFieldSearch("")
              }}
            >
              <TableCell className="font-mono text-xs">
                {formatTime(log.tsNanos)}
              </TableCell>
              <TableCell>
                <Badge variant={severityVariant(log.severityNum)}>
                  {log.severityText || "—"}
                </Badge>
              </TableCell>
              <TableCell className="truncate">{log.service}</TableCell>
              <TableCell className="max-w-xl truncate font-mono text-xs">
                {log.body}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Sheet
        open={selected !== null}
        onOpenChange={(open) => {
          if (!open) setSelected(null)
        }}
      >
        <SheetContent className="w-full overflow-y-auto sm:max-w-xl">
          <SheetHeader>
            <SheetTitle>Log document</SheetTitle>
            <SheetDescription>
              {selected
                ? `${selected.service} · ${formatTime(selected.tsNanos)}`
                : ""}
            </SheetDescription>
          </SheetHeader>
          {selected ? (
            <div className="space-y-3 px-4 pb-6">
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
                      <TableCell className="align-top font-medium">
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

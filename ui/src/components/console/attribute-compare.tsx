import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { AttributeCompareRow } from "@/lib/api"
import { formatPercent } from "@/lib/format"
import { cn } from "@/lib/utils"

function count(value: string): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : 0
}

function share(part: string, total: string): number {
  const numerator = count(part)
  const denominator = count(total)
  return denominator > 0 ? numerator / denominator : 0
}

function identifierKey(key: string): boolean {
  const lower = key.toLowerCase()
  return lower.endsWith(".id") || lower.endsWith("_id") || lower === "id"
}

function ShareBar({
  value,
  muted,
}: {
  value: number
  muted?: boolean
}) {
  const pct = Math.max(0, Math.min(1, value)) * 100
  return (
    <span className="flex min-w-32 items-center gap-2">
      <span className="h-2 flex-1 rounded-full bg-muted">
        <span
          className={cn(
            "block h-2 rounded-full",
            muted ? "bg-muted-foreground/45" : "bg-primary",
          )}
          style={{ width: `${pct}%` }}
        />
      </span>
      <span className="w-12 text-right font-mono text-xs tabular-nums">
        {formatPercent(value)}
      </span>
    </span>
  )
}

export function AttributeComparePanel({
  rows,
}: {
  rows: AttributeCompareRow[]
}) {
  if (rows.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        No overrepresented span attributes in this window.
      </p>
    )
  }

  return (
    <Table density="compact">
      <TableHeader>
        <TableRow>
          <TableHead className="w-16">Rank</TableHead>
          <TableHead>Field</TableHead>
          <TableHead>Value</TableHead>
          <TableHead align="right">Selected</TableHead>
          <TableHead align="right">Baseline</TableHead>
          <TableHead align="right">Lift</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((row, index) => {
          const selectedShare = share(row.selectedCount, row.selectedTotal)
          const baselineShare = share(row.baselineCount, row.baselineTotal)
          return (
            <TableRow key={`${row.key}-${row.value}`}>
              <TableCell className="font-mono text-xs tabular-nums">
                #{index + 1}
              </TableCell>
              <TableCell>
                <span className="flex min-w-0 flex-wrap items-center gap-1.5">
                  <span className="break-all font-mono text-xs">{row.key}</span>
                  {identifierKey(row.key) ? (
                    <Badge variant="outline">identifier</Badge>
                  ) : null}
                </span>
              </TableCell>
              <TableCell className="max-w-72 whitespace-normal break-words font-mono text-xs">
                {row.value}
              </TableCell>
              <TableCell align="right">
                <ShareBar value={selectedShare} />
              </TableCell>
              <TableCell align="right">
                <ShareBar value={baselineShare} muted />
              </TableCell>
              <TableCell align="right">
                <Badge variant="secondary">{formatPercent(row.score)}</Badge>
              </TableCell>
            </TableRow>
          )
        })}
      </TableBody>
    </Table>
  )
}

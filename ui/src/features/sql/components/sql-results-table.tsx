import { useMemo } from "react"
import { Link } from "@tanstack/react-router"

import { targetForCell } from "@/features/sql/model/sql-cell-target"
import type { SqlResult } from "@/features/sql/model/sql-result"
import { normalizeColumn, parseResultRow } from "@/features/sql/model/sql-row"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

export function SqlResultsTable({ result }: { result: SqlResult }) {
  const parsedRows = useMemo(() => result.rows.map(parseResultRow), [result.rows])

  return (
    <ScrollArea className="max-h-[520px] overflow-auto">
      <Table>
        <TableHeader className="sticky top-0 bg-card">
          <TableRow>
            {result.columns.map((column) => (
              <TableHead key={column}>{column}</TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {parsedRows.map((cells, rowIndex) => (
            <SqlResultRow key={rowIndex} columns={result.columns} cells={cells} />
          ))}
        </TableBody>
      </Table>
    </ScrollArea>
  )
}

function SqlResultRow({
  columns,
  cells,
}: {
  columns: readonly string[]
  cells: readonly string[]
}) {
  const rowByColumn = Object.fromEntries(
    columns.map((column, index) => [normalizeColumn(column), cells[index] ?? ""])
  )
  return (
    <TableRow>
      {cells.map((cell, cellIndex) => {
        const column = columns[cellIndex] ?? ""
        const target = targetForCell(column, cell, rowByColumn)
        return (
          <TableCell key={cellIndex} className="max-w-md truncate font-mono text-xs" title={cell}>
            {target ? (
              <Link to={target.to} params={target.params} className="underline underline-offset-4">
                {cell}
              </Link>
            ) : (
              cell
            )}
          </TableCell>
        )
      })}
    </TableRow>
  )
}

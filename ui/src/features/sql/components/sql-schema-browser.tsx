import { IconTable } from "@tabler/icons-react"

import type { SchemaColumn } from "@/features/sql/model/sql-row"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"

export function SqlSchemaBrowser({
  schema,
  openTable,
  onToggleTable,
  onInsertIdentifier,
}: {
  schema: Map<string, SchemaColumn[]>
  openTable: string | null
  onToggleTable: (table: string) => void
  onInsertIdentifier: (identifier: string) => void
}) {
  return (
    <Card className="h-fit">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-sm">
          <IconTable />
          Tables
        </CardTitle>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[520px]">
          <ul className="flex flex-col gap-1 text-xs">
            {[...schema.keys()].map((table) => (
              <li key={table}>
                <button
                  type="button"
                  className="font-mono hover:underline"
                  onClick={() => onToggleTable(table)}
                >
                  {table}
                </button>
                {openTable === table ? (
                  <ul className="mt-1 ml-3 flex flex-col gap-0.5">
                    {(schema.get(table) ?? []).map((column) => (
                      <li key={column.name}>
                        <button
                          type="button"
                          className="font-mono text-muted-foreground hover:text-foreground"
                          onClick={() => onInsertIdentifier(column.name)}
                        >
                          {column.name}{" "}
                          <span className="opacity-60">{column.dataType.toLowerCase()}</span>
                        </button>
                      </li>
                    ))}
                  </ul>
                ) : null}
              </li>
            ))}
          </ul>
        </ScrollArea>
      </CardContent>
    </Card>
  )
}

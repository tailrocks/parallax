export function normalizeColumn(column: string): string {
  return column
    .trim()
    .replace(/^"+|"+$/g, "")
    .toLowerCase()
}

export function displayCell(cell: unknown): string {
  return typeof cell === "string" ? cell : JSON.stringify(cell)
}

/** Parse a JSON row string into display cells; malformed → empty row. */
export function parseResultRow(row: string): string[] {
  try {
    const cells: unknown = JSON.parse(row)
    return Array.isArray(cells) ? cells.map(displayCell) : []
  } catch {
    return []
  }
}

export type SchemaColumn = {
  readonly name: string
  readonly dataType: string
}

/**
 * Schema-discovery row acceptance matches baseline: skip non-arrays and
 * falsey first-three cells; truthy non-strings still pass (presentation may
 * coerce). Returns null when the row should be skipped.
 */
export function parseSchemaRow(row: string): { table: string; column: SchemaColumn } | null {
  try {
    const cells: unknown = JSON.parse(row)
    if (!Array.isArray(cells)) return null
    const [table, column, dataType] = cells as Array<string | undefined>
    if (!table || !column || !dataType) return null
    return {
      table: String(table),
      column: { name: String(column), dataType: String(dataType) },
    }
  } catch {
    return null
  }
}

export function groupSchemaRows(rows: readonly string[]): Map<string, SchemaColumn[]> {
  const grouped = new Map<string, SchemaColumn[]>()
  for (const row of rows) {
    const parsed = parseSchemaRow(row)
    if (!parsed) continue
    const list = grouped.get(parsed.table) ?? []
    list.push(parsed.column)
    grouped.set(parsed.table, list)
  }
  return grouped
}

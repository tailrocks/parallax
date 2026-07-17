export type SqlResult = {
  readonly columns: readonly string[]
  readonly rows: readonly string[]
  readonly rowCount: number
  readonly truncated: boolean
}

export function mapSqlResult(raw: {
  readonly columns: readonly string[]
  readonly rows: readonly string[]
  readonly rowCount: number
  readonly truncated: boolean
}): SqlResult {
  return {
    columns: raw.columns,
    rows: raw.rows,
    rowCount: raw.rowCount,
    truncated: raw.truncated,
  }
}

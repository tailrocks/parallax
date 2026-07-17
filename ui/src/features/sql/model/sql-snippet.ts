export type SqlSnippet = {
  readonly id: string
  readonly name: string
  readonly page: string
  readonly state: string
  readonly updatedAtNanos: string
}

export const SQL_SNIPPET_PAGE = "/sql"

export function mapSqlSnippet(raw: {
  readonly id: string
  readonly name: string
  readonly page: string
  readonly state: string
  readonly updatedAtNanos: string
}): SqlSnippet {
  return {
    id: raw.id,
    name: raw.name,
    page: raw.page,
    state: raw.state,
    updatedAtNanos: raw.updatedAtNanos,
  }
}

import { SqlResultsTable } from "@/features/sql/components/sql-results-table"
import type { SqlResult } from "@/features/sql/model/sql-result"

export function SqlResultBody({ result }: { result: SqlResult }) {
  return (
    <>
      {result.truncated ? (
        <p className="mb-3 text-sm text-amber-700 dark:text-amber-300">
          Result capped at 2,000 rows — refine the query or add LIMIT/ORDER BY.
        </p>
      ) : null}
      <SqlResultsTable result={result} />
    </>
  )
}

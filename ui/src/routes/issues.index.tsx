import { createFileRoute } from "@tanstack/react-router"

import { IssuesPage, loadIssues, validateIssuesSearch } from "@/features/issues"
import { resolveRangeSearch } from "@/domain/time-range/range"

export const Route = createFileRoute("/issues/")({
  validateSearch: validateIssuesSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadIssues(deps, resolveRangeSearch(deps)),
  component: IssuesRoute,
})

function IssuesRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return <IssuesPage data={data} search={search} />
}

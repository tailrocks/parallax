import { createFileRoute } from "@tanstack/react-router"

import {
  IssueDetailRoutePage,
  loadIssueDetail,
  validateIssuesSearch,
} from "@/features/issues"
import { resolveRangeSearch } from "@/lib/range"

export const Route = createFileRoute("/issues/$fingerprint")({
  validateSearch: validateIssuesSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ params, deps }) =>
    loadIssueDetail(params.fingerprint, resolveRangeSearch(deps)),
  component: IssueDetailRoute,
})

function IssueDetailRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return <IssueDetailRoutePage data={data} search={search} />
}

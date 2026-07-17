import { createFileRoute } from "@tanstack/react-router"

import { TestsPage, loadTests, validateTestsSearch } from "@/features/tests"
import { resolveRangeSearch } from "@/domain/time-range/range"

export const Route = createFileRoute("/tests/")({
  validateSearch: validateTestsSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadTests(deps, resolveRangeSearch(deps)),
  component: TestsRoute,
})

function TestsRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return <TestsPage data={data} search={search} />
}

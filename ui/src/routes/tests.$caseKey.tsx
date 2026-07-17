import { createFileRoute } from "@tanstack/react-router"

import { TestCaseDetailRoutePage, loadTestCaseDetail, validateTestsSearch } from "@/features/tests"

export const Route = createFileRoute("/tests/$caseKey")({
  validateSearch: validateTestsSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ params }) => loadTestCaseDetail(params.caseKey),
  component: TestCaseDetailRoute,
})

function TestCaseDetailRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return <TestCaseDetailRoutePage data={data} search={search} />
}

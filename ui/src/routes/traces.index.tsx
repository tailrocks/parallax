import { createFileRoute } from "@tanstack/react-router"

import { TracesPage, loadTraces, validateTracesSearch } from "@/features/traces"

export const Route = createFileRoute("/traces/")({
  validateSearch: validateTracesSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadTraces(deps),
  component: TracesRoute,
})

function TracesRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return <TracesPage data={data} search={search} />
}

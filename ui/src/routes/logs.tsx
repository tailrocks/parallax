import { createFileRoute } from "@tanstack/react-router"

import { LogsPage, loadLogs, validateLogsSearch } from "@/features/logs"

export const Route = createFileRoute("/logs")({
  validateSearch: validateLogsSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadLogs(deps),
  component: LogsRoute,
})

function LogsRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return <LogsPage data={data} search={search} />
}

import { createFileRoute } from "@tanstack/react-router"

import {
  InvocationsPage,
  loadInvocations,
  validateInvocationsSearch,
} from "@/features/invocations"

export const Route = createFileRoute("/invocations/")({
  validateSearch: validateInvocationsSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadInvocations(deps),
  component: InvocationsRoute,
})

function InvocationsRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return <InvocationsPage data={data} search={search} />
}

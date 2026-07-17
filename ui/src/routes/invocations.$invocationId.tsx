import { createFileRoute } from "@tanstack/react-router"

import {
  InvocationHubPage,
  loadInvocationHub,
  validateHubSearch,
} from "@/features/invocations"

export const Route = createFileRoute("/invocations/$invocationId")({
  validateSearch: validateHubSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ params }) => loadInvocationHub(params.invocationId),
  component: InvocationHubRoute,
})

function InvocationHubRoute() {
  const data = Route.useLoaderData()
  const { invocationId } = Route.useParams()
  const search = Route.useSearch()
  return (
    <InvocationHubPage
      invocationId={invocationId}
      data={data}
      search={search}
    />
  )
}

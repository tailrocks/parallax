import { createFileRoute } from "@tanstack/react-router"

import { InvestigationsPage } from "@/features/investigations"
import { investigationsListQueryOptions } from "@/features/investigations/queries/options"

export const Route = createFileRoute("/investigations/")({
  loader: ({ context: { queryClient } }) =>
    queryClient.ensureQueryData(investigationsListQueryOptions()).then((investigations) => ({
      investigations,
    })),
  component: InvestigationsRoute,
})

function InvestigationsRoute() {
  const { investigations } = Route.useLoaderData()
  return <InvestigationsPage investigations={investigations} />
}

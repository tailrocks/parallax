import { createFileRoute, notFound } from "@tanstack/react-router"

import { navItem } from "@/shared/navigation"
import { InvestigationDetailPage } from "@/features/investigations"
import { investigationDetailQueryOptions } from "@/features/investigations/queries/options"

export const Route = createFileRoute("/investigations/$investigationId")({
  loader: async ({ context: { queryClient }, params }) => {
    const investigation = await queryClient.ensureQueryData(
      investigationDetailQueryOptions(params.investigationId)
    )
    if (!investigation) throw notFound()
    return { investigation }
  },
  component: InvestigationDetailRoute,
})

function InvestigationDetailRoute() {
  const { investigation } = Route.useLoaderData()
  const back = navItem("/investigations")
  return <InvestigationDetailPage investigation={investigation} {...(back ? { back } : {})} />
}

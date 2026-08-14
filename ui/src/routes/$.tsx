import { createFileRoute } from "@tanstack/react-router"

import { RouteNotFoundPanel } from "@/shared/route-not-found"

// Unmatched URLs must be a real child of root so SPA hydration uses the same
// document/shell tree as every other surface (React #418).
export const Route = createFileRoute("/$")({
  component: RouteNotFoundPanel,
})

// App-owned router composition entry (Plan 143).
// TanStack Start registers via ui/src/router.tsx; this module is the product owner.
import { getRouter } from "@/router"

export const createRouter = getRouter
export type AppRouter = ReturnType<typeof getRouter>

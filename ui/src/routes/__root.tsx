import {
  ClientOnly,
  HeadContent,
  Outlet,
  Scripts,
  createRootRouteWithContext,
  redirect,
  useRouterState,
} from "@tanstack/react-router"
import { ThemeProvider } from "next-themes"

import appCss from "../styles.css?url"
import { ParallaxShell, RouteErrorPanel, RouteNotFoundPanel, RoutePendingPanel } from "@/layout"
import { AppQueryProvider } from "@/platform/query/provider"
import type { AppRouterContext } from "@/router-context"
import { getApiToken } from "@/platform/auth/api-token"
import { loadAuthStatus } from "@/platform/auth/status"

export const Route = createRootRouteWithContext<AppRouterContext>()({
  beforeLoad: async ({ location }) => {
    if (typeof window === "undefined") return
    if (location.pathname === "/login") return
    const status = await loadAuthStatus()
    if (status.loginEnabled && !getApiToken()) {
      throw redirect({ to: "/login" })
    }
  },
  head: () => ({
    meta: [
      {
        charSet: "utf-8",
      },
      {
        name: "viewport",
        content: "width=device-width, initial-scale=1",
      },
      {
        title: "Parallax",
      },
    ],
    links: [
      {
        rel: "stylesheet",
        href: appCss,
      },
    ],
  }),
  errorComponent: RouteErrorPanel,
  pendingComponent: RoutePendingPanel,
  notFoundComponent: HydrationSafeNotFound,
  // MatchInner replaces `component` on not-found. Keep document/theme/shell here
  // so SSR and client 404 trees stay identical (React #418).
  shellComponent: RootShell,
  component: RootOutlet,
})

function RootShell({ children }: { children: React.ReactNode }) {
  const { queryClient } = Route.useRouteContext()
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  return (
    <RootDocument>
      <AppQueryProvider client={queryClient}>
        <ThemeProvider attribute="class" defaultTheme="dark" enableSystem disableTransitionOnChange>
          {pathname === "/login" ? children : <ParallaxShell>{children}</ParallaxShell>}
        </ThemeProvider>
      </AppQueryProvider>
    </RootDocument>
  )
}

function RootOutlet() {
  return <Outlet />
}

function HydrationSafeNotFound() {
  return (
    <ClientOnly fallback={<RoutePendingPanel />}>
      <RouteNotFoundPanel />
    </ClientOnly>
  )
}

function RootDocument({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <HeadContent />
      </head>
      <body>
        {children}
        <Scripts />
      </body>
    </html>
  )
}

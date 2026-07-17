import { HeadContent, Outlet, Scripts, createRootRouteWithContext } from "@tanstack/react-router"
import { ThemeProvider } from "next-themes"

import appCss from "../styles.css?url"
import { ParallaxShell, RouteErrorPanel, RouteNotFoundPanel, RoutePendingPanel } from "@/layout"
import { AppQueryProvider } from "@/platform/query/provider"
import type { AppRouterContext } from "@/router-context"

export const Route = createRootRouteWithContext<AppRouterContext>()({
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
  notFoundComponent: RouteNotFoundPanel,
  component: RootOutlet,
})

function RootOutlet() {
  const { queryClient } = Route.useRouteContext()
  return (
    <RootDocument>
      <AppQueryProvider client={queryClient}>
        <ThemeProvider attribute="class" defaultTheme="dark" enableSystem disableTransitionOnChange>
          <ParallaxShell>
            <Outlet />
          </ParallaxShell>
        </ThemeProvider>
      </AppQueryProvider>
    </RootDocument>
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

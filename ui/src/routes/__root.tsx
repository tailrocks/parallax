import {
  HeadContent,
  Outlet,
  Scripts,
  createRootRoute,
} from "@tanstack/react-router"

import appCss from "../styles.css?url"
import { ParallaxShell } from "@/components/parallax-shell"
import {
  RouteErrorPanel,
  RouteNotFoundPanel,
  RoutePendingPanel,
} from "@/components/route-fallbacks"

export const Route = createRootRoute({
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
  return (
    <RootDocument>
      <ParallaxShell>
        <Outlet />
      </ParallaxShell>
    </RootDocument>
  )
}

function RootDocument({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
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

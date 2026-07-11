/* @vitest-environment jsdom */

import { cleanup, fireEvent, render, screen } from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import { afterEach, describe, expect, it, vi } from "vitest"

import { nav } from "@/components/nav"
import { PageHeader } from "@/components/page-header"
import { ParallaxShell } from "@/components/parallax-shell"
import { RouteErrorPanel } from "@/components/route-fallbacks"
import { graphql } from "@/lib/api"

vi.mock("@/lib/api", () => ({
  graphql: vi.fn().mockImplementation(async (query: string) => {
    if (query.includes("{ services }")) return { services: [] }
    if (query.includes("tracesPage")) {
      return { tracesPage: { items: [] }, runs: [] }
    }
    return { dashboards: [] }
  }),
}))

class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}

globalThis.ResizeObserver = ResizeObserverMock
window.HTMLElement.prototype.scrollIntoView = vi.fn()

function renderWithRouter(component: React.ReactNode, initialEntries = ["/"]) {
  window.scrollTo = () => {}
  window.matchMedia = () =>
    ({
      matches: false,
      media: "",
      onchange: null,
      addListener() {},
      removeListener() {},
      addEventListener() {},
      removeEventListener() {},
      dispatchEvent: () => true,
    }) as MediaQueryList
  const rootRoute = createRootRoute({
    component: Outlet,
  })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => component,
  })
  const dashboardsRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "dashboards",
    component: () => component,
  })
  const routeTree = rootRoute.addChildren([indexRoute, dashboardsRoute])
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries }),
  })

  return render(<RouterProvider router={router} />)
}

describe("shell primitives", () => {
  afterEach(() => {
    cleanup()
  })

  it("renders PageHeader breadcrumb shape", async () => {
    const item = nav[0]!
    renderWithRouter(
      <PageHeader
        title="Detail"
        back={{
          href: item.href,
          label: item.label,
          icon: item.icon,
          ...(item.iconClassName ? { iconClassName: item.iconClassName } : {}),
        }}
      />
    )

    expect(await screen.findByRole("link", { name: item.label })).toBeTruthy()
    expect(
      screen.getByRole("heading", {
        name: new RegExp(`${item.label}detail`, "i"),
      }).className
    ).toContain("text-base")
  })

  it("defines icons for every nav item", () => {
    expect(nav.length).toBeGreaterThan(0)
    for (const item of nav) {
      expect(item.icon).toBeDefined()
      expect(item.activeIcon).toBeDefined()
    }
  })

  it("keeps the styled error fallback inside the shell", async () => {
    renderWithRouter(
      <ParallaxShell>
        <RouteErrorPanel error={new Error("offline")} reset={() => {}} />
      </ParallaxShell>
    )

    expect(await screen.findByLabelText("Parallax home")).toBeTruthy()
    fireEvent.click(screen.getByRole("button", { name: /search/i }))
    expect(await screen.findByPlaceholderText(/search pages/i)).toBeTruthy()
    expect(screen.getByText("Parallax API did not answer")).toBeTruthy()
    expect(screen.getByText("offline")).toBeTruthy()
  })

  it("surfaces dashboard navigation load failures", async () => {
    vi.mocked(graphql).mockRejectedValueOnce(new Error("dashboard query failed"))

    renderWithRouter(
      <ParallaxShell>
        <div>Dashboard content</div>
      </ParallaxShell>,
      ["/dashboards"]
    )

    expect(await screen.findByText("dashboard query failed")).toBeTruthy()
  })

  it("keeps the sidebar shortcut independent from the command palette shortcut", async () => {
    const { container } = renderWithRouter(
      <ParallaxShell>
        <div>Dashboard content</div>
      </ParallaxShell>
    )

    await screen.findByText("Dashboard content")
    const sidebar = container.querySelector('[data-slot="sidebar"]') as Element
    expect(sidebar).toBeTruthy()
    expect(sidebar.getAttribute("data-state")).toBe("expanded")

    fireEvent.keyDown(window, { key: "b", metaKey: true })
    expect(sidebar.getAttribute("data-state")).toBe("collapsed")

    fireEvent.keyDown(window, { key: "k", metaKey: true })
    expect((await screen.findAllByPlaceholderText(/search pages/i)).length).toBeGreaterThan(0)
    expect(sidebar.getAttribute("data-state")).toBe("collapsed")
  })
})

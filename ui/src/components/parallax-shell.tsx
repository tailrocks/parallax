import { Link, useRouterState } from "@tanstack/react-router"
import { useEffect, useState } from "react"

import { NavIcon } from "@/components/nav-icon"
import type { NavItem } from "@/components/nav"
import { primaryNav, workspaceNav } from "@/components/nav"
import { ThemeSwitcher } from "@/components/theme-switcher"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { graphql } from "@/lib/api"
import { cn } from "@/lib/utils"

function isActive(pathname: string, href: string) {
  if (href === "/") return pathname === "/"
  return pathname === href || pathname.startsWith(`${href}/`)
}

function BrandMark() {
  return (
    <Link
      to="/"
      aria-label="Parallax home"
      className="flex min-w-0 items-center gap-2 rounded-full px-1.5 text-sidebar-foreground transition-opacity hover:opacity-80"
    >
      <span className="flex shrink-0 items-center">
        <span className="size-4 rounded-full bg-foreground shadow-[0_0_12px_oklch(1_0_0_/_16%)]" />
        <span className="-ml-1.5 size-4 rounded-full bg-blue-500" />
        <span className="-ml-1.5 size-4 rounded-full bg-orange-500" />
      </span>
      <span className="truncate font-heading text-lg font-semibold tracking-tight group-data-[collapsible=icon]:hidden">
        Parallax
      </span>
    </Link>
  )
}

interface DashboardNavItem {
  id: string
  name: string
}

function NavGroup({
  items,
  dashboards = [],
}: {
  items: readonly NavItem[]
  dashboards?: DashboardNavItem[]
}) {
  const pathname = useRouterState({ select: (s) => s.location.pathname })

  return (
    <SidebarMenu>
      {items.map((item) => {
        const active = isActive(pathname, item.href)

        return (
          <SidebarMenuItem key={item.href}>
            <SidebarMenuButton
              render={<Link to={item.href} />}
              isActive={active}
              tooltip={item.label}
            >
              <NavIcon
                icon={item.icon}
                activeIcon={item.activeIcon}
                active={active}
                className={item.iconClassName}
              />
              <span>{item.label}</span>
            </SidebarMenuButton>
            {item.href === "/dashboards" && dashboards.length > 0 ? (
              <SidebarMenuSub>
                {dashboards.slice(0, 7).map((dashboard) => (
                  <SidebarMenuSubItem key={dashboard.id}>
                    <SidebarMenuSubButton
                      render={
                        <Link
                          to="/dashboards/$dashboardId"
                          params={{ dashboardId: dashboard.id }}
                        />
                      }
                      isActive={pathname === `/dashboards/${dashboard.id}`}
                    >
                      <span className="truncate">{dashboard.name}</span>
                    </SidebarMenuSubButton>
                  </SidebarMenuSubItem>
                ))}
                {dashboards.length > 7 ? (
                  <SidebarMenuSubItem>
                    <SidebarMenuSubButton render={<Link to="/dashboards" />}>
                      All dashboards
                    </SidebarMenuSubButton>
                  </SidebarMenuSubItem>
                ) : null}
              </SidebarMenuSub>
            ) : null}
          </SidebarMenuItem>
        )
      })}
    </SidebarMenu>
  )
}

function StatusPill() {
  const [online, setOnline] = useState<boolean | null>(null)

  useEffect(() => {
    const controller = new AbortController()

    fetch("/graphql", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ query: "{ health }" }),
      signal: controller.signal,
    })
      .then((response) => response.json())
      .then((body: { data?: { health?: unknown } }) => {
        setOnline(body.data?.health === "ok" || body.data?.health === true)
      })
      .catch(() => setOnline(false))

    return () => controller.abort()
  }, [])

  const healthy = online !== false

  return (
    <div className="flex h-8 items-center gap-2 rounded-full bg-background px-3 text-xs text-muted-foreground shadow-(--custom-shadow) group-data-[collapsible=icon]:hidden">
      <span
        className={cn(
          "size-1.5 rounded-full",
          healthy ? "bg-green-500" : "bg-rose-500"
        )}
      />
      <span className="font-medium text-foreground">
        {healthy ? "Local" : "Offline"}
      </span>
      <span className="font-mono">127.0.0.1:4000</span>
    </div>
  )
}

export function ParallaxShell({ children }: { children: React.ReactNode }) {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const [dashboards, setDashboards] = useState<DashboardNavItem[]>([])

  useEffect(() => {
    if (!pathname.startsWith("/dashboards")) return
    const controller = new AbortController()
    void graphql<{ dashboards: DashboardNavItem[] }>(`
      {
        dashboards {
          id
          name
        }
      }
    `)
      .then((data) => setDashboards(data.dashboards))
      .catch(() => {})
    return () => controller.abort()
  }, [pathname])

  return (
    <SidebarProvider className="relative h-svh min-h-0 overflow-hidden">
      <Sidebar variant="inset" collapsible="icon">
        <SidebarHeader className="px-3 py-4">
          <div className="flex h-9 items-center gap-2">
            <div className="min-w-0 flex-1">
              <BrandMark />
            </div>
            <SidebarTrigger className="shrink-0" />
          </div>
        </SidebarHeader>

        <SidebarContent className="px-2">
          <SidebarGroup>
            <SidebarGroupContent>
              <NavGroup items={primaryNav} />
            </SidebarGroupContent>
          </SidebarGroup>

          <SidebarGroup>
            <SidebarGroupLabel>Workspace</SidebarGroupLabel>
            <SidebarGroupContent>
              <NavGroup items={workspaceNav} dashboards={dashboards} />
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>

        <SidebarFooter className="px-3 py-4">
          <StatusPill />
          <div className="group-data-[collapsible=icon]:hidden">
            <ThemeSwitcher />
          </div>
        </SidebarFooter>
      </Sidebar>

      <SidebarInset className="min-h-0 overflow-hidden">
        <main className="min-h-0 flex-1 overflow-y-auto">
          <div className="mx-auto flex max-w-380 flex-col gap-6 p-10 2xl:p-16">
            {children}
          </div>
        </main>
      </SidebarInset>
    </SidebarProvider>
  )
}

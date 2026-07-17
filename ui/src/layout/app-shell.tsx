import { Link, useRouterState } from "@tanstack/react-router"
import { IconSearch } from "@tabler/icons-react"
import { useEffect, useState } from "react"

import { CommandPalette } from "@/layout/command-palette"
import { NavIcon } from "@/layout/nav-icon"
import { primaryNav, workspaceNav } from "@/shared/navigation"
import type { NavItem } from "@/shared/navigation"
import { ThemeSwitcher } from "@/layout/theme-switcher"
import { Button } from "@/components/ui/button"
import { Kbd } from "@/components/ui/kbd"
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
import { loadAppStatus } from "@/features/app-status"
import { loadDashboardNavigation, type DashboardNavigationItem } from "@/features/dashboards"
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

function NavGroup({
  items,
  dashboards = [],
}: {
  items: readonly NavItem[]
  dashboards?: DashboardNavigationItem[]
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
  const [endpoint, setEndpoint] = useState("127.0.0.1:4000")

  useEffect(() => {
    const controller = new AbortController()
    void loadAppStatus(controller.signal).then((status) => {
      setOnline(status.healthy)
      setEndpoint(status.endpointLabel)
    })
    return () => controller.abort()
  }, [])

  const healthy = online !== false

  return (
    <div className="flex h-8 items-center gap-2 rounded-full bg-background px-3 text-xs text-muted-foreground shadow-(--custom-shadow) group-data-[collapsible=icon]:hidden">
      <span className={cn("size-1.5 rounded-full", healthy ? "bg-green-500" : "bg-rose-500")} />
      <span className="font-medium text-foreground">{healthy ? "Local" : "Offline"}</span>
      <span className="font-mono">{endpoint}</span>
    </div>
  )
}

export function ParallaxShell({ children }: { children: React.ReactNode }) {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const [dashboards, setDashboards] = useState<DashboardNavigationItem[]>([])
  const [dashboardNavError, setDashboardNavError] = useState<string | null>(null)
  const [paletteOpen, setPaletteOpen] = useState(false)

  useEffect(() => {
    if (!pathname.startsWith("/dashboards")) {
      setDashboardNavError(null)
      return
    }
    const controller = new AbortController()
    setDashboardNavError(null)
    void loadDashboardNavigation({ signal: controller.signal })
      .then((items) => setDashboards(items))
      .catch((error: unknown) => {
        if (error instanceof DOMException && error.name === "AbortError") {
          return
        }
        setDashboardNavError(error instanceof Error ? error.message : String(error))
      })
    return () => controller.abort()
  }, [pathname])

  return (
    <SidebarProvider className="relative h-svh min-h-0 overflow-hidden">
      <CommandPalette open={paletteOpen} onOpenChange={setPaletteOpen} />
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
              {dashboardNavError ? (
                <p className="px-2 pt-2 text-xs text-destructive group-data-[collapsible=icon]:hidden">
                  {dashboardNavError}
                </p>
              ) : null}
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>

        <SidebarFooter className="px-3 py-4">
          <div className="group-data-[collapsible=icon]:hidden">
            <Button
              type="button"
              variant="outline"
              size="sm"
              className="w-full justify-between"
              onClick={() => setPaletteOpen(true)}
            >
              <span className="flex min-w-0 items-center gap-1.5">
                <IconSearch className="size-3.5" />
                <span>Search...</span>
              </span>
              <Kbd>⌘K</Kbd>
            </Button>
          </div>
          <StatusPill />
          <div className="group-data-[collapsible=icon]:hidden">
            <ThemeSwitcher />
          </div>
        </SidebarFooter>
      </Sidebar>

      <SidebarInset className="min-h-0 overflow-hidden">
        <main className="min-h-0 flex-1 overflow-y-auto">
          <div className="mx-auto flex max-w-380 flex-col gap-6 p-10 2xl:p-16">{children}</div>
        </main>
      </SidebarInset>
    </SidebarProvider>
  )
}

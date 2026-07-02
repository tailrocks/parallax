import { Link, useRouterState } from "@tanstack/react-router"
import {
  BugIcon,
  ChartNoAxesCombinedIcon,
  DatabaseIcon,
  GitBranchIcon,
  NetworkIcon,
  ScrollTextIcon,
  TerminalSquareIcon,
} from "lucide-react"
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { Separator } from "@/components/ui/separator"

const NAV = [
  {
    to: "/issues",
    label: "Issues",
    icon: BugIcon,
    iconClass: "bg-[color-mix(in_srgb,var(--brand-rose),transparent_12%)]",
  },
  {
    to: "/traces",
    label: "Traces",
    icon: GitBranchIcon,
    iconClass: "bg-[color-mix(in_srgb,var(--brand-blue),transparent_10%)]",
  },
  {
    to: "/logs",
    label: "Logs",
    icon: ScrollTextIcon,
    iconClass: "bg-[color-mix(in_srgb,var(--brand-orange),transparent_14%)]",
  },
  {
    to: "/services",
    label: "Services",
    icon: NetworkIcon,
    iconClass: "bg-[color-mix(in_srgb,var(--brand-green),transparent_18%)]",
  },
  {
    to: "/runs",
    label: "Runs",
    icon: TerminalSquareIcon,
    iconClass: "bg-[color-mix(in_srgb,var(--brand-violet),transparent_14%)]",
  },
  {
    to: "/dashboards",
    label: "Dashboards",
    icon: ChartNoAxesCombinedIcon,
    iconClass: "bg-[color-mix(in_srgb,var(--brand-fuchsia),transparent_18%)]",
  },
  {
    to: "/sql",
    label: "SQL",
    icon: DatabaseIcon,
    iconClass: "bg-[color-mix(in_srgb,var(--brand-blue),transparent_20%)]",
  },
] as const

export function ParallaxShell({ children }: { children: React.ReactNode }) {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  return (
    <SidebarProvider>
      <Sidebar
        collapsible="icon"
        className="border-sidebar-border/80 bg-sidebar/95"
      >
        <SidebarHeader className="px-3 py-4">
          <Link
            to="/issues"
            aria-label="Parallax home"
            className="flex h-9 items-center gap-2 rounded-full px-1.5 text-sidebar-foreground transition-opacity hover:opacity-80"
          >
            <span className="flex items-center">
              <span className="size-4 rounded-full bg-foreground shadow-[0_0_12px_oklch(1_0_0_/_16%)]" />
              <span className="-ml-1.5 size-4 rounded-full bg-(--brand-blue)" />
              <span className="-ml-1.5 size-4 rounded-full bg-(--brand-orange)" />
            </span>
            <span className="font-heading text-lg font-semibold tracking-tight group-data-[collapsible=icon]:hidden">
              Parallax
            </span>
          </Link>
        </SidebarHeader>
        <SidebarContent className="px-2">
          <SidebarGroup>
            <SidebarGroupContent>
              <SidebarMenu className="gap-1.5">
                {NAV.map((item) => {
                  const Icon = item.icon
                  return (
                    <SidebarMenuItem key={item.to}>
                      <SidebarMenuButton
                        render={<Link to={item.to} />}
                        isActive={pathname.startsWith(item.to)}
                        tooltip={item.label}
                        className="h-10 rounded-xl text-sidebar-foreground/72 hover:bg-sidebar-accent/70 data-active:bg-sidebar-accent/90 data-active:text-sidebar-accent-foreground data-active:shadow-[inset_0_0_0_1px_var(--sidebar-border)]"
                      >
                        <span
                          className={`grid size-6 shrink-0 place-items-center rounded-lg text-white shadow-sm ${item.iconClass}`}
                        >
                          <Icon />
                        </span>
                        {item.label}
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  )
                })}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>
      </Sidebar>
      <SidebarInset className="bg-background">
        <header className="sticky top-0 z-40 flex h-16 shrink-0 items-center gap-3 border-b border-border/50 bg-background/70 px-4 backdrop-blur-sm sm:px-6">
          <SidebarTrigger className="-ml-1 rounded-full" />
          <Separator orientation="vertical" className="h-4 bg-border/70" />
          <span className="parallax-pill inline-flex h-8 items-center gap-2 px-3 text-xs text-muted-foreground">
            <span className="size-1.5 rounded-full bg-(--brand-green)" />
            <span className="font-medium text-foreground">Local</span>
            <span className="font-mono">127.0.0.1:4000</span>
          </span>
        </header>
        <main className="flex-1 overflow-auto p-4 sm:p-6">{children}</main>
      </SidebarInset>
    </SidebarProvider>
  )
}

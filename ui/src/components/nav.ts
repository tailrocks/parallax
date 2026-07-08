import type { Icon } from "@tabler/icons-react"
import {
  IconAffiliate,
  IconAffiliateFilled,
  IconArticle,
  IconArticleFilled,
  IconBug,
  IconBugFilled,
  IconDatabase,
  IconDatabaseFilled,
  IconHome,
  IconHomeFilled,
  IconLayoutDashboard,
  IconLayoutDashboardFilled,
  IconServer,
  IconSitemap,
  IconSitemapFilled,
  IconTerminal2,
} from "@tabler/icons-react"

export type NavItem = {
  href: string
  label: string
  icon: Icon
  activeIcon: Icon
  iconClassName?: string
}

export const primaryNav: NavItem[] = [
  {
    href: "/",
    label: "Overview",
    icon: IconHome,
    activeIcon: IconHomeFilled,
    iconClassName:
      "bg-sky-100 dark:bg-sky-950 rounded-xl p-0.5 corner-squircle text-sky-500 shadow-[inset_0_0_0_1px_rgba(14,165,233,0.14),0_2px_6px_-2px_rgba(14,165,233,0.25)] dark:shadow-(--custom-shadow)",
  },
  {
    href: "/issues",
    label: "Issues",
    icon: IconBug,
    activeIcon: IconBugFilled,
    iconClassName:
      "bg-rose-100 dark:bg-rose-950 rounded-xl p-0.5 corner-squircle text-rose-500 shadow-[inset_0_0_0_1px_rgba(244,63,94,0.14),0_2px_6px_-2px_rgba(244,63,94,0.25)] dark:shadow-(--custom-shadow)",
  },
  {
    href: "/traces",
    label: "Traces",
    icon: IconAffiliate,
    activeIcon: IconAffiliateFilled,
    iconClassName:
      "bg-[#ede0d4] dark:bg-[#2e211b] rounded-xl p-0.5 corner-squircle text-[#8b5e34] dark:text-[#c9a888] shadow-[inset_0_0_0_1px_rgba(139,94,52,0.14),0_2px_6px_-2px_rgba(139,94,52,0.25)] dark:shadow-(--custom-shadow)",
  },
  {
    href: "/ecosystem",
    label: "Ecosystem",
    icon: IconSitemap,
    activeIcon: IconSitemapFilled,
    iconClassName:
      "bg-cyan-100 dark:bg-cyan-950 rounded-xl p-0.5 corner-squircle text-cyan-500 shadow-[inset_0_0_0_1px_rgba(6,182,212,0.14),0_2px_6px_-2px_rgba(6,182,212,0.25)] dark:shadow-(--custom-shadow)",
  },
  {
    href: "/logs",
    label: "Logs",
    icon: IconArticle,
    activeIcon: IconArticleFilled,
    iconClassName:
      "bg-orange-100 dark:bg-orange-950 rounded-xl p-0.5 corner-squircle text-orange-500 shadow-[inset_0_0_0_1px_rgba(249,115,22,0.14),0_2px_6px_-2px_rgba(249,115,22,0.25)] dark:shadow-(--custom-shadow)",
  },
  {
    href: "/services",
    label: "Services",
    icon: IconServer,
    activeIcon: IconServer,
    iconClassName:
      "bg-emerald-100 dark:bg-emerald-950 rounded-xl p-0.5 corner-squircle text-emerald-500 shadow-[inset_0_0_0_1px_rgba(16,185,129,0.14),0_2px_6px_-2px_rgba(16,185,129,0.25)] dark:shadow-(--custom-shadow)",
  },
]

export const workspaceNav: NavItem[] = [
  {
    href: "/runs",
    label: "Runs",
    icon: IconTerminal2,
    activeIcon: IconTerminal2,
    iconClassName:
      "bg-violet-100 dark:bg-violet-950 rounded-xl p-0.5 corner-squircle text-violet-500 shadow-[inset_0_0_0_1px_rgba(139,92,246,0.14),0_2px_6px_-2px_rgba(139,92,246,0.25)] dark:shadow-(--custom-shadow)",
  },
  {
    href: "/dashboards",
    label: "Dashboards",
    icon: IconLayoutDashboard,
    activeIcon: IconLayoutDashboardFilled,
    iconClassName:
      "bg-fuchsia-100 dark:bg-fuchsia-950 rounded-xl p-0.5 corner-squircle text-fuchsia-500 shadow-[inset_0_0_0_1px_rgba(217,70,239,0.14),0_2px_6px_-2px_rgba(217,70,239,0.25)] dark:shadow-(--custom-shadow)",
  },
  {
    href: "/sql",
    label: "SQL",
    icon: IconDatabase,
    activeIcon: IconDatabaseFilled,
    iconClassName:
      "bg-yellow-100 dark:bg-yellow-950 rounded-xl p-0.5 corner-squircle text-yellow-500 shadow-[inset_0_0_0_1px_rgba(234,179,8,0.14),0_2px_6px_-2px_rgba(234,179,8,0.25)] dark:shadow-(--custom-shadow)",
  },
]

export const nav = [...primaryNav, ...workspaceNav] as const

export function navItem(href: string): NavItem | undefined {
  return nav.find((item) => item.href === href)
}

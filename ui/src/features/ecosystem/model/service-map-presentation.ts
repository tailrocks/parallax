import {
  IconArrowsRightLeft,
  IconDatabase,
  IconPlugConnected,
  IconServer,
  IconTerminal2,
  IconWorld,
} from "@tabler/icons-react"

export const SERVICE_MAP_NODE_PRESENTATION = {
  browser: { label: "browser", Icon: IconWorld, color: "text-sky-500" },
  cli: { label: "cli", Icon: IconTerminal2, color: "text-violet-500" },
  database: { label: "database", Icon: IconDatabase, color: "text-amber-600" },
  external: { label: "external", Icon: IconPlugConnected, color: "text-emerald-600" },
  queue: { label: "queue", Icon: IconArrowsRightLeft, color: "text-cyan-600" },
  service: { label: "service", Icon: IconServer, color: "text-muted-foreground" },
} as const

export function nodePresentation(kind: string) {
  return (
    SERVICE_MAP_NODE_PRESENTATION[kind as keyof typeof SERVICE_MAP_NODE_PRESENTATION] ?? {
      label: kind,
      Icon: IconServer,
      color: "text-muted-foreground",
    }
  )
}

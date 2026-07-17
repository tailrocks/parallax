import type { ReactNode } from "react"
import { IconBroadcast } from "@tabler/icons-react"

import { Chip } from "@/shared/console/chip"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export function LiveStreamPanel({
  title,
  description,
  count,
  endpoint,
  active,
  children,
}: {
  title: string
  description: string
  count: number
  endpoint: string
  active: boolean
  children?: ReactNode
}) {
  return (
    <Card>
      <CardHeader className="flex-row items-start justify-between gap-4">
        <div className="flex min-w-0 items-start gap-3">
          <span className="grid size-9 shrink-0 place-items-center rounded-lg bg-muted">
            <IconBroadcast />
          </span>
          <div className="min-w-0">
            <CardTitle className="truncate text-sm">{title}</CardTitle>
            <p className="mt-1 text-sm text-muted-foreground">{description}</p>
          </div>
        </div>
        <div className="flex flex-wrap justify-end gap-2">
          <Badge variant={active ? "emerald" : "amber"}>
            {active ? (
              <span className="size-1.5 animate-pulse rounded-full bg-current" />
            ) : null}
            {active ? "connected" : "reconnecting…"}
          </Badge>
          <Badge variant="outline">{count.toLocaleString()} shown</Badge>
          <Chip className="text-muted-foreground">{endpoint}</Chip>
        </div>
      </CardHeader>
      {children ? <CardContent>{children}</CardContent> : null}
    </Card>
  )
}

export function LiveEventStack({
  items,
}: {
  items: Array<{
    id: string
    title: string
    meta: string
    status?: "ok" | "error"
    detail?: string
  }>
}) {
  if (items.length === 0) {
    return (
      <div className="rounded-lg border border-dashed bg-muted/20 p-6 text-sm text-muted-foreground">
        Waiting for streamed events. New telemetry appears here as soon as it
        crosses the local SSE broker.
      </div>
    )
  }

  return (
    <div className="grid gap-2">
      {items.slice(0, 6).map((item, index) => (
        <div
          key={item.id}
          className="flex items-start gap-3 rounded-lg border bg-muted/20 p-3"
        >
          <span
            className={
              item.status === "error"
                ? "mt-1 size-2 rounded-full bg-rose-500"
                : "mt-1 size-2 rounded-full bg-sky-500"
            }
          />
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <span className="font-mono text-[10px] text-muted-foreground">
                #{String(index + 1).padStart(2, "0")}
              </span>
              <p className="truncate text-sm font-medium">{item.title}</p>
            </div>
            <p className="mt-1 truncate text-xs text-muted-foreground">
              {item.meta}
            </p>
            {item.detail ? (
              <p className="mt-2 line-clamp-2 text-xs text-muted-foreground/80">
                {item.detail}
              </p>
            ) : null}
          </div>
        </div>
      ))}
    </div>
  )
}

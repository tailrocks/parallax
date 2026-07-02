import type { ReactNode } from "react"
import { RadioIcon } from "lucide-react"

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
    <section className="parallax-panel parallax-glow-border overflow-hidden">
      <div className="flex flex-wrap items-center justify-between gap-4 border-b border-border/70 p-4">
        <div className="flex min-w-0 items-center gap-3">
          <span className="relative grid size-10 shrink-0 place-items-center rounded-2xl bg-[color-mix(in_srgb,var(--brand-green),transparent_16%)] text-white shadow-[0_0_26px_color-mix(in_srgb,var(--brand-green),transparent_68%)]">
            <RadioIcon />
            {active ? (
              <span className="absolute -top-0.5 -right-0.5 size-3 rounded-full bg-(--brand-green) ring-4 ring-background" />
            ) : null}
          </span>
          <div className="min-w-0">
            <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
              Live stream
            </p>
            <h2 className="truncate text-lg font-semibold tracking-tight">
              {title}
            </h2>
            <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
              {description}
            </p>
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <span className="parallax-pill inline-flex h-8 items-center gap-2 px-3 text-xs text-muted-foreground">
            <span className="size-1.5 rounded-full bg-(--brand-green)" />
            {active ? "connected" : "idle"}
          </span>
          <span className="parallax-pill h-8 px-3 py-1.5 font-mono text-xs text-muted-foreground">
            {endpoint}
          </span>
          <span className="parallax-pill h-8 px-3 py-1.5 text-xs text-muted-foreground">
            {count.toLocaleString()} shown
          </span>
        </div>
      </div>
      {children ? <div className="p-4">{children}</div> : null}
    </section>
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
      <div className="rounded-2xl border border-dashed border-border/80 bg-background/60 p-6 text-sm text-muted-foreground">
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
          className="flex items-start gap-3 rounded-2xl border border-border/70 bg-background/60 p-3"
        >
          <span
            className={
              item.status === "error"
                ? "mt-1 size-2 rounded-full bg-(--brand-rose)"
                : "mt-1 size-2 rounded-full bg-(--brand-blue)"
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

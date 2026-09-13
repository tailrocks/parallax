import { cn } from "@/lib/utils"

/** One query-toolbar composition for every list page (workstream D).
 *
 * Order contract (`DESIGN.md` §8.1): primary row (free-text search, then
 * structured filter) → secondary row (faceted selects, toggles, view
 * actions) → result count pinned right. Max two rows at 1280px.
 * Presentational only: pages keep owning state and URL search params. */

export function QueryBar({
  children,
  className,
}: {
  children: React.ReactNode
  className?: string
}) {
  return <div className={cn("flex flex-col gap-2", className)}>{children}</div>
}

export function QueryBarRow({
  children,
  className,
}: {
  children: React.ReactNode
  className?: string
}) {
  return <div className={cn("flex flex-wrap items-center gap-2", className)}>{children}</div>
}

/** Right-pinned result count (`42 of 1,024`). Renders nothing when total is
 * unknown so pages without counts keep the same row geometry. */
export function QueryBarCount({
  shown,
  total,
  unit = "rows",
  className,
}: {
  shown: number
  total?: number
  unit?: string
  className?: string
}) {
  return (
    <span className={cn("ml-auto text-xs text-muted-foreground tabular-nums", className)}>
      {shown.toLocaleString()}
      {total !== undefined ? ` of ${total.toLocaleString()}` : ""} {unit}
    </span>
  )
}

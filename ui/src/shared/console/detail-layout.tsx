import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"

/** Shared detail-page idiom (workstream D).
 *
 * Detail pages converge on: `PageHeader` (with `back`) → `DetailSummary`
 * (answer above the fold) → tabs → `SectionCard`s. Kills the 7-card scroll
 * where occurrences sit below metrics/tags/agent-handoff. */

export interface SummaryItem {
  label: string
  /** Already-formatted value node (count, time, badge, link). */
  value: React.ReactNode
  hint?: React.ReactNode
}

/** Above-the-fold answer strip. `<dl>` grid: label over value, tabular
 * numerals, wraps to 2–6 columns by item count. */
export function DetailSummary({
  items,
  loading = false,
  className,
}: {
  items: SummaryItem[]
  loading?: boolean
  className?: string
}) {
  return (
    <dl
      className={cn(
        "grid grid-cols-2 gap-x-6 gap-y-3 rounded-xl border bg-card px-4 py-3 sm:grid-cols-3 lg:grid-cols-6",
        className
      )}
    >
      {items.map((item) => (
        <div key={item.label} className="min-w-0 space-y-0.5">
          <dt className="text-xs text-muted-foreground">{item.label}</dt>
          <dd className="truncate text-sm font-medium tabular-nums">
            {loading ? (
              <span aria-hidden className="inline-block h-4 w-16 rounded bg-muted align-middle" />
            ) : (
              item.value
            )}
          </dd>
          {item.hint && !loading ? (
            <div className="truncate text-xs text-muted-foreground">{item.hint}</div>
          ) : null}
        </div>
      ))}
    </dl>
  )
}

/** One deep-linkable detail section. `id` enables `#` anchors;
 * `scroll-mt` keeps the title clear of sticky chrome. */
export function SectionCard({
  id,
  title,
  action,
  children,
  className,
}: {
  id: string
  title: string
  action?: React.ReactNode
  children: React.ReactNode
  className?: string
}) {
  return (
    <Card id={id} className={cn("scroll-mt-4", className)}>
      <CardHeader className="flex-row items-center justify-between gap-2">
        <CardTitle className="text-sm">{title}</CardTitle>
        {action}
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  )
}

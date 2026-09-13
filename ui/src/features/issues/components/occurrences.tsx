import { useMemo, type RefObject } from "react"
import { Link } from "@tanstack/react-router"
import { IconArrowUpRight } from "@tabler/icons-react"

import { CopyButton } from "@/shared/console/copy-button"
import { HeatCell, buildHeatScale } from "@/shared/console/heat-cell"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { IssueEvent } from "@/features/issues/model/issue-detail"
import { rangeLinkSearch, type ResolvedRange } from "@/domain/range"
import { formatTimeInRange } from "@/shared/format"
import { cn } from "@/lib/utils"

export function Occurrences({
  refEl,
  events,
  selectedEvent,
  onSelect,
  bucket,
  range,
}: {
  refEl: RefObject<HTMLDivElement | null>
  events: readonly IssueEvent[]
  selectedEvent: IssueEvent | null
  onSelect: (event: IssueEvent) => void
  bucket: string | null
  range: ResolvedRange
}) {
  const durations = events.map((event) => Number(event.tsNanos))
  const scale = useMemo(() => buildHeatScale(durations), [durations])
  return (
    <Card ref={refEl}>
      <CardHeader>
        <CardTitle className="text-sm">
          Occurrences
          {bucket ? (
            <span className="ml-2 font-normal text-muted-foreground">
              selected hour ({events.length})
            </span>
          ) : null}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {events.length === 0 ? (
          <p className="text-sm text-muted-foreground">No occurrences in this window.</p>
        ) : (
          <ul className="space-y-2 text-sm">
            {events.map((event) => {
              const selected =
                selectedEvent?.tsNanos === event.tsNanos && selectedEvent?.spanId === event.spanId
              return (
                <li
                  key={`${event.tsNanos}-${event.spanId}`}
                  className="grid gap-2 rounded-lg border bg-muted/20 px-2 py-1.5 md:grid-cols-[minmax(0,1fr)_auto]"
                >
                  <button
                    type="button"
                    aria-current={selected ? "true" : undefined}
                    onClick={() => onSelect(event)}
                    className={cn(
                      "flex min-w-0 flex-col gap-1 rounded-md px-2 py-1 text-left transition outline-none hover:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring",
                      selected && "bg-primary/5 ring-1 ring-primary/30"
                    )}
                  >
                    <span className="min-w-0 truncate font-medium">{event.message}</span>
                    <span className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground">
                      <HeatCell value={Number(event.tsNanos)} scale={scale}>
                        {formatTimeInRange(event.tsNanos, range)}
                      </HeatCell>
                      <Badge variant="outline">{event.service}</Badge>
                      {event.environment ? (
                        <Badge variant="secondary">{event.environment}</Badge>
                      ) : null}
                    </span>
                  </button>
                  <span className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground">
                    <CopyButton value={event.message} />
                    {event.traceId ? (
                      <Link
                        to="/traces/$traceId"
                        params={{ traceId: event.traceId }}
                        search={rangeLinkSearch(range)}
                        className="inline-flex items-center gap-1 hover:text-foreground"
                      >
                        trace
                        <IconArrowUpRight className="size-3" />
                      </Link>
                    ) : null}
                  </span>
                </li>
              )
            })}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}

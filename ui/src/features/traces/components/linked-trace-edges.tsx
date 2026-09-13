import { Link } from "@tanstack/react-router"
import { IconExternalLink } from "@tabler/icons-react"

import { Badge } from "@/components/ui/badge"
import { formatDurationNs } from "@/shared/format"
import type { SpanLink, TraceSummary } from "@/features/traces/model/wire"
import type { rangeLinkSearch } from "@/domain/time-range/range"

type TraceRangeSearch = ReturnType<typeof rangeLinkSearch>

export function LinkedTraceEdges({
  links,
  linkedTraceById,
  rangeSearch,
}: {
  links: SpanLink[]
  linkedTraceById: ReadonlyMap<string, TraceSummary>
  rangeSearch?: TraceRangeSearch
}) {
  return (
    <ul className="space-y-2">
      {links.map((link) => {
        const target = linkedTraceById.get(link.traceId)
        return (
          <li key={`${link.traceId}-${link.spanId}`} data-testid="trace-link-edge">
            <Link
              to="/traces/$traceId"
              params={{ traceId: link.traceId }}
              {...(rangeSearch ? { search: rangeSearch } : {})}
              className="block rounded-lg border border-border/70 bg-background/60 p-2 hover:bg-muted/60"
            >
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-1.5">
                    <Badge variant="outline">{target?.service ?? "unknown service"}</Badge>
                    {target?.hasError ? <Badge variant="rose">error</Badge> : null}
                  </div>
                  <p className="mt-1 font-medium break-words">
                    {target?.rootName ?? "Unresolved linked trace"}
                  </p>
                </div>
                <IconExternalLink className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
              </div>
              <div className="mt-2 flex flex-wrap gap-2 font-mono text-[11px] text-muted-foreground">
                <span>{link.traceId}</span>
                {target ? (
                  <>
                    <span>{target.spanCount.toLocaleString()} spans</span>
                    <span>{formatDurationNs(target.durationNs)}</span>
                  </>
                ) : null}
              </div>
            </Link>
            <dl className="mt-1 grid gap-1 pl-2 font-mono text-[11px] text-muted-foreground">
              <div className="flex gap-2">
                <dt>span</dt>
                <dd>{link.spanId || "-"}</dd>
              </div>
              {link.attributes && link.attributes !== "{}" ? (
                <div className="flex gap-2">
                  <dt>attributes</dt>
                  <dd className="break-all">{link.attributes}</dd>
                </div>
              ) : null}
            </dl>
          </li>
        )
      })}
    </ul>
  )
}

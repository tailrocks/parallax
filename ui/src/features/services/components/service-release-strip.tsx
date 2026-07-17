import { useMemo } from "react"

import { Badge } from "@/components/ui/badge"
import type { ReleaseWindow } from "@/features/services/model/service-detail"
import { formatCount, formatDateTime } from "@/shared/format"
import type { ResolvedRange } from "@/domain/time-range/range"

export function ServiceReleaseStrip({
  releases,
  range,
}: {
  releases: readonly ReleaseWindow[]
  range: ResolvedRange
}) {
  const segments = useMemo(() => {
    const from = BigInt(range.fromNanos)
    const to = BigInt(range.toNanos)
    const total = to - from
    if (total <= 0n) return []
    return releases.map((release) => {
      const first = BigInt(release.firstSeenNanos)
      const last = BigInt(release.lastSeenNanos)
      const start = first < from ? from : first > to ? to : first
      const end = last < from ? from : last > to ? to : last
      const left = Number(((start - from) * 10_000n) / total) / 100
      const duration = end > start ? end - start : 1n
      const rawWidth = Number((duration * 10_000n) / total) / 100
      const width = Math.max(4, Math.min(100 - left, rawWidth))
      return {
        ...release,
        left,
        width,
        title: `${release.version}: ${formatDateTime(release.firstSeenNanos)} - ${formatDateTime(release.lastSeenNanos)} (${formatCount(Number(release.spanCount))} spans)`,
      }
    })
  }, [range.fromNanos, range.toNanos, releases])

  if (segments.length === 0) return null

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-sm font-medium">Releases</h2>
        <Badge variant="secondary">{segments.length} versions</Badge>
      </div>
      <div className="relative h-9 overflow-hidden rounded-md border bg-muted/30">
        {segments.map((segment) => (
          <div
            key={`${segment.version}-${segment.firstSeenNanos}`}
            className="absolute inset-y-1 flex min-w-12 items-center justify-center truncate rounded-sm border border-primary/30 bg-primary/15 px-2 text-xs font-medium text-primary"
            style={{
              left: `${segment.left}%`,
              width: `${segment.width}%`,
            }}
            title={segment.title}
          >
            {segment.version}
          </div>
        ))}
      </div>
    </div>
  )
}

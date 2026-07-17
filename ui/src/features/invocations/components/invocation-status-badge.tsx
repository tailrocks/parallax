import { Badge } from "@/components/ui/badge"
import type { InvocationStatus } from "@/lib/invocation"

export function InvocationStatusBadge({
  status,
  exitCode,
}: {
  status: InvocationStatus
  exitCode: number | null
}) {
  if (status === "running") {
    return (
      <Badge variant="blue">
        <span className="size-1.5 animate-pulse rounded-full bg-current" />
        running
      </Badge>
    )
  }
  if (status === "stale") return <Badge variant="secondary">stale</Badge>
  if (status === "failed") {
    return (
      <Badge variant="rose">
        {exitCode != null && exitCode !== 0 ? `exit ${exitCode}` : "failed"}
      </Badge>
    )
  }
  return <Badge variant="emerald">finished</Badge>
}

export function OutcomeChip({ outcome }: { outcome: string | null }) {
  if (!outcome) return null
  const variant =
    outcome === "success"
      ? ("emerald" as const)
      : outcome === "skip" || outcome === "cancellation"
        ? ("secondary" as const)
        : ("rose" as const)
  return <Badge variant={variant}>{outcome}</Badge>
}

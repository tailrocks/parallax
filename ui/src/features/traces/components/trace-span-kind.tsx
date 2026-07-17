import type { Icon } from "@tabler/icons-react"
import {
  IconArrowBigDownLine,
  IconArrowBigUpLine,
  IconArrowUpRight,
  IconCpu,
  IconServer,
} from "@tabler/icons-react"

import { Badge } from "@/components/ui/badge"

export type SpanKind =
  | "SERVER"
  | "CLIENT"
  | "INTERNAL"
  | "PRODUCER"
  | "CONSUMER"
  | string

type SpanKindMeta = {
  variant: "blue" | "violet" | "amber" | "emerald" | "secondary" | "rose"
  icon: Icon
  bar: string
}

const kindMap: Record<string, SpanKindMeta> = {
  SERVER: {
    variant: "blue",
    icon: IconServer,
    bar: "bg-sky-500 dark:bg-sky-400",
  },
  CLIENT: {
    variant: "blue",
    icon: IconArrowUpRight,
    bar: "bg-blue-500 dark:bg-blue-400",
  },
  INTERNAL: {
    variant: "violet",
    icon: IconCpu,
    bar: "bg-violet-500 dark:bg-violet-400",
  },
  PRODUCER: {
    variant: "amber",
    icon: IconArrowBigUpLine,
    bar: "bg-amber-500 dark:bg-amber-400",
  },
  CONSUMER: {
    variant: "emerald",
    icon: IconArrowBigDownLine,
    bar: "bg-emerald-500 dark:bg-emerald-400",
  },
}

/// The wire encodes span kind as `SPAN_KIND_INTERNAL`; every display and
/// color decision works on the bare kind name.
export function spanKindLabel(kind: SpanKind): string {
  return kind.replace(/^SPAN_KIND_/, "")
}

export function spanKindMeta(kind: SpanKind, statusCode?: string) {
  if (statusCode === "STATUS_CODE_ERROR") {
    return {
      variant: "rose" as const,
      icon: IconServer,
      bar: "bg-rose-500 dark:bg-rose-400",
    }
  }
  return (
    kindMap[spanKindLabel(kind)] ?? {
      variant: "secondary" as const,
      icon: IconCpu,
      bar: "bg-muted-foreground",
    }
  )
}

export function SpanKindChip({
  kind,
  statusCode,
  compact = false,
}: {
  kind: SpanKind
  statusCode?: string
  /** Icon-only rendering for dense rows (full kind stays in the tooltip). */
  compact?: boolean
}) {
  const meta = spanKindMeta(kind, statusCode)
  const Icon = meta.icon
  const label = spanKindLabel(kind)
  return (
    <Badge variant={meta.variant} title={label}>
      <Icon />
      {compact ? null : label}
    </Badge>
  )
}

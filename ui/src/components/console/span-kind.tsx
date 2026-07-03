import type { Icon } from "@tabler/icons-react"
import {
  IconArrowBigDownLine,
  IconArrowBigUpLine,
  IconArrowUpRight,
  IconCpu,
  IconServer,
} from "@tabler/icons-react"

import { Badge } from "@/components/ui/badge"

export type SpanKind = "SERVER" | "CLIENT" | "INTERNAL" | "PRODUCER" | "CONSUMER" | string

const kindMap: Record<string, { variant: "blue" | "violet" | "amber" | "emerald" | "secondary"; icon: Icon; bar: string; chip: string }> = {
  SERVER: { variant: "blue", icon: IconServer, bar: "bg-sky-500", chip: "text-sky-600" },
  CLIENT: { variant: "blue", icon: IconArrowUpRight, bar: "bg-blue-500", chip: "text-blue-600" },
  INTERNAL: { variant: "violet", icon: IconCpu, bar: "bg-violet-500", chip: "text-violet-600" },
  PRODUCER: { variant: "amber", icon: IconArrowBigUpLine, bar: "bg-amber-500", chip: "text-amber-600" },
  CONSUMER: { variant: "emerald", icon: IconArrowBigDownLine, bar: "bg-emerald-500", chip: "text-emerald-600" },
}

export function spanKindMeta(kind: SpanKind, statusCode?: string) {
  if (statusCode === "STATUS_CODE_ERROR") {
    return { variant: "rose" as const, icon: IconServer, bar: "bg-rose-500", chip: "text-rose-600" }
  }
  return kindMap[kind] ?? { variant: "secondary" as const, icon: IconCpu, bar: "bg-slate-500", chip: "text-muted-foreground" }
}

export function SpanKindChip({ kind, statusCode }: { kind: SpanKind; statusCode?: string }) {
  const meta = spanKindMeta(kind, statusCode)
  const Icon = meta.icon
  return (
    <Badge variant={meta.variant}>
      <Icon />
      {kind}
    </Badge>
  )
}

export function SpanKindBadge(props: { kind: SpanKind; statusCode?: string }) {
  return <SpanKindChip {...props} />
}

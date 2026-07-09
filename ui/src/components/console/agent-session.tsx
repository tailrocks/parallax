import { Link } from "@tanstack/react-router"
import {
  IconAlertTriangle,
  IconArrowUpRight,
  IconBrain,
  IconCircleDot,
  IconTerminal2,
  IconTool,
} from "@tabler/icons-react"

import { HeatCell, buildHeatScale } from "@/components/console/heat-cell"
import { RelativeTime } from "@/components/console/relative-time"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { formatCount, formatDurationNs } from "@/lib/format"
import { cn } from "@/lib/utils"

export type AgentStepKind =
  | "INVOKE_AGENT"
  | "EXECUTE_TOOL"
  | "SHELL"
  | "OTHER"

export interface AgentStepData {
  spanId: string
  traceId: string
  kind: AgentStepKind
  name: string
  startNanos: string
  durationNs: string
  isError: boolean
  genAiOperation: string | null
  inputTokens: string | null
  outputTokens: string | null
}

export interface AgentSessionData {
  rootSpanId: string | null
  steps: AgentStepData[]
  totalInputTokens: string
  totalOutputTokens: string
  errorCount: number
  truncated: boolean
}

function tokenTotal(value: string): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : 0
}

function formatTokenTotal(value: string): string {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? formatCount(parsed) : value
}

function stepTone(step: AgentStepData) {
  if (step.isError) {
    return {
      icon: IconAlertTriangle,
      badge: "rose" as const,
      row: "border-rose-500/30 bg-rose-500/10",
    }
  }
  if (step.kind === "INVOKE_AGENT") {
    return { icon: IconBrain, badge: "blue" as const, row: "" }
  }
  if (step.kind === "SHELL") {
    return { icon: IconTerminal2, badge: "amber" as const, row: "" }
  }
  if (step.kind === "EXECUTE_TOOL") {
    return { icon: IconTool, badge: "emerald" as const, row: "" }
  }
  return { icon: IconCircleDot, badge: "outline" as const, row: "" }
}

function labelForKind(kind: AgentStepKind): string {
  return kind.toLowerCase().replace(/_/g, " ")
}

export function AgentSessionCard({ session }: { session: AgentSessionData }) {
  const durations = session.steps.map((step) => Number(step.durationNs))
  const scale = buildHeatScale(durations)
  const inputTokens = tokenTotal(session.totalInputTokens)
  const outputTokens = tokenTotal(session.totalOutputTokens)
  const showTokens = inputTokens > 0 || outputTokens > 0

  return (
    <Card size="sm">
      <CardHeader>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle className="text-sm">Agent session</CardTitle>
          <div className="flex flex-wrap items-center gap-1.5 text-xs">
            <Badge variant="secondary">
              {formatCount(session.steps.length)} steps
            </Badge>
            <Badge variant={session.errorCount > 0 ? "rose" : "outline"}>
              {formatCount(session.errorCount)} errors
            </Badge>
            {showTokens ? (
              <>
                <Badge variant="blue">
                  {formatTokenTotal(session.totalInputTokens)} in
                </Badge>
                <Badge variant="emerald">
                  {formatTokenTotal(session.totalOutputTokens)} out
                </Badge>
              </>
            ) : null}
            {session.truncated ? <Badge variant="amber">truncated</Badge> : null}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <ol className="flex flex-col gap-2">
          {session.steps.map((step) => {
            const tone = stepTone(step)
            const Icon = tone.icon
            return (
              <li
                key={step.spanId}
                data-testid="agent-step"
                className={cn(
                  "grid gap-3 rounded-lg border border-border/70 bg-background/70 px-3 py-2 text-sm md:grid-cols-[8rem_minmax(0,1fr)_8rem_8rem]",
                  tone.row
                )}
              >
                <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  <Icon className="size-4" aria-hidden="true" />
                  <Badge variant={tone.badge}>{labelForKind(step.kind)}</Badge>
                </span>
                <Link
                  to="/traces/$traceId"
                  params={{ traceId: step.traceId }}
                  search={{}}
                  className="inline-flex min-w-0 items-center gap-1 font-medium break-words hover:underline"
                >
                  <span className="min-w-0 truncate">{step.name}</span>
                  <IconArrowUpRight className="size-3.5 shrink-0" />
                </Link>
                <span className="text-xs text-muted-foreground tabular-nums md:text-right">
                  <RelativeTime nanos={step.startNanos} />
                </span>
                <span className="md:text-right">
                  <HeatCell value={Number(step.durationNs)} scale={scale}>
                    {formatDurationNs(step.durationNs)}
                  </HeatCell>
                </span>
              </li>
            )
          })}
        </ol>
      </CardContent>
    </Card>
  )
}

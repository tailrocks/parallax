import { useState } from "react"
import { Link } from "@tanstack/react-router"
import { IconArrowUpRight, IconRoute, IconUser } from "@tabler/icons-react"

import { AgentSessionCard } from "@/shared/console/agent-session"
import type { AgentSessionData } from "@/shared/console/agent-session"
import { EmptyState } from "@/shared/console/empty-state"
import { RelativeTime } from "@/shared/console/relative-time"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { Conversation, ScreenVisit, Session, UiAction } from "@/lib/api"
import { formatCount, formatDurationNs } from "@/lib/format"
import { cn } from "@/lib/utils"

import { ScreenVisitLane } from "./screen-visit-lane"
import { SessionJourney } from "./session-journey"
import type { JourneyError } from "./session-journey"

export function SessionsTab({
  sessions,
  visits,
  actions,
  conversations,
  errors,
  agentSession,
}: {
  sessions: Session[]
  visits: ScreenVisit[]
  actions: UiAction[]
  conversations: Conversation[]
  errors: JourneyError[]
  agentSession: AgentSessionData | null
}) {
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const selected =
    sessions.find((session) => session.sessionId === selectedId) ??
    sessions[0] ??
    null

  if (
    sessions.length === 0 &&
    visits.length === 0 &&
    actions.length === 0 &&
    conversations.length === 0
  ) {
    return (
      <EmptyState
        icon={IconUser}
        title="No interactive sessions"
        description="Nothing yet — this invocation has not emitted session.start events, screen visits, or UI actions."
      />
    )
  }

  const selectedVisits = selected
    ? visits.filter(
        (visit) =>
          visit.sessionId == null || visit.sessionId === selected.sessionId
      )
    : visits

  return (
    <div className="flex flex-col gap-4">
      {sessions.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Sessions</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="flex flex-wrap gap-2">
              {sessions.map((session) => (
                <li key={session.sessionId}>
                  <Button
                    size="sm"
                    variant={
                      selected?.sessionId === session.sessionId
                        ? "secondary"
                        : "outline"
                    }
                    onClick={() => setSelectedId(session.sessionId)}
                  >
                    <code className="max-w-36 truncate text-xs">
                      {session.sessionId}
                    </code>
                    {session.endNanos == null ? (
                      <Badge variant="blue">open</Badge>
                    ) : null}
                    {session.previousSessionId ? (
                      <span
                        className="text-xs text-muted-foreground"
                        title={`previous session ${session.previousSessionId}`}
                      >
                        ↳
                      </span>
                    ) : null}
                  </Button>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      ) : null}

      {selectedVisits.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Screen visits</CardTitle>
          </CardHeader>
          <CardContent>
            <ScreenVisitLane visits={selectedVisits} />
          </CardContent>
        </Card>
      ) : null}

      {actions.length > 0 ? <UiActionsCard actions={actions} /> : null}

      {selected ? (
        <Card>
          <CardHeader>
            <CardTitle className="inline-flex items-center gap-1.5 text-sm">
              <IconRoute className="size-4" />
              Journey
            </CardTitle>
          </CardHeader>
          <CardContent>
            <SessionJourney
              session={selected}
              visits={visits}
              actions={actions}
              errors={errors}
            />
          </CardContent>
        </Card>
      ) : null}

      {conversations.length > 0 ? (
        <ConversationsPanel
          conversations={conversations}
          agentSession={agentSession}
        />
      ) : null}
    </div>
  )
}

export function UiActionsCard({ actions }: { actions: UiAction[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">UI actions</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-hidden rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Action</TableHead>
                <TableHead className="w-36">Screen</TableHead>
                <TableHead className="w-28">Outcome</TableHead>
                <TableHead className="w-28 text-right">Duration</TableHead>
                <TableHead className="w-32 text-right">When</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {actions.map((action) => (
                <TableRow
                  key={`${action.traceId}-${action.startNanos}`}
                  className={cn(
                    action.hasError &&
                      "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
                  )}
                >
                  <TableCell>
                    <Link
                      to="/traces/$traceId"
                      params={{ traceId: action.traceId }}
                      className="inline-flex items-center gap-1 font-medium hover:underline"
                    >
                      {action.name}
                      <IconArrowUpRight className="size-3.5" />
                    </Link>
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {action.screenId ?? "-"}
                  </TableCell>
                  <TableCell>
                    {action.outcome ? (
                      <Badge
                        variant={
                          action.outcome === "success" ? "emerald" : "rose"
                        }
                      >
                        {action.outcome}
                      </Badge>
                    ) : (
                      "-"
                    )}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {action.durationMs.toFixed(1)} ms
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    <RelativeTime nanos={action.startNanos} />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  )
}

export function ConversationsPanel({
  conversations,
  agentSession,
}: {
  conversations: Conversation[]
  agentSession: AgentSessionData | null
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Agent conversations</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="overflow-hidden rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Conversation</TableHead>
                <TableHead className="w-32">Agent</TableHead>
                <TableHead className="w-28">Provider</TableHead>
                <TableHead className="w-20 text-right">Spans</TableHead>
                <TableHead className="w-28 text-right">Tokens in</TableHead>
                <TableHead className="w-28 text-right">Tokens out</TableHead>
                <TableHead className="w-32 text-right">Last</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {conversations.map((conversation) => (
                <TableRow key={conversation.conversationId}>
                  <TableCell>
                    <code className="max-w-44 truncate text-xs">
                      {conversation.conversationId}
                    </code>
                  </TableCell>
                  <TableCell className="text-xs">
                    {conversation.agentName ?? "-"}
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {conversation.providerName ?? "-"}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {formatCount(conversation.spanCount)}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {conversation.inputTokens != null
                      ? formatCount(conversation.inputTokens)
                      : "-"}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {conversation.outputTokens != null
                      ? formatCount(conversation.outputTokens)
                      : "-"}
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    <RelativeTime nanos={conversation.lastNanos} />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
        {agentSession ? <AgentSessionCard session={agentSession} /> : null}
      </CardContent>
    </Card>
  )
}

export function sessionDurationLabel(session: Session): string {
  if (session.endNanos == null) return "open"
  return formatDurationNs(
    (BigInt(session.endNanos) - BigInt(session.startNanos)).toString()
  )
}

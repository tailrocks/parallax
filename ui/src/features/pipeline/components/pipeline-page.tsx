import { IconActivity } from "@tabler/icons-react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { EmptyState } from "@/shared/console/empty-state"
import {
  formatCount,
  formatRate,
  parseCount,
  queuePressure,
  totalAccepted,
  totalDropped,
  type IngestDropRow,
  type IngestQueueRow,
  type PipelineStatus,
  type SamplingPolicyRow,
} from "@/features/pipeline/model/pipeline-status"

function PolicyTable({ policies }: { policies: readonly SamplingPolicyRow[] }) {
  if (policies.length === 0) {
    return (
      <EmptyState
        title="No sampling policy reported"
        description="The server did not attach a pipeline readout to this response."
        icon={IconActivity}
      />
    )
  }
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Signal</TableHead>
          <TableHead>Scope</TableHead>
          <TableHead>Rule</TableHead>
          <TableHead className="text-right">Keep rate</TableHead>
          <TableHead>Enforced by</TableHead>
          <TableHead>Policy</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {policies.map((policy) => (
          <TableRow key={policy.signal}>
            <TableCell className="font-medium">{policy.signal}</TableCell>
            <TableCell>{policy.service ?? "all services"}</TableCell>
            <TableCell>
              <Badge variant="blue">{policy.rule}</Badge>
            </TableCell>
            <TableCell className="text-right tabular-nums">{formatRate(policy.rate)}</TableCell>
            <TableCell className="text-muted-foreground">{policy.enforcedBy}</TableCell>
            <TableCell className="max-w-md text-muted-foreground">{policy.description}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}

function DropsTable({ drops }: { drops: readonly IngestDropRow[] }) {
  if (drops.length === 0) {
    return (
      <EmptyState
        title="No drop counters reported"
        description="The server did not attach a pipeline readout to this response."
        icon={IconActivity}
      />
    )
  }
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Signal</TableHead>
          <TableHead>Reason</TableHead>
          <TableHead className="text-right">Dropped</TableHead>
          <TableHead>What it means</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {drops.map((drop) => {
          const count = parseCount(drop.count)
          return (
            <TableRow key={`${drop.signal ?? "global"}:${drop.reason}`}>
              <TableCell className="font-medium">{drop.signal ?? "global"}</TableCell>
              <TableCell>
                <code className="rounded bg-muted px-1.5 py-0.5 text-xs">{drop.reason}</code>
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {count > 0n ? (
                  <Badge variant="destructive">{formatCount(drop.count)}</Badge>
                ) : (
                  <span className="text-muted-foreground">0</span>
                )}
              </TableCell>
              <TableCell className="max-w-md text-muted-foreground">{drop.detail}</TableCell>
            </TableRow>
          )
        })}
      </TableBody>
    </Table>
  )
}

function QueuesTable({ queues }: { queues: readonly IngestQueueRow[] }) {
  if (queues.length === 0) {
    return (
      <EmptyState
        title="No queue watermarks reported"
        description="The server did not attach a pipeline readout to this response."
        icon={IconActivity}
      />
    )
  }
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Signal</TableHead>
          <TableHead>Depth</TableHead>
          <TableHead className="text-right">High water</TableHead>
          <TableHead className="text-right">Accepted batches</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {queues.map((queue) => {
          const pressure = queuePressure(queue)
          const pct = queue.capacity > 0 ? Math.min(100, (queue.depth / queue.capacity) * 100) : 0
          return (
            <TableRow key={queue.signal}>
              <TableCell className="font-medium">{queue.signal}</TableCell>
              <TableCell>
                <div className="flex min-w-40 items-center gap-2">
                  <Progress
                    value={pct}
                    className="flex-1"
                    aria-label={`${queue.signal} queue depth`}
                  />
                  <span className="text-muted-foreground tabular-nums">
                    {queue.depth}/{queue.capacity}
                  </span>
                  {pressure === "full" ? (
                    <Badge variant="destructive">full</Badge>
                  ) : pressure === "filling" ? (
                    <Badge variant="amber">filling</Badge>
                  ) : null}
                </div>
              </TableCell>
              <TableCell className="text-right tabular-nums">{queue.highWater}</TableCell>
              <TableCell className="text-right tabular-nums">
                {formatCount(queue.accepted)}
              </TableCell>
            </TableRow>
          )
        })}
      </TableBody>
    </Table>
  )
}

export function PipelinePage({ status }: { status: PipelineStatus }) {
  const dropped = totalDropped(status.drops)
  const accepted = totalAccepted(status.queues)
  return (
    <div className="flex flex-col gap-4 p-4">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="font-heading text-xl font-semibold tracking-tight">Pipeline</h1>
        {dropped > 0n ? (
          <Badge variant="destructive">
            {dropped.toLocaleString("en-US")} dropped / {accepted.toLocaleString("en-US")} accepted
          </Badge>
        ) : (
          <Badge variant="green">
            nothing dropped / {accepted.toLocaleString("en-US")} accepted
          </Badge>
        )}
      </div>
      <p className="max-w-3xl text-sm text-muted-foreground">
        Declared sampling policy per signal plus live drop-reason accounting. The server keeps every
        batch it receives; a volume gap is either a declared producer-side rate below 100% or a
        named drop reason below — never silent.
      </p>
      <Card>
        <CardHeader>
          <CardTitle>Sampling policy</CardTitle>
        </CardHeader>
        <CardContent>
          <PolicyTable policies={status.policies} />
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Dropped by reason</CardTitle>
        </CardHeader>
        <CardContent>
          <DropsTable drops={status.drops} />
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Queues</CardTitle>
        </CardHeader>
        <CardContent>
          <QueuesTable queues={status.queues} />
        </CardContent>
      </Card>
    </div>
  )
}

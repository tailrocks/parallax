import { IconAlertTriangle, IconCode, IconGitBranch } from "@tabler/icons-react"

import { HeatCell, buildHeatScale } from "@/components/console/heat-cell"
import { Badge } from "@/components/ui/badge"
import { formatDurationNs } from "@/lib/format"
import type { GraphqlFieldNode, GraphqlOperation } from "@/lib/graphql-trace"
import { cn } from "@/lib/utils"

function bigintToNumber(value: bigint): number {
  const number = Number(value)
  return Number.isFinite(number) ? number : Number.MAX_SAFE_INTEGER
}

function FieldRows({
  nodes,
  depth = 0,
  onSelect,
}: {
  nodes: GraphqlFieldNode[]
  depth?: number
  onSelect: (spanId: string) => void
}) {
  const scale = buildHeatScale(
    nodes.map((node) => bigintToNumber(node.durationNs))
  )
  return (
    <ul className={cn(depth === 0 ? "space-y-1.5" : "mt-1.5 space-y-1.5")}>
      {nodes.map((node) => {
        const callLabel = `x${node.callCount.toLocaleString()}`
        return (
          <li key={node.path}>
            <button
              type="button"
              onClick={() => onSelect(node.spanId)}
              className="grid w-full grid-cols-[minmax(0,1fr)_6rem_6rem] items-center gap-3 rounded-md border border-border/70 bg-background/70 px-3 py-2 text-left text-sm hover:bg-accent/60"
              style={{ paddingLeft: depth * 16 + 12 }}
            >
              <span className="flex min-w-0 flex-wrap items-center gap-1.5">
                <IconGitBranch className="size-3.5 shrink-0 text-muted-foreground" />
                <span className="min-w-0 truncate font-medium">
                  {node.fieldName}
                </span>
                {node.callCount > 1 ? (
                  <Badge variant="amber">{callLabel}</Badge>
                ) : null}
                {node.hasError ? <Badge variant="rose">error</Badge> : null}
              </span>
              <span className="text-right text-xs text-muted-foreground tabular-nums">
                {formatDurationNs(node.selfDurationNs.toString())}
              </span>
              <span className="text-right text-xs tabular-nums">
                <HeatCell value={bigintToNumber(node.durationNs)} scale={scale}>
                  {formatDurationNs(node.durationNs.toString())}
                </HeatCell>
              </span>
            </button>
            {node.children.length > 0 ? (
              <FieldRows
                nodes={node.children}
                depth={depth + 1}
                onSelect={onSelect}
              />
            ) : null}
          </li>
        )
      })}
    </ul>
  )
}

function OperationSection({
  operation,
  onSelect,
}: {
  operation: GraphqlOperation
  onSelect: (spanId: string) => void
}) {
  const name = operation.operationName ?? "(anonymous)"
  const fieldErrorLabel = `${operation.fieldErrors.toLocaleString()} field ${
    operation.fieldErrors === 1 ? "error" : "errors"
  }`
  return (
    <section className="space-y-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <Badge variant="blue">{operation.operationType}</Badge>
          <span className="min-w-0 truncate text-sm font-medium">{name}</span>
          {operation.fieldErrors > 0 ? (
            <Badge variant="rose">{fieldErrorLabel}</Badge>
          ) : null}
        </div>
        <span className="text-xs text-muted-foreground tabular-nums">
          {formatDurationNs(operation.durationNs.toString())}
        </span>
      </div>

      <div className="grid grid-cols-[minmax(0,1fr)_6rem_6rem] gap-3 px-3 text-[11px] text-muted-foreground">
        <span>Field</span>
        <span className="text-right">Self</span>
        <span className="text-right">Total</span>
      </div>
      <FieldRows nodes={operation.roots} onSelect={onSelect} />

      {operation.document ? (
        <details className="rounded-md border border-border/70 bg-muted/20 px-3 py-2">
          <summary className="flex cursor-pointer items-center gap-1.5 text-xs font-medium text-muted-foreground">
            <IconCode className="size-3.5" />
            Document
          </summary>
          <pre className="mt-2 overflow-x-auto rounded-md bg-background/70 p-3 text-xs">
            <code>{operation.document}</code>
          </pre>
        </details>
      ) : null}
    </section>
  )
}

export function GraphqlOperationCard({
  operations,
  onSelect,
}: {
  operations: GraphqlOperation[]
  onSelect: (spanId: string) => void
}) {
  if (operations.length === 0) return null

  return (
    <div className="space-y-4">
      {operations.map((operation) => (
        <OperationSection
          key={operation.operationSpanId}
          operation={operation}
          onSelect={onSelect}
        />
      ))}
      {operations.some((operation) => operation.fieldErrors > 0) ? (
        <p className="flex items-center gap-1.5 text-xs text-rose-700 dark:text-rose-300">
          <IconAlertTriangle className="size-3.5" />
          Partial field errors
        </p>
      ) : null}
    </div>
  )
}

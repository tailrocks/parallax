import { useEffect, useMemo, useState } from "react"
import { Link, useNavigate } from "@tanstack/react-router"
import { Background, Handle, MarkerType, Position, ReactFlow } from "@xyflow/react"
import type { Edge, Node, NodeProps } from "@xyflow/react"
import "@xyflow/react/dist/style.css"

import { ServiceDot } from "@/shared/console/service-dot"
import { Badge } from "@/components/ui/badge"
import type { ServiceMapEdge, ServiceMapNode } from "@/features/ecosystem/model/service-map"
import { serviceMapEdgePresentation } from "@/features/ecosystem/model/ecosystem-topology"
import { nodePresentation } from "@/features/ecosystem/model/service-map-presentation"
import {
  ECOSYSTEM_NODE_HEIGHT,
  ECOSYSTEM_NODE_WIDTH,
  ecosystemTopologyKey,
  fallbackEcosystemLayout,
  layoutEcosystem,
} from "@/features/ecosystem/model/service-map-layout"
import type { EcosystemLayout } from "@/features/ecosystem/model/service-map-layout"
import { formatCount, formatDurationNs, formatPercent } from "@/shared/format"
import { rangeLinkSearch } from "@/domain/time-range/range"
import type { ResolvedRange } from "@/domain/time-range/range"
import { cn } from "@/lib/utils"
import { EcosystemLegend } from "@/features/ecosystem/components/ecosystem-legend"

const MIN_HEIGHT = 420
const NODE_WIDTH = ECOSYSTEM_NODE_WIDTH
const NODE_HEIGHT = ECOSYSTEM_NODE_HEIGHT

function count(value: string): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : 0
}

function edgeRate(edge: ServiceMapEdge): number {
  const calls = count(edge.callCount)
  return calls > 0 ? count(edge.errorCount) / calls : 0
}

/** Async ELK layout with the deterministic fallback rendered immediately;
 * stale worker results never overwrite a newer topology. */
function useEcosystemLayout(nodes: ServiceMapNode[], edges: ServiceMapEdge[]): EcosystemLayout {
  const key = ecosystemTopologyKey({ nodes, edges })
  const [resolved, setResolved] = useState<{
    key: string
    layout: EcosystemLayout
  }>(() => ({ key, layout: fallbackEcosystemLayout({ nodes, edges }) }))
  const layout = useMemo(
    () => (resolved.key === key ? resolved.layout : fallbackEcosystemLayout({ nodes, edges })),
    [edges, key, nodes, resolved]
  )
  useEffect(() => {
    let current = true
    void layoutEcosystem({ nodes, edges }).then((next) => {
      if (current) setResolved({ key, layout: next })
    })
    return () => {
      current = false
    }
  }, [edges, key, nodes])
  return layout
}

interface ServiceNodeData extends Record<string, unknown> {
  node: ServiceMapNode
  dimmed: boolean
  range: ResolvedRange
}

/** React Flow custom node (operator rule, 2026-07-17: service graphs render
 * with React Flow). Keeps the plan-162 language: ServiceDot identity + kind
 * glyph + stats. */
function ServiceGraphNode({ data }: NodeProps<Node<ServiceNodeData>>) {
  const { node, dimmed, range } = data
  const errors = count(node.errorCount)
  const { Icon, color } = nodePresentation(node.kind)
  const className = cn(
    "flex h-full w-full flex-col justify-center gap-1 rounded-lg border bg-card px-3 py-2 text-sm shadow-sm hover:bg-muted/50",
    errors > 0 && "border-rose-500/50",
    dimmed && "opacity-30"
  )
  const body = (
    <>
      <span className="inline-flex min-w-0 items-center gap-1.5 font-medium">
        <Icon className={cn("size-3.5 shrink-0", color)} />
        <ServiceDot name={node.name} />
        <span className="truncate">{node.name}</span>
      </span>
      <span className="truncate text-[10px] font-medium tracking-wide text-muted-foreground uppercase">
        {node.kind}
        {node.system && node.system !== node.name ? ` · ${node.system}` : ""}
      </span>
      <span className="text-xs text-muted-foreground tabular-nums">
        {formatCount(count(node.spanCount))} spans · {formatCount(errors)} errors
        {node.p95Ms != null ? ` · p95 ${formatDurationNs(node.p95Ms * 1_000_000)}` : ""}
      </span>
    </>
  )
  return (
    <div style={{ width: NODE_WIDTH, height: NODE_HEIGHT }}>
      <Handle type="target" position={Position.Left} className="!opacity-0" />
      {node.kind === "cli" ? (
        <Link to="/invocations" search={{ q: node.name }} className={className}>
          {body}
        </Link>
      ) : node.kind === "service" || node.kind === "browser" ? (
        <Link
          to="/services/$service"
          params={{ service: node.name }}
          search={rangeLinkSearch(range)}
          className={className}
        >
          {body}
        </Link>
      ) : (
        <div
          className={cn(
            className,
            "cursor-default focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
          )}
          role="group"
          tabIndex={0}
          aria-label={`${node.kind} dependency ${node.name}${node.system ? `, system ${node.system}` : ""}`}
        >
          {body}
        </div>
      )}
      <Handle type="source" position={Position.Right} className="!opacity-0" />
    </div>
  )
}

const nodeTypes = { service: ServiceGraphNode }

function toFlowEdges(edges: ServiceMapEdge[], dimmedNodeIds: ReadonlySet<string>): Edge[] {
  return edges.map((edge) => {
    const dimmed = dimmedNodeIds.has(edge.source) || dimmedNodeIds.has(edge.target)
    const presentation = serviceMapEdgePresentation(edge)
    const summary = `${formatCount(count(edge.callCount))} calls · ${formatPercent(edgeRate(edge))} errors · p50 ${formatDurationNs(edge.p50Ms * 1_000_000)} · p95 ${formatDurationNs(edge.p95Ms * 1_000_000)}`
    return {
      id: `${edge.source}->${edge.target}`,
      source: edge.source,
      target: edge.target,
      label: summary,
      ariaLabel: `${edge.source} → ${edge.target}: ${summary}`,
      className: presentation.className,
      labelStyle: { fill: "var(--muted-foreground)", fontSize: 10 },
      labelBgStyle: { fill: "var(--background)", fillOpacity: 0.9 },
      labelBgPadding: [5, 2],
      style: {
        stroke: presentation.hasError ? "var(--chart-error)" : "var(--border)",
        strokeWidth: presentation.width,
        opacity: dimmed ? 0.25 : 1,
      },
      markerEnd: {
        type: MarkerType.ArrowClosed,
        color: presentation.hasError ? "var(--chart-error)" : "var(--muted-foreground)",
      },
    }
  })
}

export function EcosystemGraph({
  nodes,
  edges,
  range,
  dimmedNodeIds = new Set<string>(),
  hiddenNodeCount = 0,
  hiddenEdgeCount = 0,
}: {
  nodes: ServiceMapNode[]
  edges: ServiceMapEdge[]
  range: ResolvedRange
  dimmedNodeIds?: ReadonlySet<string>
  hiddenNodeCount?: number
  hiddenEdgeCount?: number
}) {
  const navigate = useNavigate()
  const layout = useEcosystemLayout(nodes, edges)
  const height = Math.max(MIN_HEIGHT, layout.height)
  const positionByName = new Map(
    layout.positions.map((position) => [position.id, position] as const)
  )

  const flowNodes: Array<Node<ServiceNodeData>> = nodes.map((node) => {
    const position = positionByName.get(node.name)
    return {
      id: node.name,
      type: "service",
      position: {
        x: (position?.x ?? 0) - NODE_WIDTH / 2,
        y: (position?.y ?? 0) - NODE_HEIGHT / 2,
      },
      width: NODE_WIDTH,
      height: NODE_HEIGHT,
      data: { node, dimmed: dimmedNodeIds.has(node.name), range },
    }
  })

  const flowEdges = toFlowEdges(edges, dimmedNodeIds)

  if (nodes.length === 0) {
    return (
      <div className="flex items-center justify-between gap-3">
        <p className="text-sm text-muted-foreground">No service edges.</p>
        <Badge variant="secondary">trace-path</Badge>
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between gap-3">
        <span className="text-sm text-muted-foreground">
          {formatCount(nodes.length)} nodes · {formatCount(edges.length)} edges
          {hiddenNodeCount + hiddenEdgeCount > 0 ? (
            <Badge variant="secondary" className="ml-2">
              {formatCount(hiddenNodeCount + hiddenEdgeCount)} hidden
            </Badge>
          ) : null}
        </span>
        <span className="flex items-center gap-2 text-xs text-muted-foreground">
          <Badge variant="secondary">trace-path</Badge>
        </span>
      </div>
      <div
        className="overflow-hidden rounded-lg border bg-background"
        style={{ height }}
        aria-label="service dependency graph"
        role="group"
      >
        <ReactFlow
          nodes={flowNodes}
          edges={flowEdges}
          nodeTypes={nodeTypes}
          fitView
          minZoom={0.2}
          maxZoom={2}
          nodesDraggable={false}
          nodesConnectable={false}
          edgesFocusable
          onEdgeClick={(_, edge) => {
            void navigate({
              to: "/traces",
              search: { ...rangeLinkSearch(range), service: edge.source },
            })
          }}
        >
          <Background gap={24} className="!bg-background" />
        </ReactFlow>
      </div>
      <EcosystemLegend />
    </div>
  )
}

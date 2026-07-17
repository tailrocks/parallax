import { useEffect, useMemo, useState } from "react"
import { Link } from "@tanstack/react-router"
import { IconTerminal2, IconWorld } from "@tabler/icons-react"

import { ServiceDot } from "@/components/console/service-dot"
import { Badge } from "@/components/ui/badge"
import type { ServiceMapEdge, ServiceMapNode } from "@/lib/api"
import {
  ECOSYSTEM_NODE_HEIGHT,
  ECOSYSTEM_NODE_WIDTH,
  ecosystemTopologyKey,
  fallbackEcosystemLayout,
  layoutEcosystem,
} from "@/lib/ecosystem-layout"
import type { EcosystemLayout } from "@/lib/ecosystem-layout"
import { formatCount, formatDurationNs, formatPercent } from "@/lib/format"
import type { ResolvedRange } from "@/lib/range"
import { rangeLinkSearch } from "@/lib/range"
import { cn } from "@/lib/utils"

const MIN_WIDTH = 960
const MIN_HEIGHT = 420
const NODE_WIDTH = ECOSYSTEM_NODE_WIDTH
const NODE_HEIGHT = ECOSYSTEM_NODE_HEIGHT

interface PositionedNode extends ServiceMapNode {
  x: number
  y: number
}

function count(value: string): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : 0
}

function edgeRate(edge: ServiceMapEdge): number {
  const calls = count(edge.callCount)
  return calls > 0 ? count(edge.errorCount) / calls : 0
}

function positionedNodes(
  nodes: ServiceMapNode[],
  layout: EcosystemLayout
): PositionedNode[] {
  const byName = new Map(nodes.map((node) => [node.name, node]))
  return layout.positions
    .map(({ id, x, y }) => ({
      name: id,
      kind: "service" as const,
      lastSeenNanos: "0",
      spanCount: "0",
      errorCount: "0",
      p95Ms: null,
      ...byName.get(id),
      x: x + NODE_WIDTH / 2,
      y: y + NODE_HEIGHT / 2,
    }))
    .sort((a, b) => a.name.localeCompare(b.name))
}

function useEcosystemLayout(
  nodes: ServiceMapNode[],
  edges: ServiceMapEdge[]
): EcosystemLayout {
  const key = ecosystemTopologyKey({ nodes, edges })
  const [resolved, setResolved] = useState<{
    key: string
    layout: EcosystemLayout
  }>(() => ({ key, layout: fallbackEcosystemLayout({ nodes, edges }) }))
  const layout = useMemo(
    () =>
      resolved.key === key
        ? resolved.layout
        : fallbackEcosystemLayout({ nodes, edges }),
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

function EdgePath({
  edge,
  source,
  target,
}: {
  edge: ServiceMapEdge
  source: PositionedNode
  target: PositionedNode
}) {
  const startX = source.x + NODE_WIDTH / 2
  const endX = target.x - NODE_WIDTH / 2
  const midX = (startX + endX) / 2
  const path = `M ${startX} ${source.y} C ${midX} ${source.y}, ${midX} ${target.y}, ${endX} ${target.y}`
  const hasError = count(edge.errorCount) > 0
  return (
    <path
      d={path}
      className={cn(
        "fill-none stroke-muted-foreground/50",
        hasError && "stroke-rose-500/80"
      )}
      strokeWidth={Math.max(
        1.5,
        Math.min(6, Math.log2(count(edge.callCount) + 1))
      )}
      markerEnd="url(#ecosystem-arrow)"
    />
  )
}

export function EcosystemGraph({
  nodes,
  edges,
  range,
}: {
  nodes: ServiceMapNode[]
  edges: ServiceMapEdge[]
  range: ResolvedRange
}) {
  const layout = useEcosystemLayout(nodes, edges)
  const positioned = positionedNodes(nodes, layout)
  const width = Math.max(MIN_WIDTH, layout.width)
  const height = Math.max(MIN_HEIGHT, layout.height)
  const byName = new Map(positioned.map((node) => [node.name, node]))
  const renderedEdges = edges
    .map((edge) => ({
      edge,
      source: byName.get(edge.source),
      target: byName.get(edge.target),
    }))
    .filter(
      (
        item
      ): item is {
        edge: ServiceMapEdge
        source: PositionedNode
        target: PositionedNode
      } => Boolean(item.source && item.target)
    )

  if (positioned.length === 0) {
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
          {formatCount(positioned.length)} services ·{" "}
          {formatCount(edges.length)} edges
        </span>
        <span className="flex items-center gap-2 text-xs text-muted-foreground">
          <span className="inline-flex items-center gap-1">
            <IconTerminal2 className="size-3.5 text-violet-500" /> cli
          </span>
          <span className="inline-flex items-center gap-1">
            <IconWorld className="size-3.5 text-sky-500" /> browser
          </span>
          <Badge variant="secondary">trace-path</Badge>
        </span>
      </div>
      <div
        className="relative overflow-hidden rounded-lg border bg-background"
        style={{ minHeight: height }}
      >
        <svg
          viewBox={`0 0 ${width} ${height}`}
          className="absolute inset-0 size-full text-muted-foreground"
          role="img"
          aria-label="service dependency graph"
        >
          <defs>
            <marker
              id="ecosystem-arrow"
              viewBox="0 0 10 10"
              refX="9"
              refY="5"
              markerWidth="6"
              markerHeight="6"
              orient="auto-start-reverse"
            >
              <path d="M 0 0 L 10 5 L 0 10 z" className="fill-current" />
            </marker>
          </defs>
          {renderedEdges.map(({ edge, source, target }) => (
            <EdgePath
              key={`${edge.source}-${edge.target}`}
              edge={edge}
              source={source}
              target={target}
            />
          ))}
        </svg>

        {positioned.map((node) => {
          const errors = count(node.errorCount)
          const className = cn(
            "absolute flex flex-col gap-1 rounded-lg border bg-card px-3 py-2 text-sm shadow-sm hover:bg-muted/50",
            errors > 0 && "border-rose-500/50"
          )
          const style = {
            left: `${((node.x - NODE_WIDTH / 2) / width) * 100}%`,
            top: `${((node.y - NODE_HEIGHT / 2) / height) * 100}%`,
            width: NODE_WIDTH,
            minHeight: NODE_HEIGHT,
          }
          const body = (
            <>
              <span className="inline-flex min-w-0 items-center gap-1.5 font-medium">
                {node.kind === "cli" ? (
                  <IconTerminal2 className="size-3.5 shrink-0 text-violet-500" />
                ) : node.kind === "browser" ? (
                  <IconWorld className="size-3.5 shrink-0 text-sky-500" />
                ) : null}
                <ServiceDot name={node.name} />
                <span className="truncate">{node.name}</span>
              </span>
              <span className="text-xs text-muted-foreground tabular-nums">
                {formatCount(count(node.spanCount))} spans
                {node.p95Ms != null
                  ? ` · p95 ${formatDurationNs(node.p95Ms * 1_000_000)}`
                  : ""}
              </span>
            </>
          )
          if (node.kind === "cli") {
            return (
              <Link
                key={node.name}
                to="/invocations"
                search={{ q: node.name }}
                className={className}
                style={style}
              >
                {body}
              </Link>
            )
          }
          return (
            <Link
              key={node.name}
              to="/services/$service"
              params={{ service: node.name }}
              search={rangeLinkSearch(range)}
              className={className}
              style={style}
            >
              {body}
            </Link>
          )
        })}

        {renderedEdges.map(({ edge, source, target }) => {
          const x = ((source.x + target.x) / 2 / width) * 100
          const y = ((source.y + target.y) / 2 / height) * 100
          return (
            <Link
              key={`${edge.source}-${edge.target}-label`}
              to="/traces"
              search={{ ...rangeLinkSearch(range), service: edge.source }}
              className="absolute rounded-md border bg-background/95 px-2 py-1 text-xs shadow-sm hover:bg-muted/80"
              style={{
                left: `${x}%`,
                top: `${y}%`,
                transform: "translate(-50%, -50%)",
              }}
            >
              <span className="font-mono">
                {edge.source} -&gt; {edge.target}
              </span>
              <span className="ml-2 text-muted-foreground">
                {formatCount(count(edge.callCount))} calls ·{" "}
                {formatPercent(edgeRate(edge))} errors · p95{" "}
                {formatDurationNs(edge.p95Ms * 1_000_000)}
              </span>
            </Link>
          )
        })}
      </div>
    </div>
  )
}

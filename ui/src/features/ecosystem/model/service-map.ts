export type ServiceMapNodeKind = "cli" | "browser" | "service"

export type ServiceMapNode = {
  readonly name: string
  readonly kind: ServiceMapNodeKind
  readonly lastSeenNanos: string
  readonly spanCount: string
  readonly errorCount: string
  readonly p95Ms: number | null
}

export type ServiceMapEdge = {
  readonly source: string
  readonly target: string
  readonly callCount: string
  readonly errorCount: string
  readonly p50Ms: number
  readonly p95Ms: number
}

export type ServiceMap = {
  readonly nodes: readonly ServiceMapNode[]
  readonly edges: readonly ServiceMapEdge[]
}

const NODE_KINDS = new Set<ServiceMapNodeKind>(["cli", "browser", "service"])

export function mapServiceMapNode(raw: {
  readonly name: string
  readonly kind: string
  readonly lastSeenNanos: string
  readonly spanCount: string
  readonly errorCount: string
  readonly p95Ms: number | null
}): ServiceMapNode {
  const kind = NODE_KINDS.has(raw.kind as ServiceMapNodeKind)
    ? (raw.kind as ServiceMapNodeKind)
    : "service"
  return {
    name: raw.name,
    kind,
    lastSeenNanos: raw.lastSeenNanos,
    spanCount: raw.spanCount,
    errorCount: raw.errorCount,
    p95Ms: raw.p95Ms,
  }
}

export function mapServiceMapEdge(raw: {
  readonly source: string
  readonly target: string
  readonly callCount: string
  readonly errorCount: string
  readonly p50Ms: number
  readonly p95Ms: number
}): ServiceMapEdge {
  return {
    source: raw.source,
    target: raw.target,
    callCount: raw.callCount,
    errorCount: raw.errorCount,
    p50Ms: raw.p50Ms,
    p95Ms: raw.p95Ms,
  }
}

export function mapServiceMap(raw: {
  readonly nodes: readonly {
    readonly name: string
    readonly kind: string
    readonly lastSeenNanos: string
    readonly spanCount: string
    readonly errorCount: string
    readonly p95Ms: number | null
  }[]
  readonly edges: readonly {
    readonly source: string
    readonly target: string
    readonly callCount: string
    readonly errorCount: string
    readonly p50Ms: number
    readonly p95Ms: number
  }[]
}): ServiceMap {
  return {
    nodes: raw.nodes.map(mapServiceMapNode),
    edges: raw.edges.map(mapServiceMapEdge),
  }
}

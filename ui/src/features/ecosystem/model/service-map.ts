type KnownServiceMapNodeKind = "browser" | "cli" | "database" | "external" | "queue" | "service"

/** Unknown backend kinds survive as strings instead of collapsing into
 * `service`; the UI can identify them while retaining the typed common path. */
export type ServiceMapNodeKind = KnownServiceMapNodeKind | (string & {})

export type ServiceMapNode = {
  readonly name: string
  readonly kind: ServiceMapNodeKind
  readonly system: string | null
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

const NODE_KINDS = new Set<KnownServiceMapNodeKind>([
  "browser",
  "cli",
  "database",
  "external",
  "queue",
  "service",
])

export function mapServiceMapNode(raw: {
  readonly name: string
  readonly kind: string
  readonly system: string | null
  readonly lastSeenNanos: string
  readonly spanCount: string
  readonly errorCount: string
  readonly p95Ms: number | null
}): ServiceMapNode {
  const kind = NODE_KINDS.has(raw.kind as KnownServiceMapNodeKind)
    ? (raw.kind as KnownServiceMapNodeKind)
    : raw.kind
  return {
    name: raw.name,
    kind,
    system: raw.system?.trim() ? raw.system : null,
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
    readonly system: string | null
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

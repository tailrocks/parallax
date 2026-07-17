export interface GraphqlTraceSpan {
  spanId: string
  parentSpanId: string | null
  tsNanos: string
  durationNs: string
  name: string
  statusCode: string
  attributes: string
}

export interface GraphqlFieldNode {
  path: string
  fieldName: string
  spanId: string
  durationNs: bigint
  selfDurationNs: bigint
  hasError: boolean
  callCount: number
  children: GraphqlFieldNode[]
}

export interface GraphqlOperation {
  operationSpanId: string
  operationType: string
  operationName: string | null
  document: string | null
  durationNs: bigint
  fieldErrors: number
  roots: GraphqlFieldNode[]
}

interface FieldEntry {
  span: GraphqlTraceSpan
  fieldName: string
  pathAttr: string | null
  operationSpanId: string
  path: string
  parentPath: string | null
  durationNs: bigint
  startNs: bigint
  hasError: boolean
}

interface MutableFieldNode {
  path: string
  fieldName: string
  spanId: string
  durationNs: bigint
  selfDurationNs: bigint
  hasError: boolean
  callCount: number
  startNs: bigint
  selectedDurationNs: bigint
  parentPath: string | null
  children: MutableFieldNode[]
}

function parseAttributes(json: string): Record<string, unknown> {
  try {
    const parsed: unknown = JSON.parse(json)
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? (parsed as Record<string, unknown>)
      : {}
  } catch {
    return {}
  }
}

function stringAttr(attributes: Record<string, unknown>, key: string): string | null {
  const value = attributes[key]
  if (typeof value === "string") return value
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value)
  }
  return null
}

function durationNs(span: GraphqlTraceSpan): bigint {
  try {
    const duration = BigInt(span.durationNs)
    return duration > 0n ? duration : 0n
  } catch {
    return 0n
  }
}

function startNs(span: GraphqlTraceSpan): bigint {
  try {
    return BigInt(span.tsNanos)
  } catch {
    return 0n
  }
}

function parentPathFrom(path: string): string | null {
  const index = path.lastIndexOf(".")
  return index > 0 ? path.slice(0, index) : null
}

function compareNodes(a: MutableFieldNode, b: MutableFieldNode): number {
  if (a.startNs < b.startNs) return -1
  if (a.startNs > b.startNs) return 1
  const byPath = a.path.localeCompare(b.path)
  if (byPath !== 0) return byPath
  return a.spanId.localeCompare(b.spanId)
}

function finalizeNode(node: MutableFieldNode): GraphqlFieldNode {
  const children = node.children.sort(compareNodes).map(finalizeNode)
  const childDuration = children.reduce((total, child) => total + child.durationNs, 0n)
  const selfDurationNs = node.durationNs > childDuration ? node.durationNs - childDuration : 0n
  return {
    path: node.path,
    fieldName: node.fieldName,
    spanId: node.spanId,
    durationNs: node.durationNs,
    selfDurationNs,
    hasError: node.hasError,
    callCount: node.callCount,
    children,
  }
}

function nearestOperationSpanId(
  span: GraphqlTraceSpan,
  byId: ReadonlyMap<string, GraphqlTraceSpan>,
  operationIds: ReadonlySet<string>
): string | null {
  let parentId = span.parentSpanId
  const seen = new Set<string>()
  while (parentId && !seen.has(parentId)) {
    if (operationIds.has(parentId)) return parentId
    seen.add(parentId)
    parentId = byId.get(parentId)?.parentSpanId ?? null
  }
  return null
}

export function buildGraphqlOperations(spans: readonly GraphqlTraceSpan[]): GraphqlOperation[] {
  const byId = new Map(spans.map((span) => [span.spanId, span]))
  const attrBySpanId = new Map(spans.map((span) => [span.spanId, parseAttributes(span.attributes)]))
  const operationIds = new Set(
    spans
      .filter((span) =>
        Boolean(stringAttr(attrBySpanId.get(span.spanId) ?? {}, GRAPHQL_OPERATION_TYPE))
      )
      .map((span) => span.spanId)
  )
  if (operationIds.size === 0) return []

  const rawFields: FieldEntry[] = []
  const fieldBySpanId = new Map<string, FieldEntry>()

  for (const span of spans) {
    const attributes = attrBySpanId.get(span.spanId) ?? {}
    const fieldName = stringAttr(attributes, GRAPHQL_FIELD_NAME)
    if (!fieldName) continue
    // A span can be both the operation and its only resolved field (a
    // gateway resolver emitting one span): it is its own operation.
    const operationSpanId = operationIds.has(span.spanId)
      ? span.spanId
      : nearestOperationSpanId(span, byId, operationIds)
    if (!operationSpanId) continue
    const entry: FieldEntry = {
      span,
      fieldName,
      pathAttr: stringAttr(attributes, GRAPHQL_FIELD_PATH),
      operationSpanId,
      path: "",
      parentPath: null,
      durationNs: durationNs(span),
      startNs: startNs(span),
      hasError: span.statusCode === "STATUS_CODE_ERROR",
    }
    rawFields.push(entry)
    fieldBySpanId.set(span.spanId, entry)
  }

  const pathFor = (entry: FieldEntry, seen = new Set<string>()): string => {
    if (entry.path) return entry.path
    if (entry.pathAttr) {
      entry.path = entry.pathAttr
      return entry.path
    }
    const parent = entry.span.parentSpanId ? fieldBySpanId.get(entry.span.parentSpanId) : null
    if (!parent || seen.has(parent.span.spanId)) {
      entry.path = entry.fieldName
      return entry.path
    }
    seen.add(parent.span.spanId)
    entry.path = `${pathFor(parent, seen)}.${entry.fieldName}`
    return entry.path
  }

  for (const entry of rawFields) {
    const path = pathFor(entry)
    if (entry.pathAttr) {
      entry.parentPath = parentPathFrom(entry.pathAttr)
    } else {
      const parent = entry.span.parentSpanId ? fieldBySpanId.get(entry.span.parentSpanId) : null
      entry.parentPath = parent ? pathFor(parent) : parentPathFrom(path)
    }
  }

  return [...operationIds]
    .map((operationSpanId) => {
      const operationSpan = byId.get(operationSpanId)!
      const attributes = attrBySpanId.get(operationSpanId) ?? {}
      const fields = rawFields.filter((field) => field.operationSpanId === operationSpanId)
      const nodes = new Map<string, MutableFieldNode>()

      for (const field of fields) {
        const existing = nodes.get(field.path)
        if (existing) {
          existing.durationNs += field.durationNs
          existing.callCount += 1
          existing.hasError = existing.hasError || field.hasError
          if (
            field.durationNs > existing.selectedDurationNs ||
            (field.durationNs === existing.selectedDurationNs && field.startNs < existing.startNs)
          ) {
            existing.spanId = field.span.spanId
            existing.startNs = field.startNs
            existing.selectedDurationNs = field.durationNs
          }
          continue
        }
        nodes.set(field.path, {
          path: field.path,
          fieldName: field.fieldName,
          spanId: field.span.spanId,
          durationNs: field.durationNs,
          selfDurationNs: field.durationNs,
          hasError: field.hasError,
          callCount: 1,
          startNs: field.startNs,
          selectedDurationNs: field.durationNs,
          parentPath: field.parentPath,
          children: [],
        })
      }

      const roots: MutableFieldNode[] = []
      for (const node of nodes.values()) {
        const parent = node.parentPath ? nodes.get(node.parentPath) : null
        if (parent && parent !== node) {
          parent.children.push(node)
        } else {
          roots.push(node)
        }
      }

      return {
        operationSpanId,
        operationType: stringAttr(attributes, GRAPHQL_OPERATION_TYPE) ?? "graphql",
        operationName: stringAttr(attributes, GRAPHQL_OPERATION_NAME),
        document: stringAttr(attributes, GRAPHQL_DOCUMENT),
        durationNs: durationNs(operationSpan),
        fieldErrors: fields.filter((field) => field.hasError).length,
        roots: roots.sort(compareNodes).map(finalizeNode),
      }
    })
    .filter((operation) => operation.roots.length > 0)
    .sort((a, b) => {
      const aSpan = byId.get(a.operationSpanId)!
      const bSpan = byId.get(b.operationSpanId)!
      const aStart = startNs(aSpan)
      const bStart = startNs(bSpan)
      if (aStart < bStart) return -1
      if (aStart > bStart) return 1
      return a.operationSpanId.localeCompare(b.operationSpanId)
    })
}
import {
  GRAPHQL_DOCUMENT,
  GRAPHQL_FIELD_NAME,
  GRAPHQL_FIELD_PATH,
  GRAPHQL_OPERATION_NAME,
  GRAPHQL_OPERATION_TYPE,
} from "@/shared/semconv"

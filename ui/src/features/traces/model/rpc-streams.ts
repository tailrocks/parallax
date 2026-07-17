export interface RpcTraceSpan {
  spanId: string
  parentSpanId: string | null
  tsNanos: string
  durationNs: string
  service: string
  name: string
  kind: string
  statusCode: string
  attributes: string
}

export interface RpcTraceEvent {
  spanId: string
  spanName: string
  service: string
  name: string
  timeUnixNano: string
  attributes: string
}

export interface RpcMessage {
  id: number | null
  type: "SENT" | "RECEIVED" | "unknown"
  timeUnixNano: string
  size: number | null
}

export interface RpcStreamInfo {
  spanId: string
  system: string
  method: string
  startNanos: string
  durationNs: string
  grpcStatusCode: number | null
  outcome: "ok" | "deadline_exceeded" | "cancelled" | "error" | null
  messages: RpcMessage[]
  truncated: boolean
}

export interface MessagingSummary {
  producer: number
  consumer: number
  batchMax: number
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

function numberAttr(attributes: Record<string, unknown>, key: string): number | null {
  const value = attributes[key]
  const parsed =
    typeof value === "number" ? value : typeof value === "string" ? Number(value) : Number.NaN
  return Number.isFinite(parsed) ? parsed : null
}

function compareByTime(a: RpcMessage, b: RpcMessage): number {
  const aTime = BigInt(a.timeUnixNano || "0")
  const bTime = BigInt(b.timeUnixNano || "0")
  if (aTime < bTime) return -1
  if (aTime > bTime) return 1
  return (a.id ?? 0) - (b.id ?? 0)
}

function messageType(value: string | null): RpcMessage["type"] {
  const normalized = value?.toUpperCase()
  if (normalized === "SENT" || normalized === "RECEIVED") return normalized
  return "unknown"
}

function isRpcMessageEvent(name: string): boolean {
  return name === "message" || name === "rpc.message" || name.startsWith("rpc.message.")
}

function grpcOutcome(
  grpcStatusCode: number | null,
  spanStatusCode: string
): RpcStreamInfo["outcome"] {
  if (grpcStatusCode === 4) return "deadline_exceeded"
  if (grpcStatusCode === 1) return "cancelled"
  if (spanStatusCode === "STATUS_CODE_ERROR" || (grpcStatusCode != null && grpcStatusCode !== 0)) {
    return "error"
  }
  if (grpcStatusCode === 0 || spanStatusCode === "STATUS_CODE_UNSET") {
    return "ok"
  }
  return null
}

export function grpcStatusLabel(code: number | null): string | null {
  if (code === 4) return "DEADLINE_EXCEEDED (gRPC 4)"
  if (code === 1) return "CANCELLED (gRPC 1)"
  return null
}

export function parseGrpcStatusCode(value: string | number | null): number | null {
  if (value == null || value === "") return null
  const parsed = typeof value === "number" ? value : Number(value)
  return Number.isFinite(parsed) ? parsed : null
}

export function buildRpcStreams(
  spans: readonly RpcTraceSpan[],
  events: readonly RpcTraceEvent[],
  truncated = false
): RpcStreamInfo[] {
  const eventsBySpan = new Map<string, RpcMessage[]>()
  for (const event of events) {
    if (!isRpcMessageEvent(event.name)) continue
    const attributes = parseAttributes(event.attributes)
    const messages = eventsBySpan.get(event.spanId) ?? []
    messages.push({
      id: numberAttr(attributes, "rpc.message.id"),
      type: messageType(stringAttr(attributes, "rpc.message.type")),
      timeUnixNano: event.timeUnixNano,
      size: numberAttr(attributes, "rpc.message.compressed_size"),
    })
    eventsBySpan.set(event.spanId, messages)
  }

  return spans
    .map((span) => {
      const attributes = parseAttributes(span.attributes)
      const system = stringAttr(attributes, "rpc.system")
      if (!system) return null
      const messages = (eventsBySpan.get(span.spanId) ?? []).sort(compareByTime)
      if (messages.length < 2) return null
      const grpcStatusCode = numberAttr(attributes, "rpc.grpc.status_code")
      return {
        spanId: span.spanId,
        system,
        method: span.name,
        startNanos: span.tsNanos,
        durationNs: span.durationNs,
        grpcStatusCode,
        outcome: grpcOutcome(grpcStatusCode, span.statusCode),
        messages,
        truncated,
      } satisfies RpcStreamInfo
    })
    .filter((stream): stream is RpcStreamInfo => Boolean(stream))
}

export function messagingSummary(spans: readonly RpcTraceSpan[]): MessagingSummary | null {
  let producer = 0
  let consumer = 0
  let batchMax = 0
  let seenMessaging = false

  for (const span of spans) {
    const attributes = parseAttributes(span.attributes)
    if (!Object.keys(attributes).some((key) => key.startsWith("messaging."))) {
      continue
    }
    seenMessaging = true
    const kind = span.kind.replace("SPAN_KIND_", "")
    if (kind === "PRODUCER") producer += 1
    if (kind === "CONSUMER") consumer += 1
    batchMax = Math.max(batchMax, numberAttr(attributes, "messaging.batch.message_count") ?? 0)
  }

  return seenMessaging ? { producer, consumer, batchMax } : null
}

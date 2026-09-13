// R2 pipeline status: sampling policy + drop reasons + queue watermarks.
// Counts cross GraphQL as strings (Int saturation); the mapper keeps them
// exact and derives only display-safe numbers.

export type SamplingPolicyRow = {
  readonly signal: string
  readonly service: string | null
  readonly rule: string
  readonly rate: number
  readonly enforcedBy: string
  readonly description: string
}

export type IngestDropRow = {
  readonly signal: string | null
  readonly reason: string
  readonly count: string
  readonly detail: string
}

export type IngestQueueRow = {
  readonly signal: string
  readonly depth: number
  readonly capacity: number
  readonly highWater: number
  readonly accepted: string
}

export type PipelineStatus = {
  readonly policies: readonly SamplingPolicyRow[]
  readonly drops: readonly IngestDropRow[]
  readonly queues: readonly IngestQueueRow[]
}

export type QueuePressure = "ok" | "filling" | "full"

export function parseCount(value: string): bigint {
  try {
    const parsed = BigInt(value)
    return parsed < 0 ? 0n : parsed
  } catch {
    return 0n
  }
}

export function formatCount(value: string): string {
  const parsed = parseCount(value)
  if (parsed <= BigInt(Number.MAX_SAFE_INTEGER)) {
    return Number(parsed).toLocaleString("en-US")
  }
  return value
}

export function formatRate(rate: number): string {
  if (!Number.isFinite(rate)) return "—"
  return `${(rate * 100).toLocaleString("en-US", { maximumFractionDigits: 2 })}%`
}

export function queuePressure(queue: IngestQueueRow): QueuePressure {
  if (queue.capacity <= 0) return "ok"
  if (queue.depth >= queue.capacity) return "full"
  if (queue.depth * 4 >= queue.capacity * 3) return "filling"
  return "ok"
}

export function totalDropped(drops: readonly IngestDropRow[]): bigint {
  return drops.reduce((sum, row) => sum + parseCount(row.count), 0n)
}

export function totalAccepted(queues: readonly IngestQueueRow[]): bigint {
  return queues.reduce((sum, row) => sum + parseCount(row.accepted), 0n)
}

export function mapPipelineStatus(raw: {
  readonly samplingPolicy: ReadonlyArray<SamplingPolicyRow>
  readonly ingestDrops: ReadonlyArray<IngestDropRow>
  readonly ingestQueues: ReadonlyArray<IngestQueueRow>
}): PipelineStatus {
  return {
    policies: [...raw.samplingPolicy],
    drops: [...raw.ingestDrops],
    queues: [...raw.ingestQueues],
  }
}

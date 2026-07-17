// Load the fixed runtime metric strip via Plan-152 typed transport.

import {
  RuntimeMetricStripDocument,
  RuntimeMetricStripQuerySchema,
  type RuntimeMetricStripQuery,
  type RuntimeMetricStripQueryVariables,
} from "@/features/runtime-metrics/api/runtime-metrics.generated"
import {
  mapRuntimeMetricStrip,
  type StripPanel,
} from "@/features/runtime-metrics/api/runtime-metrics-mapper"
import { executeGraphqlOperation } from "@/platform/graphql/client"
import type { TypedDocumentNode } from "@/platform/graphql/typed-document"

export type LoadRuntimeMetricsInput = {
  readonly service?: string | undefined
  readonly invocationId?: string | undefined
  readonly fromNanos: string
  readonly toNanos: string
  readonly stepSeconds: number
  readonly signal?: AbortSignal | undefined
}

/**
 * Scope precedence matches the legacy strip: invocationId wins over service.
 * Uses raw (non-cached) transport — same as the pre-move MetricStrip path.
 */
export async function loadRuntimeMetricStrip(
  input: LoadRuntimeMetricsInput
): Promise<StripPanel[]> {
  const service = input.invocationId || !input.service ? null : (input.service ?? null)
  const invocationId = input.invocationId ?? null
  const variables: RuntimeMetricStripQueryVariables = {
    fromNanos: input.fromNanos,
    toNanos: input.toNanos,
    stepSeconds: input.stepSeconds,
    service,
    invocationId,
  }
  // Re-brand the generated DocumentNode for the platform client generics.
  const document = RuntimeMetricStripDocument as unknown as TypedDocumentNode<
    RuntimeMetricStripQuery,
    RuntimeMetricStripQueryVariables
  >
  const data = await executeGraphqlOperation<
    RuntimeMetricStripQuery,
    RuntimeMetricStripQueryVariables
  >(
    document,
    RuntimeMetricStripQuerySchema,
    variables,
    input.signal ? { signal: input.signal } : undefined
  )
  return mapRuntimeMetricStrip(data)
}

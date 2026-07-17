// Public facade for traces (Plan 142). Named exports only.

export {
  TracesPage,
  TraceTable,
  loadTraces,
  validateTracesSearch,
  patchTracesSearch,
  paramToTraceSort,
  traceSortToParam,
  traceDetailSearch,
} from "@/features/traces/components/traces-page"
export type { TracesSearch, TracesLoaderData } from "@/features/traces/components/traces-page"

export {
  TraceDetailPage,
  loadTraceDetail,
  validateTraceDetailSearch,
  TraceGraphqlSection,
  TraceRpcSection,
  ColorByPicker,
  TraceViewModeToggle,
  ClockSkewBanner,
  TraceCompareResult,
  LinkedTraceEdges,
  TraceErrorCallout,
  InspectorEventList,
  InspectorLinksList,
} from "@/features/traces/components/trace-detail-page"
export type {
  TraceDetailSearch,
  TraceDetailLoaderData,
  SpanEvent,
} from "@/features/traces/components/trace-detail-page"

export { TraceWaterfall, WHOLE_TRACE_ID } from "@/features/traces/components/trace-waterfall"
export type { TraceViewMode, WaterfallSpan } from "@/features/traces/components/trace-waterfall"
export { TraceFlamegraph } from "@/features/traces/components/trace-flamegraph"
export { FieldExplorer } from "@/features/traces/components/trace-field-explorer"
export { AttributeComparePanel } from "@/features/traces/components/trace-attribute-compare"
export { EvidenceGapsCard } from "@/features/traces/components/trace-evidence-gaps"
export { GraphqlOperationCard } from "@/features/traces/components/trace-graphql-operations"
export { RpcStreamCard } from "@/features/traces/components/trace-rpc-streams"
export {
  SpanKindChip,
  spanKindMeta,
  spanKindLabel,
} from "@/features/traces/components/trace-span-kind"

export {
  orderSpans,
  computeWindow,
  computeSelfTimes,
  detectSkew,
  packFlameLanes,
  TRACE_SKEW_THRESHOLD_MS,
} from "@/features/traces/model/trace-tree"
export type {
  TraceTreeSpan,
  OrderedTraceSpan,
  TraceWindow,
  SkewReport,
  SkewPair,
} from "@/features/traces/model/trace-tree"

export { buildGraphqlOperations } from "@/features/traces/model/graphql-operations"
export type {
  GraphqlOperation,
  GraphqlFieldNode,
  GraphqlTraceSpan,
} from "@/features/traces/model/graphql-operations"

export {
  buildRpcStreams,
  grpcStatusLabel,
  messagingSummary,
  parseGrpcStatusCode,
} from "@/features/traces/model/rpc-streams"
export type {
  RpcStreamInfo,
  RpcMessage,
  RpcTraceSpan,
  RpcTraceEvent,
  MessagingSummary,
} from "@/features/traces/model/rpc-streams"

export {
  SPAN_FIELDS,
  type AttributeCompareRow,
  type CriticalHop,
  type CriticalPath,
  type EvidenceGap,
  type FieldKey,
  type FieldStats,
  type FieldValueCount,
  type LiveSpan,
  type Span,
  type SpanLink,
  type TraceDiff,
  type TraceDiffChange,
  type TraceDiffSpan,
  type TraceSummary,
} from "@/features/traces/model/wire"
export {
  MINIMAP_EDGE_HANDLE_PX,
  minimapDragAction,
  minimapHitTest,
  minimapRecenterAction,
  type MinimapGesture,
  type MinimapHit,
} from "@/features/traces/model/timeline-minimap"
export {
  BAR_CLAMP_MAX_PCT,
  BAR_CLAMP_MIN_PCT,
  BAR_DURATION_LABEL_MIN_PX,
  BAR_NAME_LABEL_MIN_PX,
  DEFAULT_MAX_WINDOW_MS,
  DRAG_THRESHOLD_PX,
  MIN_VISIBLE_MS,
  ZOOM_FACTOR,
  ZOOM_TO_SPAN_PADDING,
  barLabelVisibility,
  barRect,
  clampViewport,
  initialTimelineState,
  msToPct,
  pxDeltaToMs,
  pxToMs,
  timelineReducer,
  type BarLabelVisibility,
  type BarRect,
  type TimelineAction,
  type TimelineState,
  type TimelineViewport,
} from "@/features/traces/model/timeline-viewport"
export {
  useTimelineInteractions,
  type MarqueeRange,
  type TimelineInteractionHandlers,
  type TimelineInteractions,
  type UseTimelineInteractionsOptions,
} from "@/features/traces/hooks/use-timeline-interactions"

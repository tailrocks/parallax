import { useCallback, useMemo, useRef, useState } from "react"

export interface ChartBrushPoint {
  tsNanos: string
}

interface ChartBrushOptions<T extends ChartBrushPoint> {
  series: readonly T[]
  stepSeconds: number
  disabled?: boolean
  onWindow: (fromNanos: string, toNanos: string) => void
  getReferenceValue?: (point: T) => string
}

export function indexFromState(state: unknown): number | null {
  const value = (state as { activeTooltipIndex?: unknown } | null)
    ?.activeTooltipIndex
  return typeof value === "number" ? value : null
}

export function bucketWindow(
  points: readonly ChartBrushPoint[],
  index: number,
  stepSeconds: number
) {
  const point = points[index]
  if (!point) return null
  return {
    fromNanos: point.tsNanos,
    toNanos: (
      BigInt(point.tsNanos) +
      BigInt(stepSeconds) * 1_000_000_000n
    ).toString(),
  }
}

export function dragWindow(
  points: readonly ChartBrushPoint[],
  start: number,
  end: number,
  stepSeconds: number
) {
  const low = Math.min(start, end)
  const high = Math.max(start, end)
  const from = points[low]
  const to = bucketWindow(points, high, stepSeconds)
  if (!from || !to) return null
  return { fromNanos: from.tsNanos, toNanos: to.toNanos }
}

const defaultReferenceValue = <T extends ChartBrushPoint>(point: T) =>
  point.tsNanos

export function useChartBrush<T extends ChartBrushPoint>({
  series,
  stepSeconds,
  disabled = false,
  onWindow,
  getReferenceValue,
}: ChartBrushOptions<T>) {
  const [dragStart, setDragStart] = useState<number | null>(null)
  const [dragEnd, setDragEnd] = useState<number | null>(null)
  const suppressClick = useRef(false)
  const referenceValue = getReferenceValue ?? defaultReferenceValue

  const reset = useCallback(() => {
    setDragStart(null)
    setDragEnd(null)
  }, [])

  const referenceRange = useMemo(() => {
    if (dragStart == null || dragEnd == null) return null
    const low = Math.min(dragStart, dragEnd)
    const high = Math.max(dragStart, dragEnd)
    const start = series[low]
    const end = series[high]
    if (!start || !end) return null
    return { x1: referenceValue(start), x2: referenceValue(end) }
  }, [dragEnd, dragStart, referenceValue, series])

  const onClick = useCallback(
    (state: unknown) => {
      if (disabled) return
      if (suppressClick.current) {
        suppressClick.current = false
        return
      }
      const index = indexFromState(state)
      if (index == null) return
      const window = bucketWindow(series, index, stepSeconds)
      if (window) onWindow(window.fromNanos, window.toNanos)
    },
    [disabled, onWindow, series, stepSeconds]
  )

  const onMouseDown = useCallback(
    (state: unknown) => {
      if (disabled) return
      suppressClick.current = false
      const index = indexFromState(state)
      setDragStart(index)
      setDragEnd(index)
    },
    [disabled]
  )

  const onMouseMove = useCallback(
    (state: unknown) => {
      if (disabled || dragStart == null) return
      setDragEnd(indexFromState(state))
    },
    [disabled, dragStart]
  )

  const onMouseUp = useCallback(() => {
    if (disabled || dragStart == null || dragEnd == null) {
      reset()
      return
    }
    if (dragStart !== dragEnd) {
      const window = dragWindow(series, dragStart, dragEnd, stepSeconds)
      suppressClick.current = true
      if (window) onWindow(window.fromNanos, window.toNanos)
    }
    reset()
  }, [disabled, dragEnd, dragStart, onWindow, reset, series, stepSeconds])

  return {
    dragStart,
    dragEnd,
    referenceRange,
    chartHandlers: {
      onClick,
      onMouseDown,
      onMouseMove,
      onMouseUp,
    },
    reset,
  }
}

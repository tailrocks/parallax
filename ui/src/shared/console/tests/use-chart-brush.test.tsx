/* @vitest-environment jsdom */

import { cleanup, fireEvent, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { useChartBrush } from "@/shared/console/use-chart-brush"

const points = [0, 30, 60, 90, 120, 150].map((seconds, index) => ({
  tsNanos: `${seconds * 1_000_000_000}`,
  label: `bucket-${index}`,
}))

afterEach(cleanup)

function Harness({ onWindow }: { onWindow: (fromNanos: string, toNanos: string) => void }) {
  const brush = useChartBrush({
    series: points,
    stepSeconds: 30,
    onWindow,
    getReferenceValue: (point) => point.label,
  })
  return (
    <div>
      <output data-testid="range">
        {brush.referenceRange ? `${brush.referenceRange.x1}:${brush.referenceRange.x2}` : "none"}
      </output>
      <button type="button" onClick={() => brush.chartHandlers.onClick({ activeTooltipIndex: 1 })}>
        click-1
      </button>
      <button
        type="button"
        onClick={() => brush.chartHandlers.onMouseDown({ activeTooltipIndex: 2 })}
      >
        start-2
      </button>
      <button
        type="button"
        onClick={() => brush.chartHandlers.onMouseDown({ activeTooltipIndex: 5 })}
      >
        start-5
      </button>
      <button
        type="button"
        onClick={() => brush.chartHandlers.onMouseMove({ activeTooltipIndex: 2 })}
      >
        move-2
      </button>
      <button
        type="button"
        onClick={() => brush.chartHandlers.onMouseMove({ activeTooltipIndex: 5 })}
      >
        move-5
      </button>
      <button type="button" onClick={() => brush.chartHandlers.onMouseUp()}>
        up
      </button>
    </div>
  )
}

describe("useChartBrush", () => {
  it("clicks a single bucket window", () => {
    const onWindow = vi.fn()
    render(<Harness onWindow={onWindow} />)

    fireEvent.click(screen.getByText("click-1"))

    expect(onWindow).toHaveBeenCalledWith("30000000000", "60000000000")
  })

  it("drags forward and exposes the reference range", () => {
    const onWindow = vi.fn()
    render(<Harness onWindow={onWindow} />)

    fireEvent.click(screen.getByText("start-2"))
    fireEvent.click(screen.getByText("move-5"))
    expect(screen.getByTestId("range").textContent).toBe("bucket-2:bucket-5")
    fireEvent.click(screen.getByText("up"))

    expect(onWindow).toHaveBeenCalledWith("60000000000", "180000000000")
  })

  it("normalizes reversed drags", () => {
    const onWindow = vi.fn()
    render(<Harness onWindow={onWindow} />)

    fireEvent.click(screen.getByText("start-5"))
    fireEvent.click(screen.getByText("move-2"))
    fireEvent.click(screen.getByText("up"))

    expect(onWindow).toHaveBeenCalledWith("60000000000", "180000000000")
  })
})

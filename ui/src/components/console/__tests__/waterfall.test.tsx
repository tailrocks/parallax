/* @vitest-environment jsdom */

import { fireEvent, render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"

import {
  TraceWaterfall,
  WHOLE_TRACE_ID,
} from "@/components/console/trace-waterfall"
import type { WaterfallSpan } from "@/components/console/trace-waterfall"

const spans: WaterfallSpan[] = [
  {
    spanId: "root",
    parentSpanId: null,
    tsNanos: "100",
    durationNs: "100",
    service: "api",
    name: "GET /checkout",
    kind: "SERVER",
    statusCode: "STATUS_CODE_UNSET",
    statusMessage: "",
  },
  {
    spanId: "child",
    parentSpanId: "root",
    tsNanos: "125",
    durationNs: "25",
    service: "payments",
    name: "POST /pay",
    kind: "CLIENT",
    statusCode: "STATUS_CODE_ERROR",
    statusMessage: "boom",
  },
]

describe("TraceWaterfall", () => {
  it("renders whole-trace and ordered span rows", () => {
    render(
      <TraceWaterfall
        spans={spans}
        selectedId={WHOLE_TRACE_ID}
        onSelect={vi.fn()}
      />
    )

    expect(screen.getByText("Whole trace")).toBeTruthy()
    expect(screen.getByText("GET /checkout")).toBeTruthy()
    expect(screen.getByText("POST /pay")).toBeTruthy()
    expect(screen.getByText("error")).toBeTruthy()
  })

  it("moves selection with j/k and arrow keys", () => {
    const onSelect = vi.fn()
    const { container, rerender } = render(
      <TraceWaterfall
        spans={spans}
        selectedId={WHOLE_TRACE_ID}
        onSelect={onSelect}
      />
    )
    const waterfall = container.querySelector("[tabindex='0']")
    expect(waterfall).toBeTruthy()

    fireEvent.keyDown(waterfall!, { key: "j" })
    expect(onSelect).toHaveBeenLastCalledWith("root")

    rerender(
      <TraceWaterfall spans={spans} selectedId="root" onSelect={onSelect} />
    )
    fireEvent.keyDown(waterfall!, { key: "ArrowDown" })
    expect(onSelect).toHaveBeenLastCalledWith("child")

    rerender(
      <TraceWaterfall spans={spans} selectedId="child" onSelect={onSelect} />
    )
    fireEvent.keyDown(waterfall!, { key: "k" })
    expect(onSelect).toHaveBeenLastCalledWith("root")
  })
})

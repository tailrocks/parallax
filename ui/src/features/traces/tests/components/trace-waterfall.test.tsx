/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import { TraceWaterfall, WHOLE_TRACE_ID } from "@/features/traces/components/trace-waterfall"
import type { WaterfallSpan } from "@/features/traces/components/trace-waterfall"

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

const modeSpans: WaterfallSpan[] = [
  spans[0]!,
  spans[1]!,
  {
    spanId: "cache",
    parentSpanId: "root",
    tsNanos: "160",
    durationNs: "15",
    service: "cache",
    name: "GET /cache",
    kind: "CLIENT",
    statusCode: "STATUS_CODE_UNSET",
    statusMessage: "",
  },
]

afterEach(cleanup)

describe("TraceWaterfall", () => {
  it("renders whole-trace and ordered span rows", () => {
    render(<TraceWaterfall spans={spans} selectedId={WHOLE_TRACE_ID} onSelect={vi.fn()} />)

    expect(screen.getByText("Whole trace")).toBeTruthy()
    expect(screen.getByText("GET /checkout")).toBeTruthy()
    expect(screen.getByText("POST /pay")).toBeTruthy()
    expect(screen.getByText("error")).toBeTruthy()
    expect(screen.getAllByTestId("trace-minimap-bar")).toHaveLength(2)
  })

  it("moves selection with j/k and arrow keys", async () => {
    const user = userEvent.setup()
    const onSelect = vi.fn()
    const { container, rerender } = render(
      <TraceWaterfall spans={spans} selectedId={WHOLE_TRACE_ID} onSelect={onSelect} />
    )
    const waterfall = container.querySelector("[tabindex='0']")
    expect(waterfall).toBeTruthy()
    if (!(waterfall instanceof HTMLElement)) {
      throw new Error("waterfall keyboard target is not an HTMLElement")
    }
    waterfall.focus()

    await user.keyboard("j")
    expect(onSelect).toHaveBeenLastCalledWith("root")

    rerender(<TraceWaterfall spans={spans} selectedId="root" onSelect={onSelect} />)
    await user.keyboard("{ArrowDown}")
    expect(onSelect).toHaveBeenLastCalledWith("child")

    rerender(<TraceWaterfall spans={spans} selectedId="child" onSelect={onSelect} />)
    await user.keyboard("k")
    expect(onSelect).toHaveBeenLastCalledWith("root")
  })
})

describe("TraceWaterfall modes", () => {
  it("highlights critical-path span rows", () => {
    render(
      <TraceWaterfall
        spans={spans}
        selectedId={WHOLE_TRACE_ID}
        onSelect={vi.fn()}
        highlightIds={new Set(["child"])}
      />
    )

    expect(screen.getByTestId("trace-row-child").className).toContain("border-primary")
    expect(screen.getByTestId("trace-row-root").className).not.toContain("border-primary")
  })

  it("shows only errors and ancestors in errors mode", () => {
    render(
      <TraceWaterfall
        spans={modeSpans}
        selectedId={WHOLE_TRACE_ID}
        onSelect={vi.fn()}
        mode="errors"
      />
    )

    expect(screen.getByText("GET /checkout")).toBeTruthy()
    expect(screen.getByText("POST /pay")).toBeTruthy()
    expect(screen.queryByText("GET /cache")).toBeNull()
    expect(screen.getAllByTestId("trace-minimap-bar")).toHaveLength(2)
  })

  it("falls back to the full tree when errors mode has no errors", () => {
    render(
      <TraceWaterfall
        spans={modeSpans.map((span) => ({
          ...span,
          statusCode: "STATUS_CODE_UNSET",
        }))}
        selectedId={WHOLE_TRACE_ID}
        onSelect={vi.fn()}
        mode="errors"
      />
    )

    expect(screen.getByText("No errored spans. Showing full trace.")).toBeTruthy()
    expect(screen.getByText("GET /cache")).toBeTruthy()
  })

  it("renders contiguous service lanes", () => {
    render(
      <TraceWaterfall
        spans={modeSpans}
        selectedId={WHOLE_TRACE_ID}
        onSelect={vi.fn()}
        mode="lanes"
      />
    )

    expect(screen.getAllByTestId("trace-lane-header")).toHaveLength(3)
  })

  it("selects spans from the minimap", async () => {
    const user = userEvent.setup()
    const onSelect = vi.fn()
    render(<TraceWaterfall spans={modeSpans} selectedId={WHOLE_TRACE_ID} onSelect={onSelect} />)

    await user.click(screen.getAllByTestId("trace-minimap-bar")[1]!)
    expect(onSelect).toHaveBeenCalledWith("child")
  })
})

describe("corpus regressions (plan 160)", () => {
  it("D-002 t-deep: span names render on one truncating line, never char-wrapped", () => {
    render(<TraceWaterfall spans={spans} selectedId={null} onSelect={() => {}} />)
    const name = screen.getByText("GET /checkout")
    expect(name.className).toContain("truncate")
    expect(name.className).not.toContain("break-words")
    expect(name.getAttribute("title")).toBe("GET /checkout")
  })

  it("t-orphan: a span whose parent never arrived is flagged detached", () => {
    render(
      <TraceWaterfall
        spans={[
          spans[0]!,
          {
            spanId: "lost",
            parentSpanId: "never-arrived",
            tsNanos: "150",
            durationNs: "10",
            service: "api",
            name: "orphan.detached_child",
            kind: "INTERNAL",
            statusCode: "STATUS_CODE_UNSET",
            statusMessage: "",
          },
        ]}
        selectedId={null}
        onSelect={() => {}}
      />
    )
    expect(screen.getByText("orphan.detached_child")).toBeTruthy()
    expect(screen.getByText("detached")).toBeTruthy()
  })

  it("true children are not flagged detached", () => {
    render(<TraceWaterfall spans={spans} selectedId={null} onSelect={() => {}} />)
    expect(screen.queryByText("detached")).toBeNull()
  })
})

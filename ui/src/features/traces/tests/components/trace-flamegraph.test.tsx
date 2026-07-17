/* @vitest-environment jsdom */

import { fireEvent, render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"

import { TraceFlamegraph } from "@/features/traces/components/trace-flamegraph"
import type { WaterfallSpan } from "@/features/traces/components/trace-waterfall"

const spans: WaterfallSpan[] = [
  {
    spanId: "root",
    parentSpanId: null,
    tsNanos: "0",
    durationNs: "100000000",
    service: "api",
    name: "request",
    kind: "SPAN_KIND_SERVER",
    statusCode: "STATUS_CODE_UNSET",
    statusMessage: "",
  },
  {
    spanId: "child",
    parentSpanId: "root",
    tsNanos: "10000000",
    durationNs: "50000000",
    service: "db",
    name: "select",
    kind: "SPAN_KIND_CLIENT",
    statusCode: "STATUS_CODE_UNSET",
    statusMessage: "",
  },
  {
    spanId: "other",
    parentSpanId: null,
    tsNanos: "70000000",
    durationNs: "20000000",
    service: "worker",
    name: "background",
    kind: "SPAN_KIND_INTERNAL",
    statusCode: "STATUS_CODE_UNSET",
    statusMessage: "",
  },
]

describe("TraceFlamegraph", () => {
  it("renders accessible span controls and selects one", () => {
    const onSelect = vi.fn()
    render(<TraceFlamegraph spans={spans} selectedId={null} onSelect={onSelect} />)

    fireEvent.click(screen.getByRole("button", { name: /select, db, 50ms/i }))
    expect(onSelect).toHaveBeenCalledWith("child")
  })

  it("focuses a subtree on shift-click and can restore the whole trace", () => {
    render(<TraceFlamegraph spans={spans} selectedId={null} onSelect={vi.fn()} />)

    fireEvent.click(screen.getByRole("button", { name: /request, api/i }), {
      shiftKey: true,
    })
    expect(screen.queryByRole("button", { name: /background, worker/i })).toBeNull()
    expect(screen.getByRole("button", { name: /select, db/i })).toBeDefined()

    fireEvent.click(screen.getByRole("button", { name: "Show whole trace" }))
    expect(screen.getByRole("button", { name: /background, worker/i })).toBeDefined()
  })

  it("explains an empty trace", () => {
    render(<TraceFlamegraph spans={[]} selectedId={null} onSelect={vi.fn()} />)
    expect(screen.getByText(/has not emitted any span data/i)).toBeDefined()
  })
})

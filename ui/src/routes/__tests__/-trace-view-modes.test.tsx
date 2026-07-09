/* @vitest-environment jsdom */

import { cleanup, fireEvent, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import {
  ClockSkewBanner,
  TraceViewModeToggle,
  validateTraceDetailSearch,
} from "@/routes/traces.$traceId"

afterEach(cleanup)

describe("trace detail view modes", () => {
  it("validates the trace view search param", () => {
    expect(validateTraceDetailSearch({ tab: "story", view: "lanes" })).toEqual({
      tab: "story",
      view: "lanes",
      range: undefined,
      from: undefined,
      to: undefined,
    })
    expect(validateTraceDetailSearch({ tab: "nope", view: "bad" })).toEqual({
      tab: undefined,
      view: undefined,
      range: undefined,
      from: undefined,
      to: undefined,
    })
    expect(
      validateTraceDetailSearch({ range: "custom", from: 1, to: "2" })
    ).toEqual({
      tab: undefined,
      view: undefined,
      range: "custom",
      from: "1",
      to: "2",
    })
  })

  it("dispatches view changes from the mode toggle", () => {
    const onChange = vi.fn()
    render(<TraceViewModeToggle value="tree" onChange={onChange} />)

    fireEvent.click(screen.getByRole("button", { name: /lanes view/i }))

    expect(onChange).toHaveBeenCalledWith("lanes")
  })

  it("renders skew warnings only for suspect pairs", () => {
    const empty = render(
      <ClockSkewBanner report={{ suspectPairs: [], maxDriftMs: 0 }} />
    )
    expect(empty.container.textContent).toBe("")
    empty.unmount()

    render(
      <ClockSkewBanner
        report={{
          suspectPairs: [{ parentId: "root", childId: "db", driftMs: 125 }],
          maxDriftMs: 125,
        }}
      />
    )

    expect(screen.getByText(/clock skew suspected/i)).toBeTruthy()
    expect(screen.getByText(/125 ms/i)).toBeTruthy()
  })
})

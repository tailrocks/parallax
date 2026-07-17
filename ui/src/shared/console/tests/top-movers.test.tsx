/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import {
  TopMovers,
  computeMovers,
  moverSentence,
} from "@/shared/console/top-movers"
import type { ServiceMoverInput } from "@/shared/console/top-movers"
import { renderTestRouter } from "@/test/router"
import type { ResolvedRange } from "@/lib/range"

const range: ResolvedRange = {
  key: "custom",
  fromNanos: "1000",
  toNanos: "2000",
}

afterEach(cleanup)

function renderWithRouter(component: React.ReactNode) {
  return renderTestRouter(component, {
    targetPaths: ["/services/$service"],
  })
}

describe("computeMovers", () => {
  it("applies threshold edges and ranking", () => {
    const now: ServiceMoverInput[] = [
      { name: "volume", spanCount: "200", errorCount: "0", p95Ms: 10 },
      { name: "latency", spanCount: "100", errorCount: "0", p95Ms: 150 },
      { name: "error", spanCount: "100", errorCount: "2", p95Ms: 10 },
      { name: "newbie", spanCount: "1", errorCount: "0", p95Ms: null },
    ]
    const previous: ServiceMoverInput[] = [
      { name: "volume", spanCount: "100", errorCount: "0", p95Ms: 10 },
      { name: "latency", spanCount: "100", errorCount: "0", p95Ms: 100 },
      { name: "error", spanCount: "100", errorCount: "0", p95Ms: 10 },
    ]

    expect(computeMovers(now, previous).map((mover) => mover.kind)).toEqual([
      "error",
      "latency",
      "new",
      "volume",
    ])
  })

  it("caps movers at six", () => {
    const now = Array.from({ length: 8 }, (_, index) => ({
      name: `svc-${index}`,
      spanCount: "100",
      errorCount: `${8 - index}`,
      p95Ms: 10,
    }))
    const previous = now.map((row) => ({ ...row, errorCount: "0" }))

    expect(computeMovers(now, previous)).toHaveLength(6)
  })

  it("builds deterministic sentences", () => {
    const [mover] = computeMovers(
      [{ name: "checkout", spanCount: "100", errorCount: "5", p95Ms: 10 }],
      [{ name: "checkout", spanCount: "100", errorCount: "1", p95Ms: 10 }]
    )

    expect(mover ? moverSentence(mover) : "").toBe(
      "checkout error rate 1.0% -> 5.0%"
    )
  })
})

describe("TopMovers", () => {
  it("renders mover links with the current window", async () => {
    renderWithRouter(
      <TopMovers
        range={range}
        now={[
          { name: "checkout", spanCount: "100", errorCount: "5", p95Ms: 10 },
        ]}
        previous={[
          { name: "checkout", spanCount: "100", errorCount: "1", p95Ms: 10 },
        ]}
      />
    )

    const link = await screen.findByRole("link", {
      name: /checkout error rate/i,
    })
    const href = link.getAttribute("href") ?? ""
    const params = new URLSearchParams(href.split("?")[1])
    expect(href.startsWith("/services/checkout?")).toBe(true)
    expect(params.get("range")).toBe("custom")
    expect(params.get("from")?.replaceAll('"', "")).toBe("1000")
    expect(params.get("to")?.replaceAll('"', "")).toBe("2000")
  })

  it("renders the empty state", () => {
    render(
      <TopMovers
        range={range}
        now={[
          { name: "checkout", spanCount: "100", errorCount: "1", p95Ms: 10 },
        ]}
        previous={[
          { name: "checkout", spanCount: "100", errorCount: "1", p95Ms: 10 },
        ]}
      />
    )

    expect(
      screen.getByText("Nothing moved more than the thresholds in this window.")
    ).toBeTruthy()
  })
})

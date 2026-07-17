/* @vitest-environment jsdom */

import { screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import type { StoryBeat } from "@/domain/story/story-beat"
import { StoryTimeline } from "@/features/story"
import { renderTestRouter } from "@/test/router"

const beats: StoryBeat[] = [
  {
    tsNanos: "1000000000",
    lane: "api",
    kind: "span.start",
    title: "checkout",
    traceId: "trace-1",
    spanId: "span-root",
    severity: null,
    durationNs: null,
  },
  {
    tsNanos: "2000000000",
    lane: "api",
    kind: "log",
    title: "INFO cache hit",
    traceId: "trace-1",
    spanId: null,
    severity: "INFO",
    durationNs: null,
  },
  {
    tsNanos: "3000000000",
    lane: "db",
    kind: "error",
    title: "SELECT orders error",
    traceId: "trace-1",
    spanId: "span-db",
    severity: "ERROR",
    durationNs: "20000000",
  },
]

describe("StoryTimeline", () => {
  it("renders time ordered lanes with linked error beats", async () => {
    renderTestRouter(<StoryTimeline beats={beats} />, {
      targetPaths: ["/traces/$traceId", "/logs"],
    })

    const rows = await screen.findAllByTestId("story-row")
    expect(rows.map((row) => row.textContent)).toEqual([
      expect.stringContaining("checkout"),
      expect.stringContaining("INFO cache hit"),
      expect.stringContaining("SELECT orders error"),
    ])
    expect(screen.getAllByText("api")).toHaveLength(2)
    expect(screen.getByText("db")).toBeTruthy()
    expect(rows[2]!.className).toContain("border-rose")
    expect(screen.getByText("SELECT orders error").closest("a")?.href).toContain("/traces/trace-1")
    expect(screen.getByText("INFO cache hit").closest("a")?.href).toContain("/logs")
  })
})

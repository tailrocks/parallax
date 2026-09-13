import { describe, expect, it } from "vitest"

import {
  PEAK_WINDOW_PAD_NS,
  chartAnnotationMarks,
  nearestChartTime,
  peakWindowFromSeries,
  tracesAroundPeakSearch,
} from "@/features/runtime-metrics/model/peak-window"

describe("peakWindowFromSeries", () => {
  it("pads a traces window around the highest sample", () => {
    const peak = peakWindowFromSeries([
      {
        groupValue: null,
        points: [
          { tsNanos: "100000000000", value: 1 },
          { tsNanos: "200000000000", value: 9 },
          { tsNanos: "300000000000", value: 2 },
        ],
      },
    ])
    expect(peak).toEqual({
      peakNanos: "200000000000",
      peakValue: 9,
      fromNanos: (200000000000n - PEAK_WINDOW_PAD_NS).toString(),
      toNanos: (200000000000n + PEAK_WINDOW_PAD_NS).toString(),
    })
    expect(tracesAroundPeakSearch(peak!, "checkout")).toEqual({
      range: "custom",
      from: peak!.fromNanos,
      to: peak!.toNanos,
      service: "checkout",
    })
  })

  it("returns null when there are no points", () => {
    expect(peakWindowFromSeries([{ groupValue: null, points: [] }])).toBeNull()
    expect(peakWindowFromSeries([])).toBeNull()
  })

  it("clamps the window start at zero", () => {
    const peak = peakWindowFromSeries(
      [{ groupValue: "a", points: [{ tsNanos: "10", value: 4 }] }],
      100n
    )
    expect(peak).toEqual({
      peakNanos: "10",
      peakValue: 4,
      fromNanos: "0",
      toNanos: "110",
    })
    expect(tracesAroundPeakSearch(peak!)).toEqual({
      range: "custom",
      from: "0",
      to: "110",
    })
  })
})

describe("chartAnnotationMarks", () => {
  it("snaps release markers onto the nearest chart time category", () => {
    const rows = [
      { time: "10:00:00", tsNanos: "100" },
      { time: "10:01:00", tsNanos: "200" },
      { time: "10:02:00", tsNanos: "300" },
    ]
    expect(nearestChartTime(rows, "210")).toBe("10:01:00")
    expect(
      chartAnnotationMarks(rows, [
        { tsNanos: "210", kind: "release", title: "v2", service: "checkout" },
      ])
    ).toEqual([
      {
        key: "checkout:v2:210",
        time: "10:01:00",
        title: "v2",
        service: "checkout",
        kind: "release",
        tsNanos: "210",
      },
    ])
  })

  it("drops annotations that cannot snap to a row", () => {
    expect(
      chartAnnotationMarks([], [{ tsNanos: "1", kind: "release", title: "v1", service: "a" }])
    ).toEqual([])
  })
})

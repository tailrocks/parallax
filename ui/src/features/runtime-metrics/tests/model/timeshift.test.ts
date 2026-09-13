import { describe, expect, it } from "vitest"

import {
  buildCompareRows,
  compareGroupKey,
  previousWindowShiftSeconds,
  shiftSeriesForward,
} from "@/features/runtime-metrics/model/timeshift"

describe("previousWindowShiftSeconds", () => {
  it("returns the whole-second window length", () => {
    expect(
      previousWindowShiftSeconds({ key: "custom", fromNanos: "0", toNanos: "3600000000000" })
    ).toBe(3600)
  })

  it("floors sub-second remainders and clamps to at least one second", () => {
    expect(
      previousWindowShiftSeconds({ key: "custom", fromNanos: "0", toNanos: "1500000000" })
    ).toBe(1)
    expect(previousWindowShiftSeconds({ key: "custom", fromNanos: "100", toNanos: "100" })).toBe(1)
  })

  it("falls back to one second on unparseable ranges", () => {
    expect(previousWindowShiftSeconds({ key: "custom", fromNanos: "nope", toNanos: "100" })).toBe(1)
  })
})

describe("shiftSeriesForward", () => {
  it("slides stamps forward by the shift and names null groups", () => {
    const shifted = shiftSeriesForward(
      [
        {
          groupValue: null,
          points: [
            { tsNanos: "1000000000", value: 1 },
            { tsNanos: "not-a-number", value: 2 },
          ],
        },
      ],
      60_000_000_000n
    )
    expect(shifted).toEqual([
      {
        groupValue: "series-1",
        points: [{ tsNanos: "61000000000", value: 1 }],
      },
    ])
    expect(compareGroupKey("series-1")).toBe("series-1 (previous)")
  })
})

describe("buildCompareRows", () => {
  const series = [
    {
      groupValue: null,
      points: [
        { tsNanos: "60000000000", value: 1 },
        { tsNanos: "120000000000", value: 2 },
        { tsNanos: "180000000000", value: 3 },
      ],
    },
  ]

  it("keeps the current-series tail contract when compare is off", () => {
    const rows = buildCompareRows(series, [], 3600_000_000_000n)
    expect(rows).toHaveLength(3)
    // Solid line covers all but the newest bucket; the tail covers the last two.
    expect(rows[0]).toMatchObject({ "series-1": 1 })
    expect(rows[0]).not.toHaveProperty("series-1__tail")
    expect(rows[1]).toMatchObject({ "series-1": 2, "series-1__tail": 2 })
    expect(rows[2]).toMatchObject({ "series-1__tail": 3 })
    expect(rows[2]).not.toHaveProperty("series-1")
    expect(rows.map((row) => row["tsNanos"])).toEqual([
      "60000000000",
      "120000000000",
      "180000000000",
    ])
  })

  it("overlays previous-window points on the current axis", () => {
    // Current window sits one hour ahead of the previous window's stamps.
    const current = [
      {
        groupValue: null,
        points: [
          { tsNanos: "3660000000000", value: 1 },
          { tsNanos: "3720000000000", value: 2 },
          { tsNanos: "3780000000000", value: 3 },
        ],
      },
    ]
    const previous = [
      {
        groupValue: null,
        points: [
          { tsNanos: "60000000000", value: 10 },
          { tsNanos: "120000000000", value: 20 },
        ],
      },
    ]
    const rows = buildCompareRows(current, previous, 3600_000_000_000n)
    expect(rows).toHaveLength(3)
    expect(rows[0]).toMatchObject({ "series-1": 1, "series-1 (previous)": 10 })
    expect(rows[1]).toMatchObject({ "series-1": 2, "series-1 (previous)": 20 })
    expect(rows[2]).not.toHaveProperty("series-1 (previous)")
  })

  it("adds rows for previous points that land between current buckets", () => {
    const previous = [{ groupValue: "eu", points: [{ tsNanos: "90000000000", value: 7 }] }]
    const rows = buildCompareRows(series, previous, 0n)
    expect(rows).toHaveLength(4)
    const between = rows.find((row) => row["tsNanos"] === "90000000000")
    expect(between).toMatchObject({ "eu (previous)": 7 })
    expect(between).not.toHaveProperty("series-1")
  })
})

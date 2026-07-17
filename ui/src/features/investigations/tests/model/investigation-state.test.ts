import { describe, expect, it } from "vitest"

import {
  appendInvestigationPin,
  buildInvestigationPin,
  emptyInvestigationState,
  hrefForPin,
  parseInvestigationState,
  serializeInvestigationState,
  windowFromHref,
} from "@/features/investigations/model/investigation-state"

describe("investigation state helpers", () => {
  it("serializes the current route as a restorable pin", () => {
    const pin = buildInvestigationPin(
      "trace",
      "Checkout root",
      "/traces/abc?view=errors&range=24h"
    )

    expect(pin).toEqual({
      kind: "trace",
      ref: "/traces/abc?view=errors&range=24h",
      label: "Checkout root",
      note: "",
    })
    expect(hrefForPin(pin)).toBe("/traces/abc?view=errors&range=24h")
  })

  it("captures window params from hrefs", () => {
    expect(windowFromHref("/traces/abc?range=1h")).toEqual({ range: "1h" })
    expect(windowFromHref("/issues/fp?range=custom&from=10&to=20")).toEqual({
      range: "custom",
      from: "10",
      to: "20",
    })
  })

  it("parses and caps investigation state", () => {
    const state = emptyInvestigationState()
    const withPin = appendInvestigationPin(
      state,
      buildInvestigationPin("issue", "panic", "/issues/fp")
    )
    const parsed = parseInvestigationState(
      serializeInvestigationState({
        ...withPin,
        pins: [
          ...withPin.pins,
          { kind: "bad", ref: "/bad", label: "bad" } as never,
        ],
        notes: "plain text",
      })
    )

    expect(parsed.version).toBe(1)
    expect(parsed.notes).toBe("plain text")
    expect(parsed.pins).toHaveLength(1)
    expect(parsed.pins[0]?.label).toBe("panic")
  })
})

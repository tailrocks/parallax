/* @vitest-environment jsdom */

import { describe, expect, it } from "vitest"

import {
  diagnosticMismatch,
  expectDiagnostic,
} from "../../src/test/diagnostics"

describe("test diagnostic policy", () => {
  it("accepts exact ordered diagnostics", () => {
    const diagnostics = [
      { level: "warn" as const, message: "expected warning" },
      { level: "error" as const, message: "expected error" },
    ]
    expect(diagnosticMismatch(diagnostics, diagnostics)).toBeNull()
  })

  it("rejects unexpected, missing, reordered, and substring-only matches", () => {
    const expected = [{ level: "warn" as const, message: "exact warning" }]
    expect(diagnosticMismatch(expected, [])).toContain(
      "runtime diagnostics differ"
    )
    expect(
      diagnosticMismatch([], [{ level: "error", message: "unexpected" }])
    ).toContain("runtime diagnostics differ")
    expect(
      diagnosticMismatch(expected, [
        { level: "warn", message: "prefix exact warning suffix" },
      ])
    ).toContain("runtime diagnostics differ")
    expect(
      diagnosticMismatch(
        [...expected, { level: "error", message: "second" }],
        [{ level: "error", message: "second" }, ...expected]
      )
    ).toContain("runtime diagnostics differ")
  })

  it("owns page errors exactly", () => {
    expectDiagnostic("error", "page error Error: page exploded")
    window.dispatchEvent(
      new ErrorEvent("error", { error: new Error("page exploded") })
    )
  })

  it("owns browser rejections exactly", () => {
    expectDiagnostic(
      "error",
      "browser unhandled rejection Error: promise exploded"
    )
    const event = new Event("unhandledrejection", { cancelable: true })
    Object.defineProperty(event, "reason", {
      value: new Error("promise exploded"),
    })
    window.dispatchEvent(event)
  })
})

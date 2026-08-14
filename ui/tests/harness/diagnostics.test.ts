/* @vitest-environment jsdom */

import { describe, expect, it } from "vitest"

import { isBrowserEngineNoise, isUnusedModulepreloadWarning } from "../e2e/fixtures/diagnostics"
import { diagnosticMismatch, expectDiagnostic } from "../../src/test/diagnostics"

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
    expect(diagnosticMismatch(expected, [])).toContain("runtime diagnostics differ")
    expect(diagnosticMismatch([], [{ level: "error", message: "unexpected" }])).toContain(
      "runtime diagnostics differ"
    )
    expect(
      diagnosticMismatch(expected, [{ level: "warn", message: "prefix exact warning suffix" }])
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
    expect(
      window.dispatchEvent(
        new ErrorEvent("error", {
          cancelable: true,
          error: new Error("page exploded"),
        })
      )
    ).toBe(false)
  })

  it("treats WebKit unused modulepreload as engine noise", () => {
    const webkit =
      "The resource http://127.0.0.1:4174/assets/routes-N2TyLTuY.js was preloaded using link preload but not used within a few seconds from the window's load event. Please make sure it wasn't preloaded for nothing."
    expect(isUnusedModulepreloadWarning(webkit)).toBe(true)
    expect(isBrowserEngineNoise({ kind: "console-warning", message: webkit })).toBe(true)
    expect(isUnusedModulepreloadWarning("React key warning on list row")).toBe(false)
    expect(
      isBrowserEngineNoise({ kind: "console-warning", message: "React key warning on list row" })
    ).toBe(false)
    expect(isBrowserEngineNoise({ kind: "pageerror", message: webkit })).toBe(false)
  })

  it("owns browser rejections exactly", () => {
    expectDiagnostic("error", "browser unhandled rejection Error: promise exploded")
    const event = new Event("unhandledrejection", { cancelable: true })
    Object.defineProperty(event, "reason", {
      value: new Error("promise exploded"),
    })
    expect(window.dispatchEvent(event)).toBe(false)
  })
})

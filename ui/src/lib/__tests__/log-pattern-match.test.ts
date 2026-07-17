import { describe, expect, it } from "vitest"

import {
  bodyMatchesTemplate,
  filterBodiesByTemplate,
  templateStableFragment,
  templateToRegExp,
} from "@/lib/log-pattern-match"

describe("bodyMatchesTemplate", () => {
  const template = "checkout authorize user=<*> duration=<*>"

  it("matches parameter churn on the same skeleton", () => {
    expect(
      bodyMatchesTemplate("checkout authorize user=u-111 duration=12ms", template)
    ).toBe(true)
    expect(
      bodyMatchesTemplate("checkout authorize user=u-999 duration=1ms", template)
    ).toBe(true)
  })

  it("rejects a different stable skeleton", () => {
    expect(
      bodyMatchesTemplate("inventory reserve sku=widget qty=3", template)
    ).toBe(false)
  })

  it("requires full-line alignment", () => {
    expect(
      bodyMatchesTemplate("prefix checkout authorize user=x duration=1", template)
    ).toBe(false)
  })
})

describe("templateStableFragment", () => {
  it("returns longest contiguous stable run", () => {
    expect(templateStableFragment("a <*> b c <*> d")).toBe("b c")
    expect(templateStableFragment("<*> <*>")).toBeNull()
    expect(templateStableFragment("checkout authorize <*>")).toBe(
      "checkout authorize"
    )
  })
})

describe("filterBodiesByTemplate", () => {
  it("keeps only matching bodies in order", () => {
    const template = "svc handler-<*> done"
    const bodies = [
      "svc handler-alpha done",
      "other line",
      "svc handler-bravo done",
    ]
    expect(filterBodiesByTemplate(bodies, template)).toEqual([
      "svc handler-alpha done",
      "svc handler-bravo done",
    ])
  })
})

describe("templateToRegExp", () => {
  it("escapes regex metacharacters in stable tokens", () => {
    const re = templateToRegExp("a+b <*> end")
    expect(re.test("a+b foo end")).toBe(true)
    expect(re.test("axb foo end")).toBe(false)
  })
})

import { describe, expect, it } from "vitest"

import { gqlString } from "@/lib/api"

describe("gqlString", () => {
  it("escapes backslash, quote, newline, and tab", () => {
    expect(gqlString('a\\b"c\nd\te')).toBe('a\\\\b\\"c\\nd\\te')
  })

  it("strips carriage returns", () => {
    expect(gqlString("a\rb")).toBe("ab")
  })

  it("escapes form-feed as a unicode escape", () => {
    expect(gqlString("a\u000cb")).toBe("a\\u000cb")
  })

  it("escapes NUL as a unicode escape", () => {
    expect(gqlString("a\u0000b")).toBe("a\\u0000b")
  })

  it("leaves plain ASCII unchanged", () => {
    expect(gqlString("hello world")).toBe("hello world")
  })
})

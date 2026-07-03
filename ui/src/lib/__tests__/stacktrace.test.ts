import { describe, expect, it } from "vitest"

import { parseStacktrace, structuredFrameCount } from "@/lib/stacktrace"

describe("parseStacktrace", () => {
  it("parses Rust frames", () => {
    const frames = parseStacktrace(
      "0: checkout::cart::total\n   at src/cart.rs:99:5"
    )

    expect(frames[0]).toMatchObject({
      fn: "checkout::cart::total",
      file: "src/cart.rs",
      line: 99,
      col: 5,
      isApp: true,
    })
  })

  it("parses compact Rust-style frames", () => {
    const frames = parseStacktrace("checkout::cart::total at src/cart.rs:99")

    expect(frames[0]).toMatchObject({
      fn: "checkout::cart::total",
      file: "src/cart.rs",
      line: 99,
    })
  })

  it("parses Python frames", () => {
    const frames = parseStacktrace(
      'File "/app/checkout.py", line 42, in total\nraise RuntimeError()'
    )

    expect(frames[0]).toMatchObject({
      fn: "total",
      file: "/app/checkout.py",
      line: 42,
      isApp: true,
    })
  })

  it("parses Node/V8 frames and marks libraries", () => {
    const frames = parseStacktrace(
      "at handler (/srv/app.js:10:2)\nat next (/srv/node_modules/lib/index.js:1:1)"
    )

    expect(frames[0]).toMatchObject({
      fn: "handler",
      file: "/srv/app.js",
      line: 10,
      col: 2,
      isApp: true,
    })
    expect(frames[1]?.isApp).toBe(false)
  })

  it("parses Go frames", () => {
    const frames = parseStacktrace("main.checkout()\n\t/app/main.go:17 +0x20")

    expect(frames[0]).toMatchObject({
      fn: "main.checkout",
      file: "/app/main.go",
      line: 17,
      isApp: true,
    })
  })

  it("parses Java frames", () => {
    const frames = parseStacktrace(
      "at com.tailrocks.Checkout.total(Checkout.java:77)"
    )

    expect(frames[0]).toMatchObject({
      fn: "com.tailrocks.Checkout.total",
      file: "Checkout.java",
      line: 77,
    })
  })

  it("keeps garbage as raw passthrough", () => {
    const frames = parseStacktrace("not a stack frame")

    expect(frames).toEqual([{ raw: "not a stack frame" }])
    expect(structuredFrameCount(frames)).toBe(0)
  })
})

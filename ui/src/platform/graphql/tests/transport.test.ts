/* @vitest-environment jsdom */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { clearGraphqlCache, gqlString, graphql, graphqlCached } from "@/platform/graphql/transport"
import { createAppQueryClient } from "@/platform/query/client"
import { installBrowserQueryClient } from "@/platform/query/graphql-query"

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

describe("graphqlCached (TanStack Query)", () => {
  const query = `{ hello }`

  beforeEach(() => {
    const client = createAppQueryClient()
    installBrowserQueryClient(client)
    clearGraphqlCache()
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json({ data: { hello: "world" } }))
    )
  })

  afterEach(() => {
    clearGraphqlCache()
    vi.unstubAllGlobals()
  })

  it("dedupes concurrent identical queries into one fetch", async () => {
    const [a, b] = await Promise.all([
      graphqlCached<{ hello: string }>(query),
      graphqlCached<{ hello: string }>(query),
    ])
    expect(a).toEqual({ hello: "world" })
    expect(b).toEqual({ hello: "world" })
    expect(fetch).toHaveBeenCalledTimes(1)
  })

  it("serves a second call from Query cache within staleTime", async () => {
    await graphqlCached(query)
    await graphqlCached(query)
    expect(fetch).toHaveBeenCalledTimes(1)
  })

  it("refetches after invalidate", async () => {
    await graphqlCached(query)
    clearGraphqlCache()
    await graphqlCached(query)
    expect(fetch).toHaveBeenCalledTimes(2)
  })

  it("raw graphql always hits the network", async () => {
    await graphql(query)
    await graphql(query)
    expect(fetch).toHaveBeenCalledTimes(2)
  })

  it("is available in the browser (client-side cache active)", () => {
    expect(typeof window).not.toBe("undefined")
  })
})

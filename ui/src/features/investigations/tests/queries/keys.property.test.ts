// Plan-103 bounded property suite: TanStack Query key-factory identity
// (docs/research/testing/property-invariants.md). Same inputs must produce
// deeply equal keys; distinct ids must never collide; every key extends its
// scope prefix so invalidation stays hierarchical.

import fc from "fast-check"
import { describe, expect, it } from "vitest"

import { investigationKeys } from "@/features/investigations/queries/keys"

describe("investigation query key properties", () => {
  it("is deterministic and prefix-hierarchical for arbitrary ids", () => {
    fc.assert(
      fc.property(fc.string({ maxLength: 32 }), (id) => {
        expect(investigationKeys.detail(id)).toEqual(investigationKeys.detail(id))
        const detail = investigationKeys.detail(id)
        const prefix = investigationKeys.details()
        expect(detail.slice(0, prefix.length)).toEqual([...prefix])
        expect(detail.slice(0, investigationKeys.all.length)).toEqual([...investigationKeys.all])
      }),
      { numRuns: 200 }
    )
  })

  it("distinct ids never collide", () => {
    fc.assert(
      fc.property(fc.string({ maxLength: 32 }), fc.string({ maxLength: 32 }), (a, b) => {
        fc.pre(a !== b)
        expect(investigationKeys.detail(a)).not.toEqual(investigationKeys.detail(b))
      }),
      { numRuns: 200 }
    )
  })
})

import { describe, expect, it } from "vitest"

import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import { decodeSearchValue } from "@/platform/url/decode-search-value"

const numberDecoder: RuntimeDecoder<number> = {
  safeParse(input) {
    return typeof input === "number" && Number.isFinite(input)
      ? { success: true, data: input }
      : { success: false, error: "bad" }
  },
}

describe("decodeSearchValue", () => {
  it("returns decoded value on success", () => {
    expect(decodeSearchValue(3, numberDecoder)).toEqual({
      ok: true,
      value: 3,
    })
  })

  it("rejects primitives/objects/arrays that fail the feature schema", () => {
    for (const input of ["x", null, {}, [], true]) {
      const result = decodeSearchValue(input, numberDecoder)
      expect(result.ok).toBe(false)
    }
  })
})

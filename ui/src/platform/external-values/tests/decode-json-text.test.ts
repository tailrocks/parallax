import { afterEach, describe, expect, it } from "vitest"

import {
  formatBoundaryDiagnostic,
  setBoundaryDiagnosticSink,
} from "@/platform/external-values/boundary-diagnostic"
import type { BoundaryError } from "@/platform/external-values/boundary-error"
import { decodeJsonText } from "@/platform/external-values/decode-json-text"
import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"

const stringDecoder: RuntimeDecoder<string> = {
  safeParse(input) {
    return typeof input === "string"
      ? { success: true, data: input }
      : { success: false, error: "not-string" }
  },
}

const throwingDecoder: RuntimeDecoder<string> = {
  safeParse() {
    throw new Error("decoder-boom SECRET_TOKEN")
  },
}

afterEach(() => {
  setBoundaryDiagnosticSink(null)
})

describe("decodeJsonText", () => {
  it("decodes a valid JSON string through one decoder", () => {
    expect(decodeJsonText(JSON.stringify("hello"), stringDecoder)).toEqual({
      ok: true,
      value: "hello",
    })
  })

  it("rejects non-string input kinds without throwing", () => {
    const inputs: unknown[] = [
      undefined,
      null,
      1,
      true,
      {},
      [],
      () => undefined,
    ]
    const codes = inputs.map((input) => {
      const result = decodeJsonText(input, stringDecoder)
      return result.ok ? "ok" : result.error.code
    })
    expect(codes).toEqual(Array(inputs.length).fill("invalid-type"))
  })

  it("rejects invalid JSON", () => {
    expect(decodeJsonText("{not-json", stringDecoder)).toMatchObject({
      ok: false,
      error: { code: "invalid-json" },
    })
  })

  it("rejects schema failures", () => {
    expect(decodeJsonText("123", stringDecoder)).toMatchObject({
      ok: false,
      error: { code: "schema-rejected" },
    })
  })

  it("defends against throwing decoders", () => {
    const result = decodeJsonText('"x"', throwingDecoder)
    expect(result).toMatchObject({
      ok: false,
      error: { code: "schema-rejected" },
    })
    const error = (result as { ok: false; error: BoundaryError }).error
    const rendered = formatBoundaryDiagnostic(error)
    expect(rendered).not.toContain("SECRET_TOKEN")
    expect(rendered).not.toContain("decoder-boom")
  })

  it("never puts payload sentinels into diagnostics", () => {
    const reports: string[] = []
    setBoundaryDiagnosticSink({
      report(error) {
        reports.push(formatBoundaryDiagnostic(error))
      },
    })
    decodeJsonText('{"token":"super-secret-value-xyz"}', stringDecoder)
    expect(reports.join("\n")).not.toContain("super-secret-value-xyz")
    expect(reports.join("\n")).not.toContain("token")
  })
})

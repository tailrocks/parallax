import { describe, expect, it } from "vitest"

import { guessId } from "@/lib/quick-jump"

describe("guessId", () => {
  it("routes 32-hex OTel trace ids directly to traces", () => {
    expect(guessId("53e97e432cbb9280841b90ca56c4e4c4")).toEqual([
      { kind: "trace", id: "53e97e432cbb9280841b90ca56c4e4c4" },
    ])
  })

  it("keeps 16-hex ids ambiguous across run, fingerprint, and span", () => {
    // Source shapes: parallax-cli new_run_id() hex-encodes nanos,
    // fingerprint.rs returns 16 hex, and OTel span ids are 16 hex.
    expect(guessId("A7A77B573B7261A1")).toEqual([
      { kind: "invocation", id: "a7a77b573b7261a1" },
      { kind: "fingerprint", id: "a7a77b573b7261a1" },
      { kind: "span-in-trace", id: "a7a77b573b7261a1" },
    ])
  })

  it("recognizes real-shaped non-hex run ids from fixtures and playground", () => {
    expect(guessId(" run-a ")).toEqual([{ kind: "invocation", id: "run-a" }])
    expect(guessId("run_cli")).toEqual([{ kind: "invocation", id: "run_cli" }])
    expect(guessId("plan065-live-1783552201")).toEqual([
      { kind: "invocation", id: "plan065-live-1783552201" },
    ])
  })

  it("does not route arbitrary words", () => {
    expect(guessId("checkout")).toEqual([])
  })
})

/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import { RpcStreamCard } from "@/features/traces/components/trace-rpc-streams"
import type { RpcStreamInfo } from "@/features/traces/model/rpc-streams"

afterEach(cleanup)

const stream: RpcStreamInfo = {
  spanId: "stream",
  system: "grpc",
  method: "QuoteService/StreamQuotes",
  startNanos: "100",
  durationNs: "1000",
  grpcStatusCode: 4,
  outcome: "deadline_exceeded",
  truncated: true,
  messages: [
    { id: 1, type: "SENT", timeUnixNano: "100", size: 80 },
    { id: 2, type: "RECEIVED", timeUnixNano: "500", size: 120 },
    { id: 3, type: "SENT", timeUnixNano: "900", size: null },
  ],
}

describe("RpcStreamCard", () => {
  it("renders message dots outcome badge and truncated line", () => {
    render(<RpcStreamCard streams={[stream]} />)

    expect(screen.getByText("grpc")).toBeTruthy()
    expect(screen.getByText("QuoteService/StreamQuotes")).toBeTruthy()
    expect(screen.getByText("deadline")).toBeTruthy()
    expect(screen.getAllByTestId("rpc-message-dot")).toHaveLength(3)
    expect(screen.getByText("showing first 3 messages")).toBeTruthy()
  })

  it("renders nothing for empty streams", () => {
    const { container } = render(<RpcStreamCard streams={[]} />)

    expect(container.textContent).toBe("")
  })
})

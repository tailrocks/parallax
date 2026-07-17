import { describe, expect, it } from "vitest"

import {
  buildRpcStreams,
  grpcStatusLabel,
  messagingSummary,
} from "@/features/traces/model/rpc-streams"
import type {
  RpcTraceEvent,
  RpcTraceSpan,
} from "@/features/traces/model/rpc-streams"

function span(
  spanId: string,
  attributes: Record<string, unknown>,
  overrides: Partial<RpcTraceSpan> = {}
): RpcTraceSpan {
  return {
    spanId,
    parentSpanId: null,
    tsNanos: "100",
    durationNs: "1000",
    service: "checkout",
    name: "QuoteService/StreamQuotes",
    kind: "SPAN_KIND_CLIENT",
    statusCode: "STATUS_CODE_UNSET",
    attributes: JSON.stringify(attributes),
    ...overrides,
  }
}

function event(
  spanId: string,
  timeUnixNano: string,
  attributes: Record<string, unknown>,
  name = "rpc.message"
): RpcTraceEvent {
  return {
    spanId,
    spanName: "QuoteService/StreamQuotes",
    service: "checkout",
    name,
    timeUnixNano,
    attributes: JSON.stringify(attributes),
  }
}

describe("rpc trace builder", () => {
  it("builds ordered message timelines for multi-message RPC spans", () => {
    const streams = buildRpcStreams(
      [span("stream", { "rpc.system": "grpc", "rpc.grpc.status_code": 0 })],
      [
        event("stream", "150", {
          "rpc.message.type": "RECEIVED",
          "rpc.message.id": 2,
          "rpc.message.compressed_size": 120,
        }),
        event("stream", "110", {
          "rpc.message.type": "SENT",
          "rpc.message.id": 1,
          "rpc.message.compressed_size": 80,
        }),
        event("stream", "130", {
          "rpc.message.type": "SENT",
          "rpc.message.id": 3,
        }),
        event("stream", "170", { "rpc.message.type": "RECEIVED" }, "message"),
        event("stream", "190", { "rpc.message.type": "SENT" }),
      ]
    )

    expect(streams).toHaveLength(1)
    expect(streams[0]).toMatchObject({
      spanId: "stream",
      system: "grpc",
      outcome: "ok",
      truncated: false,
    })
    expect(streams[0]!.messages.map((message) => message.type)).toEqual([
      "SENT",
      "SENT",
      "RECEIVED",
      "RECEIVED",
      "SENT",
    ])
    expect(streams[0]!.messages[0]).toMatchObject({
      id: 1,
      size: 80,
    })
  })

  it("maps deadline outcomes and labels", () => {
    const streams = buildRpcStreams(
      [
        span(
          "deadline",
          { "rpc.system": "grpc", "rpc.grpc.status_code": "4" },
          { statusCode: "STATUS_CODE_ERROR" }
        ),
      ],
      [
        event("deadline", "110", { "rpc.message.type": "SENT" }),
        event("deadline", "120", { "rpc.message.type": "RECEIVED" }),
      ],
      true
    )

    expect(streams[0]).toMatchObject({
      outcome: "deadline_exceeded",
      truncated: true,
      grpcStatusCode: 4,
    })
    expect(grpcStatusLabel(4)).toBe("DEADLINE_EXCEEDED (gRPC 4)")
    expect(grpcStatusLabel(1)).toBe("CANCELLED (gRPC 1)")
  })

  it("excludes unary and non-rpc spans", () => {
    const streams = buildRpcStreams(
      [
        span("unary", { "rpc.system": "grpc" }),
        span("http", { "http.route": "/quotes" }),
      ],
      [
        event("unary", "110", { "rpc.message.type": "SENT" }),
        event("http", "120", { "rpc.message.type": "SENT" }),
      ]
    )

    expect(streams).toEqual([])
  })

  it("summarizes messaging producers consumers and max batch size", () => {
    expect(
      messagingSummary([
        span(
          "producer",
          {
            "messaging.system": "kafka",
            "messaging.batch.message_count": "12",
          },
          { kind: "SPAN_KIND_PRODUCER" }
        ),
        span(
          "consumer",
          {
            "messaging.system": "kafka",
            "messaging.batch.message_count": 5,
          },
          { kind: "SPAN_KIND_CONSUMER" }
        ),
      ])
    ).toEqual({ producer: 1, consumer: 1, batchMax: 12 })
    expect(messagingSummary([span("http", {})])).toBeNull()
  })
})

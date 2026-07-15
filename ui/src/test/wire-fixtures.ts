export type GraphqlFixtureKind = "data" | "empty" | "error" | "malformed"
export type SseFixtureKind = "data" | "malformed" | "reconnect" | "complete"

const GRAPHQL_FIXTURES = {
  data: { data: { services: ["checkout"] } },
  empty: { data: { services: [] } },
  error: {
    data: null,
    errors: [{ extensions: { code: "INTERNAL" }, message: "query failed" }],
  },
  malformed: { data: { services: "not-an-array" } },
} as const

const SSE_FIXTURES = {
  data: { data: '{"items":[{"traceId":"trace-1"}]}', event: "message" },
  malformed: { data: "{not-json", event: "message" },
  reconnect: { data: "", event: "error" },
  complete: { data: "", event: "complete" },
} as const

export function graphqlWireFixture(kind: GraphqlFixtureKind): unknown {
  return structuredClone(GRAPHQL_FIXTURES[kind])
}

export function sseWireFixture(kind: SseFixtureKind): unknown {
  return structuredClone(SSE_FIXTURES[kind])
}

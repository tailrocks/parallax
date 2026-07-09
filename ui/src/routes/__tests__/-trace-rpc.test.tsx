/* @vitest-environment jsdom */

import { cleanup, fireEvent, render, screen } from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import { afterEach, describe, expect, it } from "vitest"

import type { SpanLink } from "@/lib/api"
import type { RpcStreamInfo } from "@/lib/rpc-trace"
import {
  InspectorEventList,
  InspectorLinksList,
  TraceErrorCallout,
  TraceRpcSection,
} from "@/routes/traces.$traceId"
import type { SpanEvent } from "@/routes/traces.$traceId"

afterEach(cleanup)

function renderWithRouter(component: React.ReactNode) {
  window.scrollTo = () => {}
  const rootRoute = createRootRoute({ component: Outlet })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => component,
  })
  const traceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "traces/$traceId",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, traceRoute]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  })
  return render(<RouterProvider router={router} />)
}

function event(index: number): SpanEvent {
  return {
    name: "rpc.message",
    timeUnixNano: String(1_000_000_000 + index),
    attributes: { "rpc.message.id": index },
  }
}

function link(index: number): SpanLink {
  return {
    traceId: `trace-${index}`,
    spanId: `span-${index}`,
    attributes: "{}",
  }
}

const stream: RpcStreamInfo = {
  spanId: "stream",
  system: "grpc",
  method: "QuoteService/StreamQuotes",
  startNanos: "100",
  durationNs: "1000",
  grpcStatusCode: 0,
  outcome: "ok",
  truncated: false,
  messages: [
    { id: 1, type: "SENT", timeUnixNano: "150", size: null },
    { id: 2, type: "RECEIVED", timeUnixNano: "250", size: null },
  ],
}

describe("trace RPC inspector helpers", () => {
  it("caps inspector events and expands on demand", () => {
    render(
      <InspectorEventList
        events={Array.from({ length: 60 }, (_, index) => event(index))}
      />
    )

    expect(screen.getAllByTestId("inspector-event")).toHaveLength(25)
    fireEvent.click(screen.getByRole("button", { name: /show all 60 events/i }))
    expect(screen.getAllByTestId("inspector-event")).toHaveLength(60)
  })

  it("caps inspector links and expands on demand", async () => {
    renderWithRouter(
      <InspectorLinksList
        links={Array.from({ length: 60 }, (_, index) => link(index))}
        linkedTraceById={new Map()}
        rangeSearch={{ range: "24h" }}
      />
    )

    expect(await screen.findAllByTestId("trace-link-edge")).toHaveLength(25)
    fireEvent.click(screen.getByRole("button", { name: /show all 60 links/i }))
    expect(screen.getAllByTestId("trace-link-edge")).toHaveLength(60)
  })

  it("surfaces deadline and cancel grpc status labels", () => {
    const { rerender } = render(
      <TraceErrorCallout statusMessage="" grpcStatusCode="4" />
    )
    expect(screen.getByText("DEADLINE_EXCEEDED (gRPC 4)")).toBeTruthy()

    rerender(<TraceErrorCallout statusMessage="" grpcStatusCode="1" />)
    expect(screen.getByText("CANCELLED (gRPC 1)")).toBeTruthy()
  })

  it("renders the RPC card only for stream data", () => {
    const empty = render(<TraceRpcSection streams={[]} />)
    expect(empty.container.textContent).toBe("")
    empty.unmount()

    render(<TraceRpcSection streams={[stream]} />)
    expect(screen.getByText("RPC streams")).toBeTruthy()
    expect(screen.getByText("QuoteService/StreamQuotes")).toBeTruthy()
  })
})

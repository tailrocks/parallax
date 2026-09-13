/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import {
  InvocationLink,
  IssueLink,
  LogsLink,
  MetricLink,
  ServiceLink,
  TraceLink,
} from "@/shared/console/entity-links"
import { customRange } from "@/domain/time-range/range"
import { renderTestRouter } from "@/test/router"

afterEach(cleanup)

const range = customRange("1000", "2000")

function renderLink(node: React.ReactNode) {
  return renderTestRouter(node, {
    componentPaths: ["/"],
    targetPaths: [
      "/traces/$traceId",
      "/issues/$service/$fingerprint",
      "/services/$service",
      "/invocations/$invocationId",
      "/metrics/$metricName",
      "/logs",
    ],
  })
}

describe("entity links", () => {
  it("carries range on trace, issue, service, and metric links", async () => {
    renderLink(
      <>
        <TraceLink traceId="abc123" range={range} />
        <IssueLink service="checkout" fingerprint="fp9" range={range}>
          issue
        </IssueLink>
        <ServiceLink service="checkout" range={range} />
        <MetricLink metricName="http.server.duration" range={range} />
      </>
    )
    await screen.findByLabelText("trace abc123")
    for (const raw of [
      (screen.getByLabelText("trace abc123") as HTMLAnchorElement).href,
      (screen.getByText("issue") as HTMLAnchorElement).href,
      (screen.getByText("checkout") as HTMLAnchorElement).href,
      (screen.getByText("http.server.duration") as HTMLAnchorElement).href,
    ]) {
      const href = decodeURIComponent(raw)
      expect(href).toContain("range=custom")
      expect(href).toContain('from="1000"')
      expect(href).toContain('to="2000"')
    }
    expect((screen.getByLabelText("trace abc123") as HTMLAnchorElement).href).toContain(
      "/traces/abc123"
    )
  })

  it("shortens trace ids and invocation ids", async () => {
    renderLink(
      <>
        <TraceLink traceId="abcdef123456" range={range} short />
        <InvocationLink invocationId="invocation-42" range={range} />
      </>
    )
    expect(await screen.findByLabelText("trace abcdef123456").then((el) => el.textContent)).toBe(
      "abcdef12"
    )
    expect(screen.getByLabelText("invocation invocation-42").textContent).toBe("invocati")
  })

  it("merges logs context over the range", async () => {
    renderLink(
      <LogsLink
        range={range}
        service="checkout"
        q="panic"
        trace="abc123"
        anchor="1500"
      >
        surrounding logs
      </LogsLink>
    )
    const href = decodeURIComponent(
      ((await screen.findByText("surrounding logs")) as HTMLAnchorElement).href
    )
    expect(href).toContain("/logs?")
    expect(href).toContain("service=checkout")
    expect(href).toContain("q=panic")
    expect(href).toContain("trace=abc123")
    expect(href).toContain('anchor="1500"')
    expect(href).toContain("range=custom")
  })
})

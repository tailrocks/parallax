/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import { InvocationErrorsTab } from "@/features/invocations/components/invocation-errors-tab"
import { InvocationTracesTab } from "@/features/invocations/components/invocation-traces-tab"
import { JobsCyclesTab } from "@/features/invocations/components/jobs-cycles-tab"
import { SessionsTab } from "@/features/invocations/components/sessions-tab"
import { renderTestRouter } from "@/test/router"

const range = { key: "custom", fromNanos: "0", toNanos: "9000000000" }

afterEach(cleanup)

function renderTab(component: React.ReactNode) {
  return renderTestRouter(component, {
    targetPaths: ["/traces/$traceId", "/issues/$fingerprint"],
  })
}

describe("JobsCyclesTab", () => {
  it("renders the explicit empty state", async () => {
    renderTab(<JobsCyclesTab cycles={[]} jobs={[]} />)
    expect(await screen.findByText("No background work")).toBeTruthy()
  })

  it("renders cycle aggregates and job attempt chips with trace links", async () => {
    renderTab(
      <JobsCyclesTab
        cycles={[
          {
            name: "sync.remotes",
            count: 2,
            errorCount: 1,
            p50Ms: 900,
            p95Ms: 1400,
            lastNanos: "1000",
            lastTraceId: "trace-c",
          },
        ]}
        jobs={[
          {
            jobId: "job-1",
            jobType: "index.rebuild",
            producedNanos: "500",
            attempts: [
              {
                startNanos: "600",
                durationMs: 12,
                outcome: "success",
                hasError: false,
                traceId: "trace-j",
              },
            ],
            lastTraceId: "trace-j",
          },
        ]}
      />
    )
    expect(await screen.findByText("sync.remotes")).toBeTruthy()
    expect(screen.getByText("index.rebuild")).toBeTruthy()
    expect(
      screen.getAllByText(
        (_, element) => element?.textContent?.trim() === "#1 success"
      ).length
    ).toBeGreaterThan(0)
    const links = screen.getAllByRole("link")
    expect(
      links.some((link) => link.getAttribute("href") === "/traces/trace-j")
    ).toBe(true)
  })
})

describe("SessionsTab", () => {
  it("renders the explicit empty state for a one-shot invocation", async () => {
    renderTab(
      <SessionsTab
        sessions={[]}
        visits={[]}
        actions={[]}
        conversations={[]}
        errors={[]}
        agentSession={null}
      />
    )
    expect(await screen.findByText("No interactive sessions")).toBeTruthy()
  })

  it("renders sessions, visits, actions, and conversations", async () => {
    renderTab(
      <SessionsTab
        sessions={[
          {
            sessionId: "s1",
            previousSessionId: null,
            startNanos: "100",
            endNanos: null,
          },
        ]}
        visits={[
          {
            screenId: "home",
            visitId: "v1",
            sessionId: "s1",
            navigationSequence: 1,
            transitionReason: null,
            enteredNanos: "200",
            exitedNanos: null,
          },
        ]}
        actions={[
          {
            name: "submit",
            screenId: "home",
            widgetName: null,
            sessionId: "s1",
            traceId: "trace-a",
            startNanos: "300",
            durationMs: 4,
            outcome: "success",
            hasError: false,
          },
        ]}
        conversations={[
          {
            conversationId: "c1",
            agentName: "navigator",
            providerName: "anthropic",
            firstNanos: "100",
            lastNanos: "400",
            spanCount: 2,
            inputTokens: 10,
            outputTokens: 3,
          },
        ]}
        errors={[]}
        agentSession={null}
      />
    )
    expect(await screen.findByText("Screen visits")).toBeTruthy()
    expect(screen.getByText("UI actions")).toBeTruthy()
    expect(screen.getByText("Journey")).toBeTruthy()
    expect(screen.getByText("Agent conversations")).toBeTruthy()
    expect(screen.getByText("navigator")).toBeTruthy()
  })
})

describe("InvocationTracesTab", () => {
  it("renders the explicit empty state", async () => {
    renderTab(
      <InvocationTracesTab
        traces={[]}
        liveSpans={[]}
        live={false}
        range={range}
      />
    )
    expect(await screen.findByText("No traces")).toBeTruthy()
  })
})

describe("InvocationErrorsTab", () => {
  it("renders the explicit empty state", async () => {
    renderTab(<InvocationErrorsTab issues={[]} range={range} />)
    expect(await screen.findByText("No errors")).toBeTruthy()
  })
})

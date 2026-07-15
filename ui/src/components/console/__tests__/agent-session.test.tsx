/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import { AgentSessionCard } from "@/components/console/agent-session"
import type { AgentSessionData } from "@/components/console/agent-session"
import { renderTestRouter } from "@/test/router"

afterEach(cleanup)

function textContentIs(text: string) {
  return (_content: string, element: Element | null) =>
    element?.textContent === text
}

const session: AgentSessionData = {
  rootSpanId: "root",
  totalInputTokens: "12",
  totalOutputTokens: "5",
  errorCount: 1,
  truncated: false,
  steps: [
    {
      spanId: "root",
      traceId: "trace-agent",
      kind: "INVOKE_AGENT",
      name: "invoke_agent",
      startNanos: "1000000000",
      durationNs: "100000000",
      isError: false,
      genAiOperation: "invoke_agent",
      inputTokens: null,
      outputTokens: null,
    },
    {
      spanId: "tool",
      traceId: "trace-agent",
      kind: "EXECUTE_TOOL",
      name: "inspect_repo",
      startNanos: "1100000000",
      durationNs: "25000000",
      isError: false,
      genAiOperation: "execute_tool",
      inputTokens: "12",
      outputTokens: null,
    },
    {
      spanId: "shell",
      traceId: "trace-agent",
      kind: "SHELL",
      name: "false",
      startNanos: "1200000000",
      durationNs: "25000000",
      isError: true,
      genAiOperation: "execute_tool",
      inputTokens: null,
      outputTokens: "5",
    },
  ],
}

describe("AgentSessionCard", () => {
  it("renders step timeline with trace links and token totals", async () => {
    renderTestRouter(<AgentSessionCard session={session} />, {
      targetPaths: ["/traces/$traceId"],
    })

    expect(await screen.findByText("Agent session")).toBeTruthy()
    expect(screen.getAllByTestId("agent-step")).toHaveLength(3)
    expect(screen.getByText("inspect_repo").closest("a")?.href).toContain(
      "/traces/trace-agent"
    )
    expect(screen.getByText("false").closest("li")?.className).toContain(
      "border-rose"
    )
    expect(screen.getByText(textContentIs("12 in"))).toBeTruthy()
    expect(screen.getByText(textContentIs("5 out"))).toBeTruthy()
  })

  it("hides token totals when both totals are zero", async () => {
    renderTestRouter(
      <AgentSessionCard
        session={{ ...session, totalInputTokens: "0", totalOutputTokens: "0" }}
      />,
      { targetPaths: ["/traces/$traceId"] }
    )

    expect(await screen.findByText("Agent session")).toBeTruthy()
    expect(screen.queryByText(textContentIs("0 in"))).toBeNull()
    expect(screen.queryByText(textContentIs("0 out"))).toBeNull()
    expect(screen.getByText(textContentIs("3 steps"))).toBeTruthy()
  })
})

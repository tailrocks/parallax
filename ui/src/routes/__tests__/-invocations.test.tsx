/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import {
  InvocationsContent,
  filterInvocationsByRange,
} from "@/routes/invocations.index"
import { errorTypeBreakdown } from "@/components/console/invocations/invocation-errors-tab"
import { mergeLiveTraces } from "@/components/console/invocations/invocation-traces-tab"
import { mergeInvocations } from "@/lib/invocation"
import { renderTestRouter } from "@/test/router"

const NOW_MS = 1_720_000_000_000
const NOW_NS = BigInt(NOW_MS) * 1_000_000n

function nanos(offsetSeconds: number): string {
  return (NOW_NS + BigInt(offsetSeconds) * 1_000_000_000n).toString()
}

const range = {
  key: "custom",
  fromNanos: nanos(-3_600),
  toNanos: nanos(0),
}

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

const rows = mergeInvocations(
  [
    {
      invocationId: "inv-cli",
      registration: "cli" as const,
      command: "jk workspace sync",
      appMode: "interactive",
      outcome: "success",
      status: "finished",
      exitCode: 0,
      startedAtNanos: nanos(-120),
      endedAtNanos: nanos(-60),
      errorCount: 2,
      traceCount: 4,
      sessionCount: 1,
    },
  ],
  [
    {
      invocationId: "inv-observed",
      service: "wrapped-tool",
      lastCommand: null,
      appMode: null,
      firstNanos: nanos(-90),
      lastNanos: nanos(-1),
      spanCount: 7,
      logCount: 2,
    },
  ]
)

function renderList(
  overrides: Partial<Parameters<typeof InvocationsContent>[0]> = {}
) {
  const onSearch = vi.fn()
  const onOpen = vi.fn()
  const onRefresh = vi.fn()
  renderTestRouter(
    <InvocationsContent
      rows={rows}
      search={{}}
      range={range}
      live={false}
      onSearch={onSearch}
      onRefresh={onRefresh}
      onOpen={onOpen}
      {...overrides}
    />,
    {
      targetPaths: ["/invocations/$invocationId", "/invocations"],
    }
  )
  return { onSearch, onOpen, onRefresh }
}

describe("InvocationsContent", () => {
  it("renders merged rows with mode, status, outcome, and links", async () => {
    renderList()
    expect(await screen.findByText("jk workspace sync")).toBeTruthy()
    expect(screen.getByText("interactive")).toBeTruthy()
    expect(screen.getByText("success")).toBeTruthy()
    expect(screen.getByText("finished")).toBeTruthy()
    // The observed fixture is hours old relative to real now — stale.
    expect(screen.getByText("stale")).toBeTruthy()
    const link = screen
      .getAllByRole("link")
      .find((anchor) => anchor.getAttribute("href")?.includes("inv-cli"))
    expect(link?.getAttribute("href")).toContain("/invocations/inv-cli")
  })

  it("toggles live mode through the search params", async () => {
    const user = userEvent.setup()
    const { onSearch } = renderList()
    await user.click(await screen.findByRole("button", { name: /go live/i }))
    expect(onSearch).toHaveBeenCalledWith({ live: true })
  })

  it("shows the empty state when nothing matches", async () => {
    renderList({ rows: [] })
    expect(await screen.findByText("No CLI invocations yet")).toBeTruthy()
  })
})

describe("filterInvocationsByRange", () => {
  it("keeps rows overlapping the window and open-ended running rows", () => {
    const filtered = filterInvocationsByRange(rows, {
      key: "custom",
      fromNanos: nanos(-30),
      toNanos: nanos(0),
    })
    expect(filtered.map((row) => row.invocationId)).toEqual(["inv-observed"])
  })
})

describe("errorTypeBreakdown", () => {
  it("sums event counts per stable error type, sorted", () => {
    const breakdown = errorTypeBreakdown([
      {
        fingerprint: "a",
        title: "x",
        status: "open",
        eventCount: 2,
        errorType: "io::Timeout",
      },
      {
        fingerprint: "b",
        title: "y",
        status: "open",
        eventCount: 5,
        errorType: "jk::AttachFailed",
      },
      { fingerprint: "c", title: "z", status: "open", eventCount: 1 },
    ])
    expect(breakdown).toEqual([
      { errorType: "jk::AttachFailed", count: 5 },
      { errorType: "io::Timeout", count: 2 },
      { errorType: "(untyped)", count: 1 },
    ])
  })
})

describe("mergeLiveTraces", () => {
  it("prepends unseen live traces and aggregates spans per trace", () => {
    const merged = mergeLiveTraces(
      [
        {
          traceId: "t-loaded",
          rootName: "existing",
          service: "cli",
          startNanos: nanos(-10),
          durationNs: "1000",
          spanCount: 3,
          hasError: false,
        },
      ],
      [
        {
          tsNanos: nanos(-2),
          service: "cli",
          traceId: "t-live",
          spanId: "s1",
          parentSpanId: null,
          name: "live.root",
          kind: "SPAN_KIND_INTERNAL",
          statusCode: "STATUS_CODE_ERROR",
          durationNs: "500",
          invocationId: "inv-cli",
          sessionId: null,
        },
        {
          tsNanos: nanos(-1),
          service: "cli",
          traceId: "t-live",
          spanId: "s2",
          parentSpanId: "s1",
          name: "live.child",
          kind: "SPAN_KIND_INTERNAL",
          statusCode: "STATUS_CODE_UNSET",
          durationNs: "100",
          invocationId: "inv-cli",
          sessionId: null,
        },
      ]
    )
    expect(merged.map((trace) => trace.traceId)).toEqual(["t-live", "t-loaded"])
    expect(merged[0]!.spanCount).toBe(2)
    expect(merged[0]!.hasError).toBe(true)
  })
})

/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import { afterEach, describe, expect, it, vi } from "vitest"

import {
  RunsContent,
  durationNs,
  filterRunsByRange,
  mergeRuns,
  statusTone,
} from "@/routes/runs.index"
import type { RunsData } from "@/routes/runs.index"
import { graphqlCached } from "@/lib/api"
import {
  RunDetailContent,
  loadRunDetail,
  snapshotFromNanos,
} from "@/routes/runs.$runId"

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal()
  return {
    ...(actual as object),
    graphql: vi.fn(async () => ({ bundle: { markdown: "# bundle" } })),
    graphqlCached: vi.fn(),
  }
})

const range = {
  key: "custom",
  fromNanos: "1500000000",
  toNanos: "4000000000",
}

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

const merged = mergeRuns(
  [
    {
      runId: "run-a",
      command: "cargo test",
      status: "finished",
      exitCode: 1,
      startedAtNanos: "1000000000",
      endedAtNanos: "3000000000",
      errorCount: 2,
      traceCount: 4,
    },
  ],
  [
    {
      runId: "run-b",
      service: "worker",
      firstNanos: "2000000000",
      lastNanos: "5000000000",
      spanCount: 5,
      logCount: 6,
    },
  ]
)

const data: RunsData = { rows: merged }

function renderWithRouter(component: React.ReactNode, path = "/runs") {
  window.scrollTo = () => {}
  window.matchMedia = () =>
    ({
      matches: false,
      media: "",
      onchange: null,
      addListener() {},
      removeListener() {},
      addEventListener() {},
      removeEventListener() {},
      dispatchEvent: () => true,
    }) as MediaQueryList

  const rootRoute = createRootRoute({ component: Outlet })
  const runsRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/runs",
    component: () => component,
  })
  const runRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/runs/$runId",
    component: () => component,
  })
  const issueRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/issues/$fingerprint",
    component: () => null,
  })
  const traceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/traces/$traceId",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([
      runsRoute,
      runRoute,
      issueRoute,
      traceRoute,
    ]),
    history: createMemoryHistory({ initialEntries: [path] }),
  })

  return render(<RouterProvider router={router} />)
}

describe("Runs route", () => {
  it("merges registered and observed rows with duration/error fallbacks", () => {
    expect(merged).toHaveLength(2)
    expect(merged.find((row) => row.runId === "run-a")?.errorCount).toBe(2)
    expect(durationNs(merged.find((row) => row.runId === "run-a")!)).toBe(
      "2000000000"
    )
    expect(statusTone(merged.find((row) => row.runId === "run-a")!)).toBe(
      "rose"
    )
    expect(merged.find((row) => row.runId === "run-b")?.errorCount).toBeNull()
  })

  it("filters rows by activity window overlap", () => {
    const rows = [
      {
        ...merged[0]!,
        runId: "inside",
        startedAtNanos: "2000000000",
        endedAtNanos: "3000000000",
        lastNanos: "3000000000",
      },
      {
        ...merged[0]!,
        runId: "before",
        startedAtNanos: "1000000000",
        endedAtNanos: "1200000000",
        lastNanos: "1200000000",
      },
      {
        ...merged[0]!,
        runId: "running",
        status: "running" as const,
        startedAtNanos: "3500000000",
        endedAtNanos: null,
        lastNanos: "3500000000",
      },
    ]

    expect(
      filterRunsByRange(rows, range, "5000000000").map((row) => row.runId)
    ).toEqual(["inside", "running"])
  })

  it("renders list error and duration columns", async () => {
    renderWithRouter(
      <RunsContent
        data={data}
        search={{}}
        range={range}
        onSearch={() => {}}
        onRun={() => {}}
      />
    )

    expect(await screen.findByText("Errors")).toBeTruthy()
    expect(screen.getByText("Traces")).toBeTruthy()
    expect(screen.getByText("Duration")).toBeTruthy()
    expect(
      screen.getByRole("link", { name: "run-a" }).getAttribute("href")
    ).toBe("/runs/run-a?range=custom&from=%221500000000%22&to=%224000000000%22")
    expect(screen.getByRole("link", { name: "4" }).getAttribute("href")).toBe(
      "/runs/run-a?range=custom&from=%221500000000%22&to=%224000000000%22"
    )
    expect(screen.getByRole("link", { name: "2" }).getAttribute("href")).toBe(
      "/runs/run-a?range=custom&from=%221500000000%22&to=%224000000000%22"
    )
    expect(
      screen.getAllByText(
        (_content, element) => element?.textContent === "exit 1"
      ).length
    ).toBeGreaterThan(0)
    expect(screen.getByText("2.00s")).toBeTruthy()
  })

  it("renders detail stat row and download action", async () => {
    renderWithRouter(
      <RunDetailContent
        runId="run-a"
        run={{
          runId: "run-a",
          command: "cargo test",
          status: "finished",
          exitCode: 1,
          startedAtNanos: "1000000000",
          endedAtNanos: "3000000000",
          errorCount: 2,
          traceCount: 4,
          issues: [
            {
              fingerprint: "panic-a",
              title: "checkout total overflowed",
              status: "open",
              eventCount: 2,
              errorType: "panic",
            },
          ],
        }}
        traces={[
          {
            traceId: "trace-a",
            rootName: "POST /checkout",
            service: "checkout",
            startNanos: "2000000000",
            durationNs: "25000000",
            spanCount: 4,
            hasError: true,
          },
        ]}
        logs={[]}
        bundle={{ markdown: "# bundle" }}
        runtimeSnapshot={[]}
        range={range}
        live={false}
        liveLogs={[]}
        liveSpans={[]}
        onLive={() => {}}
      />,
      "/runs/run-a"
    )

    expect(await screen.findByText("Status")).toBeTruthy()
    expect(screen.getAllByText("Traces").length).toBeGreaterThan(0)
    expect(
      screen
        .getByRole("link", { name: /panic: checkout total overflowed/i })
        .getAttribute("href")
    ).toBe(
      "/issues/panic-a?range=custom&from=%221500000000%22&to=%224000000000%22"
    )
    expect(
      screen
        .getByRole("link", { name: /post \/checkout/i })
        .getAttribute("href")
    ).toBe(
      "/traces/trace-a?range=custom&from=%221500000000%22&to=%224000000000%22"
    )
    expect(screen.queryByText("Agent session")).toBeNull()
    expect(
      screen.getAllByRole("button", { name: /download/i }).length
    ).toBeGreaterThan(0)
  })


  it("bounds runtimeSnapshot fromNanos to the run start and omits bundle", async () => {
    expect(snapshotFromNanos("12345")).toBe("12345")
    expect(snapshotFromNanos(null, 1_000_000)).toBe(
      (BigInt(1_000_000) * 1_000_000n - 86_400_000_000_000n).toString()
    )

    vi.mocked(graphqlCached)
      .mockResolvedValueOnce({
        run: {
          runId: "run-a",
          command: "cargo test",
          status: "finished",
          exitCode: 0,
          startedAtNanos: "5000000000",
          endedAtNanos: "6000000000",
          errorCount: 0,
          traceCount: 1,
          issues: [],
        },
      })
      .mockResolvedValueOnce({
        tracesByRun: [],
        logsByRun: [],
        story: [],
        runtimeSnapshot: [],
        agentSession: null,
      })

    await loadRunDetail("run-a", 10_000)
    expect(vi.mocked(graphqlCached)).toHaveBeenCalledTimes(2)
    const secondQuery = String(vi.mocked(graphqlCached).mock.calls[1]?.[0])
    expect(secondQuery).toContain('fromNanos: "5000000000"')
    expect(secondQuery).not.toContain('fromNanos: "0"')
    expect(secondQuery).not.toContain("bundle")
  })
})
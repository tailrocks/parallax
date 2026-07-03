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
import { afterEach, describe, expect, it } from "vitest"

import {
  RunsContent,
  durationNs,
  mergeRuns,
  statusTone,
} from "@/routes/runs.index"
import type { RunsData } from "@/routes/runs.index"
import { RunDetailContent } from "@/routes/runs.$runId"

afterEach(cleanup)

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

  it("renders list error and duration columns", async () => {
    renderWithRouter(
      <RunsContent
        data={data}
        search={{}}
        onSearch={() => {}}
        onRun={() => {}}
      />
    )

    expect(await screen.findByText("Errors")).toBeTruthy()
    expect(screen.getByText("Duration")).toBeTruthy()
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
          issues: [],
        }}
        traces={[]}
        logs={[]}
        bundle={{ markdown: "# bundle" }}
        live={false}
        liveLogs={[]}
        liveSpans={[]}
        onLive={() => {}}
      />,
      "/runs/run-a"
    )

    expect(await screen.findByText("Status")).toBeTruthy()
    expect(screen.getByText("Traces")).toBeTruthy()
    expect(
      screen.getAllByRole("button", { name: /download/i }).length
    ).toBeGreaterThan(0)
  })
})

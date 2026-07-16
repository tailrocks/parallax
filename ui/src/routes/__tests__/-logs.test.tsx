/* @vitest-environment jsdom */

import {
  act,
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { defaultParseSearch } from "@tanstack/react-router"
import { afterEach, describe, expect, it, vi } from "vitest"
import { useState } from "react"

import {
  LogsTable,
  parseLogColumns,
  serializeLogColumns,
  severityVariant,
} from "@/components/logs-table"
import type { LogDoc } from "@/components/logs-table"
import { bucketWindow, dragWindow } from "@/components/console/use-chart-brush"
import { formatDateTime } from "@/lib/format"
import { customRange } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import {
  ColumnMenu,
  SavedViewsMenu,
  contextWindow,
  parseSavedViewState,
} from "@/routes/logs"
import { renderTestRouter } from "@/test/router"

const range: ResolvedRange = {
  key: "7d",
  fromNanos: "1000000000",
  toNanos: "604801000000000",
}
const custom = customRange("1500000000", "4000000000")

function parseHref(href: string) {
  const url = new URL(href, "http://example.test")
  return { search: defaultParseSearch(url.search), url }
}

const log: LogDoc = {
  tsNanos: "2000000000",
  eventName: "checkout.completed",
  observedTsNanos: "5000000000",
  service: "checkout",
  severityNum: 17,
  severityText: "ERROR",
  body: "checkout failed",
  traceId: "trace-a",
  spanId: "span-a",
  invocationId: "run-a",
  scopeName: "seed",
  attributes: '{"error":"boom"}',
  resource: '{"service.name":"checkout"}',
}

afterEach(cleanup)

function renderWithRouter(component: React.ReactNode) {
  return renderTestRouter(component, {
    targetPaths: ["/traces/$traceId", "/runs/$runId"],
  })
}

function renderLogsHost(
  initialLogs: LogDoc[],
  props: Pick<
    React.ComponentProps<typeof LogsTable>,
    "anchorNanos" | "onShowContext"
  > = {}
) {
  let setRows!: React.Dispatch<React.SetStateAction<LogDoc[]>>
  function Host() {
    const [rows, setLogs] = useState(initialLogs)
    setRows = setLogs
    return (
      <LogsTable
        logs={rows}
        range={range}
        columns={["service", "trace"]}
        {...props}
      />
    )
  }

  const rendered = renderWithRouter(<Host />)
  return {
    ...rendered,
    setRows: (next: React.SetStateAction<LogDoc[]>) => setRows(next),
  }
}

function makeLogs(count: number): LogDoc[] {
  return Array.from({ length: count }, (_, index) => ({
    ...log,
    _key: `log-${index}`,
    tsNanos: `${2_000_000_000 + index}`,
    traceId: `trace-${index}`,
    body: `log body ${index}`,
  }))
}

describe("logs redesign helpers", () => {
  it("computes bucket and drag windows", () => {
    const points = [
      { tsNanos: "1000000000", value: 1 },
      { tsNanos: "31000000000", value: 2 },
      { tsNanos: "61000000000", value: 3 },
    ]

    expect(bucketWindow(points, 1, 30)).toEqual({
      fromNanos: "31000000000",
      toNanos: "61000000000",
    })
    expect(dragWindow(points, 2, 0, 30)).toEqual({
      fromNanos: "1000000000",
      toNanos: "91000000000",
    })
  })

  it("round-trips optional column params", () => {
    expect(parseLogColumns("trace,event,scope,nope,trace")).toEqual([
      "trace",
      "event",
      "scope",
    ])
    expect(serializeLogColumns(["service", "scope"])).toBe("service,scope")
  })

  it("builds context windows and strips unknown saved-view params", () => {
    expect(contextWindow("35000000000")).toEqual({
      key: "custom",
      fromNanos: "5000000000",
      toNanos: "65000000000",
    })
    expect(parseSavedViewState("?service=api&sev=17&unknown=1")).toMatchObject({
      service: "api",
      sev: 17,
      live: false,
    })
  })

  it("maps all severity bands", () => {
    expect(severityVariant(1)).toBe("outline")
    expect(severityVariant(9)).toBe("secondary")
    expect(severityVariant(13)).toBe("amber")
    expect(severityVariant(17)).toBe("rose")
  })

  it("offers the event column in the menu", async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<ColumnMenu columns={["service", "trace"]} onChange={onChange} />)

    await user.click(screen.getByRole("button", { name: /columns/i }))
    expect(await screen.findByText("event")).toBeTruthy()
    await user.click(screen.getByText("event"))
    expect(onChange).toHaveBeenCalledWith(["service", "trace", "event"])
  })
})

describe("LogsTable", () => {
  it("skips virtualizer overhead for capped log sets", async () => {
    const { container } = renderWithRouter(
      <LogsTable logs={makeLogs(100)} range={range} columns={["service"]} />
    )

    expect(await screen.findByText("log body 0")).toBeTruthy()
    expect(container.querySelector("[data-virtualized='logs']")).toBeNull()
  })

  it("virtualizes log sets over the threshold", async () => {
    const { container } = renderWithRouter(
      <LogsTable logs={makeLogs(101)} range={range} columns={["service"]} />
    )

    await waitFor(() => {
      expect(container.querySelector("[data-virtualized='logs']")).toBeTruthy()
    })
  })

  it("keeps existing row DOM nodes when live logs prepend", async () => {
    const logs = [
      { ...log, tsNanos: "4000000000", traceId: "trace-a", body: "a" },
      { ...log, tsNanos: "3000000000", traceId: "trace-b", body: "b" },
      { ...log, tsNanos: "2000000000", traceId: "trace-c", body: "c" },
    ]
    const { container, setRows } = renderLogsHost(logs)
    await screen.findByText("a")
    const before = Array.from(container.querySelectorAll("tbody tr"))

    await act(async () => {
      setRows([
        { ...log, tsNanos: "5000000000", traceId: "trace-d", body: "d" },
        ...logs,
      ])
    })

    const after = Array.from(container.querySelectorAll("tbody tr"))
    expect(after[1]).toBe(before[0])
    expect(after[2]).toBe(before[1])
    expect(after[3]).toBe(before[2])
  })

  it("renders date-aware time for multi-day ranges and opens the sheet", async () => {
    const user = userEvent.setup()
    renderWithRouter(
      <LogsTable
        logs={[log]}
        range={range}
        columns={["service", "event", "trace", "scope"]}
      />
    )

    expect(await screen.findByText(formatDateTime(log.tsNanos))).toBeTruthy()
    expect(screen.getByText("checkout.completed")).toBeTruthy()
    expect(
      screen.getByRole("link", { name: "Trace trace-a" }).getAttribute("href")
    ).toBe("/traces/trace-a?range=7d")
    await user.click(screen.getByText("checkout failed"))
    expect(await screen.findByText("Log document")).toBeTruthy()
    expect(screen.getByText("event.name")).toBeTruthy()
    expect(screen.getByText("@observed")).toBeTruthy()
    expect(
      screen
        .getAllByRole("link", { name: /trace trace-a/i })
        .map((link) => link.getAttribute("href"))
    ).toContain("/traces/trace-a?range=7d")
    expect(
      screen.getByRole("link", { name: /run run-a/i }).getAttribute("href")
    ).toBe("/runs/run-a?range=7d")
  })
})

describe("LogsTable navigation", () => {
  it("preserves custom ranges in trace drilldown links", async () => {
    renderWithRouter(
      <LogsTable logs={[log]} range={custom} columns={["service", "trace"]} />
    )

    expect(await screen.findByText("checkout failed")).toBeTruthy()
    const { search, url } = parseHref(
      screen.getByRole("link", { name: "Trace trace-a" }).getAttribute("href")!
    )
    expect(url.pathname).toBe("/traces/trace-a")
    expect(search).toMatchObject({
      range: "custom",
      from: custom.fromNanos,
      to: custom.toNanos,
    })
  })

  it("opens the document sheet from keyboard row activation", async () => {
    const user = userEvent.setup()
    const { container } = renderWithRouter(
      <LogsTable logs={[log]} range={range} columns={["service", "trace"]} />
    )
    await within(container).findByText("checkout failed")
    const row = container.querySelector("tbody tr") as HTMLElement

    row.focus()
    expect(document.activeElement).toBe(row)
    await user.keyboard("{Enter}")

    expect(await screen.findByText("Log document")).toBeTruthy()
  })

  it("highlights the anchor row and exposes the context action", async () => {
    const user = userEvent.setup()
    const onShowContext = vi.fn()
    const { container } = renderWithRouter(
      <LogsTable
        logs={[log]}
        range={range}
        columns={["service", "trace"]}
        anchorNanos={log.tsNanos}
        onShowContext={onShowContext}
      />
    )
    await within(container).findByText("checkout failed")
    const row = container.querySelector("tbody tr")
    expect(row?.getAttribute("data-anchor")).toBe("true")

    await user.click(within(container).getByText("checkout failed"))
    await user.click(await screen.findByText("Show context (±30s)"))
    expect(onShowContext).toHaveBeenCalledWith(log)
  })
})

describe("SavedViewsMenu", () => {
  it("renders saved views and dispatches select/delete/save actions", async () => {
    const user = userEvent.setup()
    const view = {
      id: "view-1",
      name: "Errors",
      page: "/logs",
      state: "?sev=17",
      updatedAtNanos: "1",
    }
    const onSelect = vi.fn()
    const onDelete = vi.fn()
    const onSave = vi.fn()
    renderWithRouter(
      <SavedViewsMenu
        views={[view]}
        onSelect={onSelect}
        onDelete={onDelete}
        onSave={onSave}
      />
    )

    await user.click(await screen.findByText("Views"))
    await user.click((await screen.findAllByText("Errors"))[0]!)
    expect(onSelect).toHaveBeenCalledWith(view)

    await user.click(await screen.findByText("Views"))
    await user.click(await screen.findByText("Save current view"))
    expect(onSave).toHaveBeenCalled()

    await user.click(await screen.findByText("Views"))
    const deleteItems = await screen.findAllByText("Errors")
    await user.click(deleteItems.at(-1)!)
    expect(onDelete).toHaveBeenCalledWith("view-1")
  })
})

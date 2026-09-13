/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import { DetailSummary, SectionCard } from "@/shared/console/detail-layout"
import { QueryBar, QueryBarCount, QueryBarRow } from "@/shared/console/query-bar"
import { EmptyState } from "@/shared/console/empty-state"

afterEach(cleanup)

describe("detailSummary", () => {
  it("renders label/value pairs with tabular numerals", () => {
    const { container } = render(
      <DetailSummary
        items={[
          { label: "Events", value: "1,024" },
          { label: "Status", value: "open", hint: "since Tue" },
        ]}
      />
    )
    expect(container.querySelector("dl")).toBeTruthy()
    expect(screen.getByText("Events")).toBeTruthy()
    expect(screen.getByText("1,024")).toBeTruthy()
    expect(screen.getByText("since Tue")).toBeTruthy()
  })

  it("renders placeholders while loading", () => {
    const { container } = render(
      <DetailSummary loading items={[{ label: "Events", value: "1,024" }]} />
    )
    expect(screen.queryByText("1,024")).toBeNull()
    expect(container.querySelector('[aria-hidden="true"]')).toBeTruthy()
  })
})

describe("SectionCard", () => {
  it("anchors sections with title and action", () => {
    const { container } = render(
      <SectionCard id="stacktrace" title="Stacktrace" action={<button type="button">Copy</button>}>
        frames
      </SectionCard>
    )
    expect(container.querySelector("#stacktrace")).toBeTruthy()
    expect(screen.getByText("Stacktrace")).toBeTruthy()
    expect(screen.getByRole("button", { name: "Copy" })).toBeTruthy()
    expect(screen.getByText("frames")).toBeTruthy()
  })
})

describe("QueryBar", () => {
  it("composes rows with a right-pinned count", () => {
    render(
      <QueryBar>
        <QueryBarRow>
          <input aria-label="search" />
        </QueryBarRow>
        <QueryBarRow>
          <button type="button">Filter</button>
          <QueryBarCount shown={42} total={1024} unit="issues" />
        </QueryBarRow>
      </QueryBar>
    )
    expect(screen.getByLabelText("search")).toBeTruthy()
    const count = screen.getByText("42 of 1,024 issues")
    expect(count.className).toContain("ml-auto")
    expect(count.className).toContain("tabular-nums")
  })
})

describe("EmptyState action", () => {
  it("renders the inline next step", () => {
    render(
      <EmptyState
        title="No issues match filters"
        action={<button type="button">Clear filters</button>}
      />
    )
    expect(screen.getByRole("button", { name: "Clear filters" })).toBeTruthy()
  })
})

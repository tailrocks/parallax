/* @vitest-environment jsdom */

import { IconHome } from "@tabler/icons-react"
import { screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { PageHeader } from "@/shared/components/page-header"
import { renderTestRouter } from "@/test/router"

describe("PageHeader", () => {
  it("renders PageHeader breadcrumb shape", async () => {
    renderTestRouter(
      <PageHeader
        title="Detail"
        back={{
          href: "/",
          label: "Home",
          icon: IconHome,
        }}
      />,
      {
        componentPaths: ["/", "/dashboards"],
        initialPath: "/",
      }
    )

    expect(await screen.findByRole("link", { name: "Home" })).toBeTruthy()
    expect(
      screen.getByRole("heading", {
        name: /homedetail/i,
      }).className
    ).toContain("text-base")
  })

  it("renders title with optional actions and no back link", async () => {
    renderTestRouter(
      <PageHeader
        title="Overview"
        actions={<button type="button">Go</button>}
      />,
      {
        componentPaths: ["/"],
        initialPath: "/",
      }
    )
    expect(
      await screen.findByRole("heading", { name: /overview/i })
    ).toBeTruthy()
    expect(screen.getByRole("button", { name: "Go" })).toBeTruthy()
    expect(screen.queryByRole("link")).toBeNull()
  })
})

/* @vitest-environment jsdom */

import { cleanup, fireEvent, screen } from "@testing-library/react"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { resolvePreset } from "@/domain/time-range/range"
import { IssueDetailContent } from "@/features/issues"
import type * as IssuesApi from "@/features/issues/api/issues-api"
import { loadIssueCorrelation } from "@/features/issues/api/issues-api"
import { frameBasename, SourceMapCard } from "@/features/issues/components/source-map-card"
import type { IssueDetailData } from "@/features/issues/model/issue-detail"
import { renderTestRouter } from "@/test/router"

vi.mock("@/features/issues/api/issues-api", async (importOriginal) => {
  const actual = await importOriginal<typeof IssuesApi>()
  return {
    ...actual,
    loadIssueCorrelation: vi.fn(),
    loadSourceMaps: vi.fn(),
    uploadSourceMap: vi.fn(),
  }
})

const { loadSourceMaps, uploadSourceMap } = await import("@/features/issues/api/issues-api")

afterEach(() => {
  cleanup()
  vi.mocked(loadIssueCorrelation).mockReset()
  vi.mocked(loadSourceMaps).mockReset()
  vi.mocked(uploadSourceMap).mockReset()
})

const range = resolvePreset("24h", 1_720_000_000_000)

function detailWithMappedFrames(): IssueDetailData {
  return {
    issue: {
      fingerprint: "js-a",
      title: "Error: boom",
      errorType: "Error",
      culprit: "src/app.ts",
      service: "web",
      status: "open",
      firstSeenNanos: "1719999900000000000",
      lastSeenNanos: "1719999990000000000",
      eventCount: 2,
      lastTraceId: null,
      tags: "{}",
      groupingExplanation: null,
      events: [
        {
          tsNanos: "1719999990000000000",
          service: "web",
          serviceVersion: "1.2.3",
          message: "boom",
          stacktrace:
            "Error: boom\n    at onClick (https://cdn.example.com/app.min.js:1:25)\n    at https://cdn.example.com/app.min.js:1:99",
          source: "exception",
          traceId: "",
          spanId: "span-js",
          attributes: "{}",
          mappedFrames: [
            {
              raw: "at onClick (https://cdn.example.com/app.min.js:1:25)",
              file: "https://cdn.example.com/app.min.js",
              line: 1,
              column: 25,
              resolved: true,
              source: "src/app.ts",
              sourceLine: 3,
              sourceColumn: 2,
              name: "render",
            },
            {
              raw: "at https://cdn.example.com/app.min.js:1:99",
              file: "https://cdn.example.com/app.min.js",
              line: 1,
              column: 99,
              resolved: false,
              source: null,
              sourceLine: null,
              sourceColumn: null,
              name: null,
            },
          ],
        },
      ],
    },
    issueTrend: [{ tsNanos: "1719999990000000000", count: 2 }],
  }
}

function renderWithRouter(component: React.ReactNode, path = "/issues/web/js-a") {
  return renderTestRouter(component, {
    componentPaths: ["/issues", "/issues/$service/$fingerprint"],
    initialPath: path,
    targetPaths: ["/traces/$traceId", "/invocations/$invocationId", "/logs"],
  })
}

describe("frameBasename", () => {
  it("strips URL wrapping like the server matcher", () => {
    expect(frameBasename("https://cdn.example.com/s/app.min.js?d=1#x")).toBe("app.min.js")
    expect(frameBasename("app.min.js")).toBe("app.min.js")
  })
})

describe("mapped stacktrace", () => {
  beforeEach(() => {
    vi.mocked(loadIssueCorrelation).mockResolvedValue({ status: "trace-unavailable" })
    vi.mocked(loadSourceMaps).mockResolvedValue([])
  })

  it("renders mapped positions with a mapping badge and upload affordance", async () => {
    renderWithRouter(
      <IssueDetailContent data={detailWithMappedFrames()} range={range} onRange={() => {}} />
    )

    expect(
      await screen.findByText((_, element) => element?.textContent === "1/2 frames mapped")
    ).toBeTruthy()
    expect(screen.getByText((_, element) => element?.textContent === "src/app.ts:3:2")).toBeTruthy()
    expect(screen.getByText(/render ·/)).toBeTruthy()
    // Unresolved frame falls back to the generated position + raw line.
    expect(
      screen.getByText(
        (_, element) => element?.textContent === "https://cdn.example.com/app.min.js:1:99"
      )
    ).toBeTruthy()
    // Partial mapping surfaces the upload path inline.
    expect(await screen.findByText("Upload map")).toBeTruthy()
    expect(vi.mocked(loadSourceMaps)).toHaveBeenCalledWith("web", "1.2.3")
  })
})

describe("SourceMapCard", () => {
  it("explains the missing-version case without a version", async () => {
    renderWithRouter(
      <SourceMapCard service="web" version={null} frameFile={null} onUploaded={() => {}} />
    )
    expect(await screen.findByText("Source maps")).toBeTruthy()
    expect(screen.getByText(/no service version/)).toBeTruthy()
  })

  it("lists stored artifacts and uploads a picked map", async () => {
    vi.mocked(loadSourceMaps).mockResolvedValue([
      {
        service: "web",
        version: "1.2.3",
        file: "app.min.js",
        debugId: null,
        uploadedAtNanos: "1719999990000000000",
        mapBytes: 128,
        mapSha256: "abc123def4567890abc123def4567890abc123def4567890abc123def4567890",
      },
    ])
    vi.mocked(uploadSourceMap).mockResolvedValue({
      service: "web",
      version: "1.2.3",
      file: "app.min.js",
      debugId: null,
      uploadedAtNanos: "1719999990000000000",
      mapBytes: 64,
      mapSha256: "ff",
    })
    const onUploaded = vi.fn()
    renderWithRouter(
      <SourceMapCard
        service="web"
        version="1.2.3"
        frameFile="https://cdn.example.com/app.min.js"
        onUploaded={onUploaded}
      />
    )

    expect(await screen.findByText("app.min.js")).toBeTruthy()
    expect(screen.getByText((_, element) => element?.textContent === "128 bytes")).toBeTruthy()

    fireEvent.click(screen.getByText("Upload map"))
    const picker = (await screen.findByLabelText("Source map (.map)")) as HTMLInputElement
    const map = new File(['{"version":3}'], "app.min.js.map", { type: "application/json" })
    fireEvent.change(picker, { target: { files: [map] } })
    expect(await screen.findByText(/app\.min\.js\.map/)).toBeTruthy()

    const upload = screen.getByRole("button", { name: "Upload" })
    expect(upload.hasAttribute("disabled")).toBe(false)
    fireEvent.click(upload)
    expect(await screen.findByText("Upload map")).toBeTruthy()
    expect(vi.mocked(uploadSourceMap)).toHaveBeenCalledWith({
      service: "web",
      version: "1.2.3",
      file: "app.min.js",
      map: '{"version":3}',
      debugId: null,
    })
    expect(onUploaded).toHaveBeenCalled()
  })
})

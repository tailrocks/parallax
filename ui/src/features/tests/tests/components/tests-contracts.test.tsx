/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import { resolvePreset } from "@/domain/time-range/range"
import { TestCaseDetailContent, TestsContent, type TestsData } from "@/features/tests"
import { renderTestRouter } from "@/test/router"

afterEach(cleanup)

const range = resolvePreset("24h", 1_720_000_000_000)

const testsFixture: TestsData = {
  hasMore: false,
  services: ["checkout"],
  items: [
    {
      caseKey: "tc1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      variantKey: "tv1:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      name: "cart_total_overflow",
      suitePath: ["checkout", "cart"],
      codeReference: "src/cart.rs:99",
      explicitId: null,
      firstSeenNanos: "1719999900000000000",
      lastSeenNanos: "1719999990000000000",
      parameters: [{ name: "os", value: "linux", excluded: false }],
      invocationId: "inv-a",
      rollup: "FLAKY_PASS",
      attemptCount: 2,
      lastResult: {
        invocationId: "inv-a",
        attempt: 2,
        status: "PASSED",
        traceId: "trace-a",
        spanId: "span-a",
        startedAtNanos: "1719999980000000000",
        endedAtNanos: "1719999990000000000",
        service: "checkout",
        serviceVersion: "1.2.3",
        vcsHeadRevision: "deadbeef",
        failureFingerprint: "panic-a",
        configuration: [{ key: "os", value: "linux" }],
      },
      flaky: {
        state: "FLAKY",
        sameCommitDivergence: true,
        intraInvocationMix: true,
        transitionCount: 1,
        consecutivePasses: 0,
        updatedAtNanos: "1719999990000000000",
      },
    },
  ],
}

const detailFixture = {
  case: {
    caseKey: testsFixture.items[0]!.caseKey,
    name: testsFixture.items[0]!.name,
    identitySource: "CODE_REFERENCE" as const,
    suitePath: testsFixture.items[0]!.suitePath,
    codeReference: testsFixture.items[0]!.codeReference,
    explicitId: null,
    firstSeenNanos: testsFixture.items[0]!.firstSeenNanos,
    lastSeenNanos: testsFixture.items[0]!.lastSeenNanos,
    variants: [
      {
        variantKey: testsFixture.items[0]!.variantKey,
        parameters: testsFixture.items[0]!.parameters,
        firstSeenNanos: testsFixture.items[0]!.firstSeenNanos,
        lastSeenNanos: testsFixture.items[0]!.lastSeenNanos,
        history: [testsFixture.items[0]!.lastResult],
        flaky: testsFixture.items[0]!.flaky,
      },
    ],
  },
}

function renderWithRouter(component: React.ReactNode, path = "/tests") {
  return renderTestRouter(component, {
    componentPaths: ["/tests", "/tests/$caseKey"],
    initialPath: path,
    targetPaths: ["/traces/$traceId", "/issues/$fingerprint", "/invocations/$invocationId"],
  })
}

describe("tests surface contracts", () => {
  it("renders explorer rows with rollup and flaky evidence", async () => {
    renderWithRouter(
      <TestsContent
        data={testsFixture}
        search={{}}
        range={range}
        onSearch={() => undefined}
        onCase={() => undefined}
      />
    )
    expect(await screen.findByText("cart_total_overflow")).toBeTruthy()
    expect(screen.getByText("flaky pass")).toBeTruthy()
    expect(screen.getByText("flaky")).toBeTruthy()
    expect(screen.getByText("checkout / cart")).toBeTruthy()
  })

  it("renders detail attempt chain with cross-links", async () => {
    renderWithRouter(
      <TestCaseDetailContent data={detailFixture} range={range} onRange={() => undefined} />,
      "/tests/tc1"
    )
    expect(await screen.findByText("cart_total_overflow")).toBeTruthy()
    expect(screen.getByText("code reference")).toBeTruthy()
    expect(screen.getByRole("link", { name: "trace-a" })).toBeTruthy()
    expect(screen.getByRole("link", { name: "panic-a" })).toBeTruthy()
    expect(screen.getByRole("link", { name: "inv-a" })).toBeTruthy()
  })
})

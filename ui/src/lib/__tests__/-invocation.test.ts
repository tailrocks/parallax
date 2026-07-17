import { describe, expect, it } from "vitest"

import {
  invocationDurationNs,
  invocationStatus,
  appModeLabel,
  mergeInvocations,
} from "@/lib/invocation"

const NOW_MS = 1_720_000_000_000
const NOW_NS = BigInt(NOW_MS) * 1_000_000n

function nanos(offsetSeconds: number): string {
  return (NOW_NS + BigInt(offsetSeconds) * 1_000_000_000n).toString()
}

describe("invocationStatus", () => {
  it("derives the full matrix", () => {
    const base = {
      startedAtNanos: nanos(-600),
      endedAtNanos: null as string | null,
      exitCode: null as number | null,
      outcome: null as string | null,
      lastNanos: nanos(-1),
    }
    expect(invocationStatus(base, NOW_MS)).toBe("running")
    expect(invocationStatus({ ...base, lastNanos: nanos(-301) }, NOW_MS)).toBe(
      "stale"
    )
    expect(
      invocationStatus(
        { ...base, endedAtNanos: nanos(-10), exitCode: 0 },
        NOW_MS
      )
    ).toBe("finished")
    expect(
      invocationStatus(
        { ...base, endedAtNanos: nanos(-10), exitCode: 3 },
        NOW_MS
      )
    ).toBe("failed")
    expect(
      invocationStatus(
        { ...base, endedAtNanos: nanos(-10), exitCode: 0, outcome: "timeout" },
        NOW_MS
      )
    ).toBe("failed")
    expect(invocationStatus({ ...base, outcome: "failure" }, NOW_MS)).toBe(
      "failed"
    )
  })
})

describe("invocationDurationNs", () => {
  it("uses now while running and end when finished", () => {
    const row = {
      startedAtNanos: nanos(-100),
      endedAtNanos: nanos(-40),
      lastNanos: nanos(-40),
    }
    expect(invocationDurationNs(row, "finished", NOW_MS)).toBe(
      (60n * 1_000_000_000n).toString()
    )
    expect(
      invocationDurationNs({ ...row, endedAtNanos: null }, "running", NOW_MS)
    ).toBe((100n * 1_000_000_000n).toString())
  })
})

describe("appModeLabel", () => {
  it("labels one_shot readably and passes the rest through", () => {
    expect(appModeLabel("one_shot")).toBe("one-shot")
    expect(appModeLabel("daemon")).toBe("daemon")
    expect(appModeLabel(null)).toBeNull()
  })
})

describe("mergeInvocations", () => {
  it("prefers registered rows while keeping observed telemetry counts", () => {
    const rows = mergeInvocations(
      [
        {
          invocationId: "inv-1",
          registration: "cli" as const,
          command: "jk attach",
          appMode: null,
          outcome: "success",
          status: "finished",
          exitCode: 0,
          startedAtNanos: nanos(-100),
          endedAtNanos: nanos(-50),
          errorCount: 1,
          traceCount: 2,
          sessionCount: 1,
        },
      ],
      [
        {
          invocationId: "inv-1",
          service: "jackin",
          lastCommand: null,
          appMode: "interactive",
          firstNanos: nanos(-100),
          lastNanos: nanos(-10),
          spanCount: 9,
          logCount: 4,
        },
        {
          invocationId: "inv-2",
          service: "other-cli",
          lastCommand: "sync",
          appMode: "daemon",
          firstNanos: nanos(-30),
          lastNanos: nanos(-5),
          spanCount: 3,
          logCount: 0,
        },
      ]
    )
    expect(rows.map((row) => row.invocationId)).toEqual(["inv-2", "inv-1"])
    const merged = rows.find((row) => row.invocationId === "inv-1")!
    expect(merged.source).toBe("cli")
    expect(merged.service).toBe("jackin")
    expect(merged.appMode).toBe("interactive")
    expect(merged.spanCount).toBe(9)
    expect(merged.lastNanos).toBe(nanos(-10))
    const observedOnly = rows.find((row) => row.invocationId === "inv-2")!
    expect(observedOnly.source).toBe("external")
    expect(observedOnly.command).toBe("sync")
    expect(observedOnly.errorCount).toBeNull()
  })
})

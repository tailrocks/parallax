import { describe, expect, it } from "vitest"

import { buildJourney } from "@/components/console/invocations/session-journey"
import type { ScreenVisit, Session, UiAction } from "@/lib/api"

const session: Session = {
  sessionId: "s1",
  previousSessionId: null,
  startNanos: "1000",
  endNanos: "9000",
}

const visits: ScreenVisit[] = [
  {
    screenId: "home",
    visitId: "v1",
    sessionId: "s1",
    navigationSequence: 1,
    transitionReason: null,
    enteredNanos: "2000",
    exitedNanos: "4000",
  },
  {
    screenId: "settings",
    visitId: "v2",
    sessionId: "s1",
    navigationSequence: 2,
    transitionReason: "user_navigation",
    enteredNanos: "5000",
    exitedNanos: null,
  },
]

const actions: UiAction[] = [
  {
    name: "submit_form",
    screenId: "home",
    sessionId: "s1",
    traceId: "trace-1",
    startNanos: "3000",
    durationMs: 5,
    outcome: "success",
    hasError: false,
  },
]

describe("buildJourney", () => {
  it("interleaves entries chronologically", () => {
    const entries = buildJourney(session, visits, actions, [])
    expect(entries.map((entry) => entry.kind)).toEqual([
      "session-start",
      "screen-entered",
      "action",
      "screen-exited",
      "screen-entered",
      "session-end",
    ])
  })

  it("attributes errors to the screen whose visit interval contains them", () => {
    const entries = buildJourney(session, visits, actions, [
      {
        tsNanos: "3500",
        title: "boom on home",
        fingerprint: "fp-1",
        traceId: null,
      },
    ])
    const error = entries.find((entry) => entry.kind === "error")
    expect(error).toBeTruthy()
    expect(error?.kind === "error" && error.screenId).toBe("home")
  })

  it("keeps unattributable errors in an outside-any-screen bucket", () => {
    const entries = buildJourney(session, visits, actions, [
      {
        tsNanos: "4500",
        title: "between screens",
        fingerprint: null,
        traceId: "trace-9",
      },
    ])
    const error = entries.find((entry) => entry.kind === "error")
    expect(error).toBeTruthy()
    expect(error?.kind === "error" && error.screenId).toBeNull()
  })

  it("drops errors outside the session window", () => {
    const entries = buildJourney(session, visits, actions, [
      {
        tsNanos: "99000",
        title: "later error",
        fingerprint: null,
        traceId: null,
      },
    ])
    expect(entries.some((entry) => entry.kind === "error")).toBe(false)
  })

  it("marks an open session without a session-end entry", () => {
    const open = { ...session, endNanos: null }
    const entries = buildJourney(open, visits, actions, [])
    expect(entries.at(-1)?.kind).not.toBe("session-end")
  })
})

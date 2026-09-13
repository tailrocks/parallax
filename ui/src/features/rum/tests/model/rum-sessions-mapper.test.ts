import { describe, expect, it } from "vitest"

import { mapRumSession, mapRumSessions } from "@/features/rum/api/rum-mapper"
import type { RumSessionQuery } from "@/features/rum/api/rum-session.generated"
import type { RumSessionsQuery } from "@/features/rum/api/rum-sessions.generated"

describe("rum session mappers", () => {
  it("maps the sessions inbox rows", () => {
    const data = {
      rumSessions: [
        {
          sessionId: "sess-1",
          service: "web",
          startNanos: "10",
          endNanos: "40",
          spanCount: 4,
          traceCount: 2,
          viewCount: 2,
          vitalCount: 1,
          errorCount: 1,
          hasError: true,
        },
      ],
    } as RumSessionsQuery
    const rows = mapRumSessions(data)
    expect(rows).toHaveLength(1)
    expect(rows[0]?.sessionId).toBe("sess-1")
    expect(rows[0]?.viewCount).toBe(2)
    expect(rows[0]?.hasError).toBe(true)
  })

  it("maps the session timeline and nulls unknown sessions", () => {
    const data = {
      rumSession: {
        session: {
          sessionId: "sess-1",
          service: "web",
          startNanos: "10",
          endNanos: "40",
          spanCount: 4,
          traceCount: 2,
          viewCount: 1,
          vitalCount: 1,
          errorCount: 1,
          hasError: true,
        },
        views: [{ tsNanos: "10", screen: "home", path: "/", traceId: "t1", spanId: "s1" }],
        vitals: [
          { tsNanos: "20", name: "LCP", value: 1200, rating: "good", traceId: "t1", spanId: "s2" },
        ],
        errors: [
          {
            tsNanos: "40",
            name: "web.error.handled",
            errorType: "TypeError",
            message: "boom",
            traceId: "t2",
            spanId: "s3",
          },
        ],
      },
    } as RumSessionQuery
    const detail = mapRumSession(data)
    expect(detail?.views).toHaveLength(1)
    expect(detail?.views[0]?.screen).toBe("home")
    expect(detail?.vitals[0]?.name).toBe("LCP")
    expect(detail?.errors[0]?.errorType).toBe("TypeError")

    expect(mapRumSession({ rumSession: null } as RumSessionQuery)).toBeNull()
  })
})

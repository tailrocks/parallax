# Plan 038 Route Range Continuity Evidence

Timestamp: 2026-07-10T06:02:01Z

Environment:

- Parallax repo: `dbaba3cd612375e725b77de0bdc0aeb2867730e9` plus the route-range proof patch in this change set.
- Playground repo: `830d2c9179dfc5dadd00bc3e2d4d10bcb6f7a9d4`.
- UI package manager: `bun 1.3.14`.

Scope:

- Issues, Services, Logs, Traces, and Dashboards preserve route range state across drilldown links.
- Preset range navigation keeps `range=<preset>` and does not carry stale `from`/`to` bounds.
- Custom range navigation preserves absolute nanosecond `from`/`to` values.
- Dashboard card links and create navigation use the same detail search payload.

Implementation proof:

- `src/routes/__tests__/-issues.test.tsx`
  - renders custom range links for `/services/$service`, `/traces/$traceId`, and `/issues/$fingerprint`.
- `src/routes/__tests__/-services.test.tsx`
  - renders custom range links from service index and detail drilldowns to service detail, traces, logs, issues, and top trace exemplar.
- `src/routes/__tests__/-logs.test.tsx`
  - renders custom range links from log rows to trace detail.
- `src/routes/__tests__/-traces-search.test.tsx`
  - verifies trace detail search generation for preset and custom ranges.
- `src/routes/__tests__/-final-sweep.test.tsx`
  - verifies preset dashboard search drops stale absolute bounds.
  - renders dashboard card links with custom range search.
  - verifies dashboard create navigation receives custom range search unchanged.

Verification command:

```bash
rtk bun run test src/routes/__tests__/-issues.test.tsx src/routes/__tests__/-services.test.tsx src/routes/__tests__/-logs.test.tsx src/routes/__tests__/-traces-search.test.tsx src/routes/__tests__/-final-sweep.test.tsx
```

Result:

```text
Test Files  5 passed (5)
Tests       35 passed (35)
```

Notes:

- TanStack Router default search serialization quotes numeric-looking strings in rendered hrefs so JSON parsing preserves big nanosecond values exactly. Tests assert both rendered paths and router-parsed search payloads, matching runtime route behavior.
- No custom GreptimeDB tables are involved in this plan; this proof is route/UI state only.

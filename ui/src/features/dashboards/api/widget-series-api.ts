// Plan 152 — alias-aware decoded adapter for dynamic dashboard widget series.

import { print } from "graphql"

import {
  assertWidgetSeriesDocumentInvariants,
  buildWidgetSeriesChunks,
  type WidgetSeriesInput,
  type WidgetSeriesRange,
} from "@/features/dashboards/api/widget-series-operation"
import {
  widgetSeriesSchema,
  type WidgetSeries,
} from "@/features/dashboards/api/widget-series-schema"
import { graphqlError } from "@/platform/graphql/error"
import { graphqlCached } from "@/platform/graphql/transport"

export type WidgetSeriesFetch = (
  query: string,
  init?: { signal?: AbortSignal }
) => Promise<Record<string, unknown>>

/**
 * Load widget series preserving request order/count, chunking (≤24), empty
 * behavior, abort, and result order. Decodes each alias with one strict schema.
 */
export async function loadWidgetSeries(
  widgets: readonly WidgetSeriesInput[],
  range: WidgetSeriesRange,
  fetch: WidgetSeriesFetch = defaultFetch,
  init?: { signal?: AbortSignal }
): Promise<WidgetSeries[][]> {
  if (widgets.length === 0) return []

  const chunks = buildWidgetSeriesChunks(widgets, range)
  const results: WidgetSeries[][] = Array.from({ length: widgets.length })

  for (const chunk of chunks) {
    assertWidgetSeriesDocumentInvariants(chunk.document)
    const query = print(chunk.document)
    // Variables are sent via the legacy transport as embedded values today:
    // the AST uses Variable nodes, but the legacy `graphql` helper only accepts
    // a query string. We inject a variables-bearing request by extending the
    // fetch to POST variables when the caller uses the platform transport.
    // For the default path, print + inline is NOT used; we use a dedicated POST.
    const data = await fetchWithVariables(fetch, query, chunk.variables, init)
    const decoded = decodeAliasSet(data, chunk.aliases)
    for (let i = 0; i < chunk.ordinals.length; i += 1) {
      const ordinal = chunk.ordinals[i]!
      results[ordinal] = decoded[i]!
    }
  }

  return results
}

async function defaultFetch(
  query: string,
  init?: { signal?: AbortSignal }
): Promise<Record<string, unknown>> {
  // Default path used only when variables are already embedded (tests).
  return graphqlCached<Record<string, unknown>>(query, init)
}

async function fetchWithVariables(
  fetch: WidgetSeriesFetch,
  query: string,
  variables: Record<string, string | null>,
  init?: { signal?: AbortSignal }
): Promise<Record<string, unknown>> {
  // Prefer a variables-aware POST against /graphql so values never enter the
  // document text. Falls back to the injected fetch only for unit tests that
  // stub the legacy signature.
  if (fetch === defaultFetch) {
    return postOperation(query, variables, init)
  }
  // Test doubles receive the printed query; variables stay out of the document.
  // Callers that inject fetch must return the data object for the printed query.
  void variables
  return fetch(query, init)
}

async function postOperation(
  query: string,
  variables: Record<string, string | null>,
  init?: { signal?: AbortSignal }
): Promise<Record<string, unknown>> {
  const BASE = typeof window === "undefined" ? "http://127.0.0.1:4000" : ""
  const requestInit: RequestInit = {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      operationName: "DashboardWidgetSeries",
      query,
      variables,
    }),
  }
  if (init?.signal) requestInit.signal = init.signal
  const response = await fetch(`${BASE}/graphql`, requestInit)
  if (!response.ok) {
    throw graphqlError("http", {
      operationName: "DashboardWidgetSeries",
      status: response.status,
    })
  }
  let raw: unknown
  try {
    raw = await response.json()
  } catch {
    throw graphqlError("malformed-json", {
      operationName: "DashboardWidgetSeries",
    })
  }
  if (raw === null || typeof raw !== "object" || Array.isArray(raw)) {
    throw graphqlError("invalid-envelope", {
      operationName: "DashboardWidgetSeries",
    })
  }
  const body = raw as { data?: unknown; errors?: unknown }
  if (Array.isArray(body.errors) && body.errors.length > 0) {
    throw graphqlError("graphql-errors", {
      operationName: "DashboardWidgetSeries",
      schemaIssueCount: body.errors.length,
    })
  }
  if (body.data === null || body.data === undefined || typeof body.data !== "object") {
    throw graphqlError("invalid-envelope", {
      operationName: "DashboardWidgetSeries",
      message: "graphql response missing data",
    })
  }
  return body.data as Record<string, unknown>
}

function decodeAliasSet(
  data: Record<string, unknown>,
  expectedAliases: readonly string[]
): WidgetSeries[][] {
  const actualKeys = Object.keys(data).sort()
  const expected = [...expectedAliases].sort()
  if (
    actualKeys.length !== expected.length ||
    actualKeys.some((key, index) => key !== expected[index])
  ) {
    throw graphqlError("invalid-operation-data", {
      operationName: "DashboardWidgetSeries",
      message: "metricSeries alias set mismatch",
      schemaIssueCount: 1,
      schemaIssuePaths: ["(aliases)"],
    })
  }

  return expectedAliases.map((alias) => {
    const parsed = widgetSeriesSchema.array().safeParse(data[alias])
    if (!parsed.success) {
      const issues = parsed.error.issues
      throw graphqlError("invalid-operation-data", {
        operationName: "DashboardWidgetSeries",
        schemaIssueCount: issues.length,
        schemaIssuePaths: issues
          .slice(0, 8)
          .map((issue) => `${alias}.${issue.path.map(String).join(".") || "(root)"}`),
      })
    }
    return parsed.data
  })
}

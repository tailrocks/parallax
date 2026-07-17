import { expect } from "@playwright/test"
import { readFileSync } from "fs"
import { createConnection } from "net"
import { join } from "path"

export interface FullStackRuntimeManifest {
  schema_version: number
  mode: "attach" | "managed" | string
  storage: string
  base_url: string
  health_url: string
  ready_url?: string
  graphql_url: string
  otlp_http_url: string
  control_url: string
  dataset_id: string
  service: string
  trace_id: string
  span_id: string
  invocation_id: string
  session_id: string
  error_type: string
  error_message: string
  log_body: string
  metric_name: string
  issue_fingerprint: string
  issue_status?: string
  start_nanos: string
  owns_process: boolean
}

export interface FullStackIssueSnapshot {
  fingerprint: string
  title: string
  status: string
  service: string
  errorType?: string
}

function runtimeManifestPath(): string {
  return join(process.cwd(), "test-results", "browser-full-stack-runtime.json")
}

export function readFullStackManifest(): FullStackRuntimeManifest {
  const raw = readFileSync(runtimeManifestPath(), "utf8")
  const parsed = JSON.parse(raw) as FullStackRuntimeManifest
  if (parsed.schema_version !== 1) {
    throw new Error(`unsupported full-stack runtime schema ${String(parsed.schema_version)}`)
  }
  if (parsed.storage !== "managed-greptime+turso") {
    throw new Error(`full-stack storage must be managed-greptime+turso, got ${parsed.storage}`)
  }
  return parsed
}

function controlHostPort(): { host: string; port: number } {
  const manifest = readFullStackManifest()
  // control_url is tcp://host:port
  const url = new URL(manifest.control_url.replace(/^tcp:\/\//, "http://"))
  return { host: url.hostname, port: Number(url.port) }
}

async function controlRequest(body: Record<string, unknown>): Promise<unknown> {
  const { host, port } = controlHostPort()
  const payload = `${JSON.stringify(body)}\n`
  return new Promise((resolve, reject) => {
    const socket = createConnection({ host, port }, () => {
      socket.write(payload)
    })
    let data = ""
    socket.setEncoding("utf8")
    socket.on("data", (chunk: string | Buffer) => {
      data += typeof chunk === "string" ? chunk : chunk.toString("utf8")
    })
    socket.on("end", () => {
      try {
        resolve(JSON.parse(data.trim()) as unknown)
      } catch (error) {
        reject(error)
      }
    })
    socket.on("error", reject)
  })
}

export async function fullStackSnapshot(): Promise<{
  ok: boolean
  dataset_id: string
  service: string
  trace_id: string
  issue: FullStackIssueSnapshot | null
}> {
  const response = (await controlRequest({ op: "snapshot" })) as {
    ok?: boolean
    dataset_id: string
    service: string
    trace_id: string
    issue: FullStackIssueSnapshot | null
  }
  expect(response.ok).toBe(true)
  return response as {
    ok: boolean
    dataset_id: string
    service: string
    trace_id: string
    issue: FullStackIssueSnapshot | null
  }
}

export async function seedLiveLog(
  body?: string,
  tsNanos?: string
): Promise<{ body: string; ts_nanos: string }> {
  const response = (await controlRequest({
    op: "seed-live-log",
    body,
    ...(tsNanos ? { ts_nanos: tsNanos } : {}),
  })) as { ok?: boolean; body?: string; ts_nanos?: string; error?: string }
  expect(response.ok, response.error ?? "seed-live-log failed").toBe(true)
  return { body: response.body ?? "", ts_nanos: response.ts_nanos ?? "" }
}

export async function seedLiveLogBurst(
  count = 5,
  prefix?: string
): Promise<{ prefix: string; count: number; bodies: string[]; ts_nanos: string }> {
  const response = (await controlRequest({
    op: "seed-live-log-burst",
    count,
    body: prefix,
  })) as {
    ok?: boolean
    prefix?: string
    count?: number
    bodies?: string[]
    ts_nanos?: string
    error?: string
  }
  expect(response.ok, response.error ?? "seed-live-log-burst failed").toBe(true)
  return {
    prefix: response.prefix ?? "",
    count: response.count ?? 0,
    bodies: response.bodies ?? [],
    ts_nanos: response.ts_nanos ?? "",
  }
}

/** One OTLP export containing two identical log rows (plan 147 identity merge). */
export async function seedLiveLogDuplicatePair(
  body?: string
): Promise<{ body: string; ts_nanos: string }> {
  const response = (await controlRequest({
    op: "seed-live-log-duplicate-pair",
    body,
  })) as { ok?: boolean; body?: string; ts_nanos?: string; error?: string }
  expect(response.ok, response.error ?? "seed-live-log-duplicate-pair failed").toBe(true)
  return { body: response.body ?? "", ts_nanos: response.ts_nanos ?? "" }
}

export async function seedLiveSpan(options?: {
  spanName?: string
  spanId?: string
  tsNanos?: string
}): Promise<{ span_name: string; span_id: string; ts_nanos: string }> {
  const response = (await controlRequest({
    op: "seed-live-span",
    span_name: options?.spanName,
    span_id: options?.spanId,
    ...(options?.tsNanos ? { ts_nanos: options.tsNanos } : {}),
  })) as {
    ok?: boolean
    span_name?: string
    span_id?: string
    ts_nanos?: string
    error?: string
  }
  expect(response.ok, response.error ?? "seed-live-span failed").toBe(true)
  return {
    span_name: response.span_name ?? "",
    span_id: response.span_id ?? "",
    ts_nanos: response.ts_nanos ?? "",
  }
}

/** Two identical spans in one export — merge must keep one row by spanId. */
export async function seedLiveSpanDuplicatePair(options?: {
  spanName?: string
  spanId?: string
}): Promise<{ span_name: string; span_id: string; ts_nanos: string }> {
  const response = (await controlRequest({
    op: "seed-live-span-duplicate-pair",
    span_name: options?.spanName,
    span_id: options?.spanId,
  })) as {
    ok?: boolean
    span_name?: string
    span_id?: string
    ts_nanos?: string
    error?: string
  }
  expect(response.ok, response.error ?? "seed-live-span-duplicate-pair failed").toBe(true)
  return {
    span_name: response.span_name ?? "",
    span_id: response.span_id ?? "",
    ts_nanos: response.ts_nanos ?? "",
  }
}

export async function graphqlQuery<T>(query: string): Promise<T> {
  const manifest = readFullStackManifest()
  const response = await fetch(manifest.graphql_url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ query }),
  })
  expect(response.ok).toBe(true)
  const json = (await response.json()) as { data?: T; errors?: unknown }
  expect(json.errors, JSON.stringify(json.errors)).toBeUndefined()
  expect(json.data).toBeTruthy()
  return json.data as T
}

export async function pollIssueStatus(
  fingerprint: string,
  expected: string,
  deadlineMs = 15_000
): Promise<FullStackIssueSnapshot> {
  const started = Date.now()
  let last: FullStackIssueSnapshot | null = null
  while (Date.now() - started < deadlineMs) {
    const data = await graphqlQuery<{
      issue: FullStackIssueSnapshot
    }>(`{ issue(fingerprint: "${fingerprint}") { fingerprint title status service errorType } }`)
    last = data.issue
    if (last?.status === expected) {
      return last
    }
    await new Promise((resolve) => setTimeout(resolve, 250))
  }
  throw new Error(
    `issue ${fingerprint} status did not become ${expected}; last=${JSON.stringify(last)}`
  )
}

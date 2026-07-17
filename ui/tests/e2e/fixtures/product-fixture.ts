import { expect } from "@playwright/test"
import { createConnection } from "node:net"
import { readFileSync } from "node:fs"
import { join } from "node:path"

export type ProductDatasetId = "shell-empty" | "investigations-pilot"

export interface RuntimeManifest {
  schema_version: number
  base_url: string
  health_url: string
  control_url: string
  dataset_id: string
  pid: number
}

export interface InvestigationSnapshot {
  id: string
  name: string
  state: string
}

export interface ControlSnapshot {
  ok: boolean
  dataset_id?: string
  investigations: InvestigationSnapshot[]
  counts: {
    spans: number
    logs: number
    metrics: number
    error_events: number
  }
}

function runtimeManifestPath(): string {
  return join(process.cwd(), "test-results", "browser-contracts-runtime.json")
}

export function readRuntimeManifest(): RuntimeManifest {
  const raw = readFileSync(runtimeManifestPath(), "utf8")
  const parsed = JSON.parse(raw) as RuntimeManifest
  if (parsed.schema_version !== 1) {
    throw new Error(
      `unsupported runtime manifest schema ${String(parsed.schema_version)}`
    )
  }
  return parsed
}

function controlHostPort(): { host: string; port: number } {
  const manifest = readRuntimeManifest()
  const url = new URL(manifest.control_url)
  return { host: url.hostname, port: Number(url.port) }
}

async function controlRequest(body: Record<string, unknown>): Promise<unknown> {
  const { host, port } = controlHostPort()
  const payload = `${JSON.stringify(body)}\n`
  return await new Promise((resolve, reject) => {
    const socket = createConnection({ host, port }, () => {
      socket.write(payload)
    })
    let data = ""
    socket.setEncoding("utf8")
    socket.on("data", (chunk) => {
      data += chunk
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

export async function resetDataset(dataset: ProductDatasetId): Promise<void> {
  const response = (await controlRequest({ op: "reset", dataset })) as {
    ok?: boolean
    error?: string
    dataset_id?: string
  }
  expect(response.ok, response.error ?? "reset failed").toBe(true)
  expect(response.dataset_id).toBe(dataset)
}

export async function snapshotState(): Promise<ControlSnapshot> {
  const response = (await controlRequest({ op: "snapshot" })) as ControlSnapshot
  expect(response.ok).toBe(true)
  return response
}

export async function failNextGraphql(): Promise<void> {
  const response = (await controlRequest({ op: "fail-next-graphql" })) as {
    ok?: boolean
  }
  expect(response.ok).toBe(true)
}

export async function controlPing(): Promise<void> {
  const response = (await controlRequest({ op: "ping" })) as { ok?: boolean }
  expect(response.ok).toBe(true)
}

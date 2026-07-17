/** Runtime manifest shape published by `cargo xtask browser-foundation-serve`. */

export interface FoundationRuntimeManifest {
  schema_version: number
  bind: string
  port: number
  health_url: string
  ui_dist: string
  pid: number
}

export function parseRuntimeManifest(raw: string): FoundationRuntimeManifest {
  const parsed = JSON.parse(raw) as FoundationRuntimeManifest
  if (parsed.schema_version !== 1) {
    throw new Error(`unsupported runtime manifest schema ${String(parsed.schema_version)}`)
  }
  return parsed
}

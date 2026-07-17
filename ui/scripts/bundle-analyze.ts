// Plan 148 — production client size snapshot (deterministic local analysis).
// Writes JSON under target/ui-bundle/ (outside ui source). Does not embed maps.

import { readdir, stat, mkdir, writeFile, readFile } from "node:fs/promises"
import { dirname, join, relative } from "node:path"
import { fileURLToPath } from "node:url"
import { gzipSync } from "node:zlib"
import { spawnSync } from "node:child_process"

const uiRoot = join(dirname(fileURLToPath(import.meta.url)), "..")
const repoRoot = join(uiRoot, "..")
const outDir = join(repoRoot, "target", "ui-bundle")
const clientCandidates = [
  join(uiRoot, "dist", "client"),
  join(uiRoot, "dist"),
  join(uiRoot, ".output", "public"),
]

async function findClientDir(): Promise<string | null> {
  for (const candidate of clientCandidates) {
    try {
      const s = await stat(candidate)
      if (s.isDirectory()) return candidate
    } catch {
      // try next
    }
  }
  return null
}

async function walkAssets(
  dir: string,
  base = dir
): Promise<Array<{ path: string; bytes: number }>> {
  const entries = await readdir(dir, { withFileTypes: true })
  const out: Array<{ path: string; bytes: number }> = []
  for (const entry of entries) {
    const full = join(dir, entry.name)
    if (entry.isDirectory()) {
      out.push(...(await walkAssets(full, base)))
      continue
    }
    if (entry.name.endsWith(".map")) {
      out.push({ path: relative(base, full), bytes: (await stat(full)).size })
      continue
    }
    if (!entry.name.endsWith(".js") && !entry.name.endsWith(".css")) continue
    const bytes = (await stat(full)).size
    out.push({ path: relative(base, full), bytes })
  }
  return out
}

async function main() {
  const runBuild = process.argv.includes("--build")
  if (runBuild) {
    console.log("building production UI…")
    const result = spawnSync("bun", ["run", "build"], {
      cwd: uiRoot,
      stdio: "inherit",
    })
    if (result.status !== 0) {
      process.exit(result.status ?? 1)
    }
  }

  const clientDir = await findClientDir()
  if (!clientDir) {
    console.error("no production client directory found; run with --build")
    process.exit(2)
  }

  const files = await walkAssets(clientDir)
  const maps = files.filter((f) => f.path.endsWith(".map"))
  const assets = []
  for (const file of files) {
    if (file.path.endsWith(".map")) continue
    const buf = await readFile(join(clientDir, file.path))
    assets.push({
      path: file.path,
      raw: file.bytes,
      gzip: gzipSync(buf).length,
    })
  }

  const totalRaw = assets.reduce((sum, a) => sum + a.raw, 0)
  const totalGzip = assets.reduce((sum, a) => sum + a.gzip, 0)
  const report = {
    generatedAt: new Date().toISOString(),
    clientDir: relative(repoRoot, clientDir),
    fileCount: assets.length,
    totalRaw,
    totalGzip,
    sourceMapFiles: maps.length,
    assets: assets.sort((a, b) => b.gzip - a.gzip),
  }

  await mkdir(outDir, { recursive: true })
  const outPath = join(outDir, "current.json")
  await writeFile(outPath, `${JSON.stringify(report, null, 2)}\n`)
  console.log(`wrote ${relative(repoRoot, outPath)}`)
  console.log(`files=${report.fileCount} raw=${totalRaw} gzip=${totalGzip} maps=${maps.length}`)
  if (maps.length > 0) {
    console.error("FAIL: source maps present in client output")
    process.exit(3)
  }
}

await main()

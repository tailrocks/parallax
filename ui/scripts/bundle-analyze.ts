// Plan 148 — production client size snapshot, budget gate, two-clean-build check.
// Writes JSON under target/ui-bundle/ (outside ui source). Does not embed maps.

import { readdir, stat, mkdir, writeFile, readFile, rm, cp } from "node:fs/promises"
import { createHash } from "node:crypto"
import { dirname, join, relative } from "node:path"
import { fileURLToPath } from "node:url"
import { gzipSync } from "node:zlib"
import { spawnSync } from "node:child_process"

const uiRoot = join(dirname(fileURLToPath(import.meta.url)), "..")
const repoRoot = join(uiRoot, "..")
const outDir = join(repoRoot, "target", "ui-bundle")
const budgetsPath = join(uiRoot, "bundle-budgets.json")
const clientCandidates = [
  join(uiRoot, "dist", "client"),
  join(uiRoot, "dist"),
  join(uiRoot, ".output", "public"),
]

interface AssetRow {
  path: string
  raw: number
  gzip: number
  sha256: string
}

interface Report {
  generatedAt: string
  clientDir: string
  fileCount: number
  totalRaw: number
  totalGzip: number
  sourceMapFiles: number
  largestGzip: number
  assets: AssetRow[]
}

interface Budgets {
  schema_version: number
  total_raw_ceiling: number
  total_gzip_ceiling: number
  file_count_ceiling: number
  largest_gzip_ceiling: number
  source_map_files_ceiling: number
  /** Deterministic build tolerance: 0 means identical inventory after normalize. */
  clean_build_tolerance_bytes: number
}

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

async function buildReport(clientDir: string): Promise<Report> {
  const files = await walkAssets(clientDir)
  const maps = files.filter((f) => f.path.endsWith(".map"))
  const assets: AssetRow[] = []
  for (const file of files) {
    if (file.path.endsWith(".map")) continue
    const buf = await readFile(join(clientDir, file.path))
    assets.push({
      path: file.path,
      raw: file.bytes,
      gzip: gzipSync(buf).length,
      sha256: createHash("sha256").update(buf).digest("hex"),
    })
  }
  assets.sort((a, b) => b.gzip - a.gzip)
  const totalRaw = assets.reduce((sum, a) => sum + a.raw, 0)
  const totalGzip = assets.reduce((sum, a) => sum + a.gzip, 0)
  return {
    generatedAt: new Date().toISOString(),
    clientDir: relative(repoRoot, clientDir),
    fileCount: assets.length,
    totalRaw,
    totalGzip,
    sourceMapFiles: maps.length,
    largestGzip: assets[0]?.gzip ?? 0,
    assets,
  }
}

function runBuild(): void {
  console.log("building production UI…")
  const result = spawnSync("bun", ["run", "build"], {
    cwd: uiRoot,
    stdio: "inherit",
  })
  if (result.status !== 0) {
    process.exit(result.status ?? 1)
  }
}

async function loadBudgets(): Promise<Budgets> {
  const raw = await readFile(budgetsPath, "utf8")
  return JSON.parse(raw) as Budgets
}

function checkBudgets(report: Report, budgets: Budgets): string[] {
  const failures: string[] = []
  if (report.sourceMapFiles > budgets.source_map_files_ceiling) {
    failures.push(
      `source maps present: ${report.sourceMapFiles} > ceiling ${budgets.source_map_files_ceiling}`
    )
  }
  if (report.totalRaw > budgets.total_raw_ceiling) {
    failures.push(`totalRaw ${report.totalRaw} > ceiling ${budgets.total_raw_ceiling}`)
  }
  if (report.totalGzip > budgets.total_gzip_ceiling) {
    failures.push(`totalGzip ${report.totalGzip} > ceiling ${budgets.total_gzip_ceiling}`)
  }
  if (report.fileCount > budgets.file_count_ceiling) {
    failures.push(`fileCount ${report.fileCount} > ceiling ${budgets.file_count_ceiling}`)
  }
  if (report.largestGzip > budgets.largest_gzip_ceiling) {
    failures.push(`largestGzip ${report.largestGzip} > ceiling ${budgets.largest_gzip_ceiling}`)
  }
  return failures
}

/** Normalize content hashes ignoring absolute paths in asset file names. */
function inventoryKey(report: Report): string {
  const rows = report.assets
    .map((a) => {
      // Strip content hashes from Vite filenames for path compare; keep sha of bytes.
      const normalizedPath = a.path.replace(/-[A-Za-z0-9_]{6,}\./g, "-[hash].").replace(/\\/g, "/")
      return `${normalizedPath}:${a.raw}:${a.gzip}:${a.sha256}`
    })
    .sort()
  return createHash("sha256").update(rows.join("\n")).digest("hex")
}

async function writeReport(name: string, report: Report): Promise<string> {
  await mkdir(outDir, { recursive: true })
  const outPath = join(outDir, name)
  await writeFile(outPath, `${JSON.stringify(report, null, 2)}\n`)
  return outPath
}

async function analyze(runBuildFirst: boolean, check: boolean): Promise<void> {
  if (runBuildFirst) runBuild()
  const clientDir = await findClientDir()
  if (!clientDir) {
    console.error("no production client directory found; run with --build")
    process.exit(2)
  }
  const report = await buildReport(clientDir)
  const outPath = await writeReport("current.json", report)
  console.log(`wrote ${relative(repoRoot, outPath)}`)
  console.log(
    `files=${report.fileCount} raw=${report.totalRaw} gzip=${report.totalGzip} maps=${report.sourceMapFiles} largestGzip=${report.largestGzip}`
  )
  if (report.sourceMapFiles > 0) {
    console.error("FAIL: source maps present in client output")
    process.exit(3)
  }
  if (check) {
    const budgets = await loadBudgets()
    const failures = checkBudgets(report, budgets)
    if (failures.length > 0) {
      for (const failure of failures) console.error(`FAIL budget: ${failure}`)
      process.exit(4)
    }
    console.log("budgets ok")
  }
}

async function buildTwice(): Promise<void> {
  const budgets = await loadBudgets()
  const staging = join(outDir, "clean-build-staging")
  await mkdir(outDir, { recursive: true })
  await rm(staging, { recursive: true, force: true })
  await mkdir(staging, { recursive: true })

  const snapshots: Report[] = []
  for (const label of ["a", "b"] as const) {
    // Wipe dist so each build is clean.
    await rm(join(uiRoot, "dist"), { recursive: true, force: true })
    runBuild()
    const clientDir = await findClientDir()
    if (!clientDir) {
      console.error(`clean build ${label}: no client dir`)
      process.exit(2)
    }
    const report = await buildReport(clientDir)
    await writeReport(`clean-build-${label}.json`, report)
    // Preserve client snapshot for offline compare.
    const snapDir = join(staging, label)
    await rm(snapDir, { recursive: true, force: true })
    await cp(clientDir, snapDir, { recursive: true })
    snapshots.push(report)
    console.log(
      `clean-build-${label}: files=${report.fileCount} raw=${report.totalRaw} gzip=${report.totalGzip} maps=${report.sourceMapFiles}`
    )
    if (report.sourceMapFiles > 0) {
      console.error(`FAIL: source maps in clean build ${label}`)
      process.exit(3)
    }
  }

  const [a, b] = snapshots
  if (!a || !b) {
    console.error("FAIL: missing clean-build snapshots")
    process.exit(5)
  }

  const keyA = inventoryKey(a)
  const keyB = inventoryKey(b)
  const rawDelta = Math.abs(a.totalRaw - b.totalRaw)
  const gzipDelta = Math.abs(a.totalGzip - b.totalGzip)

  const compare = {
    generatedAt: new Date().toISOString(),
    inventoryHashA: keyA,
    inventoryHashB: keyB,
    identical: keyA === keyB,
    rawDelta,
    gzipDelta,
    fileCountA: a.fileCount,
    fileCountB: b.fileCount,
  }
  await writeFile(join(outDir, "clean-build-compare.json"), `${JSON.stringify(compare, null, 2)}\n`)

  if (keyA !== keyB) {
    // Allow only documented byte tolerance on totals when content hashes differ by
    // absolute-path noise already normalized out of inventoryKey — if inventory
    // hashes differ, fail hard (true nondeterminism).
    console.error("FAIL: two clean builds produced different normalized inventories")
    console.error(JSON.stringify(compare, null, 2))
    process.exit(6)
  }
  if (
    rawDelta > budgets.clean_build_tolerance_bytes ||
    gzipDelta > budgets.clean_build_tolerance_bytes
  ) {
    console.error(
      `FAIL: clean build size delta raw=${rawDelta} gzip=${gzipDelta} exceeds tolerance ${budgets.clean_build_tolerance_bytes}`
    )
    process.exit(7)
  }

  // Keep current.json aligned with the second clean build.
  await writeReport("current.json", b)
  const failures = checkBudgets(b, budgets)
  if (failures.length > 0) {
    for (const failure of failures) console.error(`FAIL budget: ${failure}`)
    process.exit(4)
  }
  console.log("two clean builds identical; budgets ok")
}

async function main() {
  const args = new Set(process.argv.slice(2))
  if (args.has("--build-twice")) {
    await buildTwice()
    return
  }
  await analyze(args.has("--build"), args.has("--check") || args.has("--check-budgets"))
}

await main()

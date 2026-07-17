export interface Frame {
  raw: string
  fn?: string
  file?: string
  line?: number
  col?: number
  isApp?: boolean
}

const libraryMarkers = [
  "site-packages",
  "node_modules",
  ".cargo/registry",
  "/rustc/",
  "/go/pkg/mod/",
  "/goroot/",
  "/jdk",
  "/jre/",
]

function toNumber(value: string | undefined): number | undefined {
  if (!value) return undefined
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : undefined
}

function classify(file: string | undefined): boolean | undefined {
  if (!file) return undefined
  const lower = file.toLowerCase()
  return !libraryMarkers.some((marker) => lower.includes(marker))
}

type FrameInput = {
  fn?: string | undefined
  file?: string | undefined
  line?: number | undefined
  col?: number | undefined
}

function frame(raw: string, data: FrameInput): Frame {
  const result: Frame = { raw }
  if (data.fn) result.fn = data.fn
  if (data.file) result.file = data.file
  if (data.line != null) result.line = data.line
  if (data.col != null) result.col = data.col
  const isApp = classify(data.file)
  if (isApp != null) result.isApp = isApp
  return result
}

export function parseStacktrace(input: string | null | undefined): Frame[] {
  if (!input) return []
  const lines = input
    .split(/\r?\n/)
    .map((line) => line.trimEnd())
    .filter((line) => line.trim().length > 0)
  const frames: Frame[] = []

  for (let index = 0; index < lines.length; index += 1) {
    const raw = lines[index]!
    const trimmed = raw.trim()
    const next = lines[index + 1]?.trim()

    const python = trimmed.match(/^File "(.+)", line (\d+), in (.+)$/)
    if (python) {
      frames.push(
        frame(raw, {
          file: python[1],
          line: toNumber(python[2]),
          fn: python[3],
        })
      )
      continue
    }

    const v8WithFn = trimmed.match(/^at (.+) \((.+):(\d+):(\d+)\)$/)
    if (v8WithFn) {
      frames.push(
        frame(raw, {
          fn: v8WithFn[1],
          file: v8WithFn[2],
          line: toNumber(v8WithFn[3]),
          col: toNumber(v8WithFn[4]),
        })
      )
      continue
    }

    const v8Bare = trimmed.match(/^at (.+):(\d+):(\d+)$/)
    if (v8Bare) {
      frames.push(
        frame(raw, {
          file: v8Bare[1],
          line: toNumber(v8Bare[2]),
          col: toNumber(v8Bare[3]),
        })
      )
      continue
    }

    const java = trimmed.match(/^at ([\w.$<>]+)\((.+):(\d+)\)$/)
    if (java) {
      frames.push(
        frame(raw, {
          fn: java[1],
          file: java[2],
          line: toNumber(java[3]),
        })
      )
      continue
    }

    const rust = trimmed.match(/^\d+:\s+(.+)$/)
    const rustAt = next?.match(/^at (.+):(\d+):(\d+)$/) ?? next?.match(/^at (.+):(\d+)$/)
    if (rust && rustAt) {
      frames.push(
        frame(`${raw}\n${lines[index + 1]}`, {
          fn: rust[1],
          file: rustAt[1],
          line: toNumber(rustAt[2]),
          col: toNumber(rustAt[3]),
        })
      )
      index += 1
      continue
    }

    const compactRust =
      trimmed.match(/^(.+?) at (.+):(\d+):(\d+)$/) ?? trimmed.match(/^(.+?) at (.+):(\d+)$/)
    if (compactRust) {
      frames.push(
        frame(raw, {
          fn: compactRust[1],
          file: compactRust[2],
          line: toNumber(compactRust[3]),
          col: toNumber(compactRust[4]),
        })
      )
      continue
    }

    const goFn = trimmed.match(/^(.+)\(\)$/)
    const goAt = next?.match(/^(.+):(\d+)(?: \+0x[0-9a-f]+)?$/i)
    if (goFn && goAt) {
      frames.push(
        frame(`${raw}\n${lines[index + 1]}`, {
          fn: goFn[1],
          file: goAt[1],
          line: toNumber(goAt[2]),
        })
      )
      index += 1
      continue
    }

    frames.push({ raw })
  }

  return frames
}

export function structuredFrameCount(frames: Frame[]): number {
  return frames.filter((item) => item.file || item.fn).length
}

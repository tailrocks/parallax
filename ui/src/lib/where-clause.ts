// Plan 164: where-clause grammar — `ident op literal (AND …)*`, AND-only in v1.
// The clause is parsed into a typed filter list; raw strings never reach SQL.

export const WHERE_OPS = [
  "=",
  "!=",
  ">=",
  "<=",
  ">",
  "<",
  "CONTAINS",
  "NOT CONTAINS",
] as const

export type WhereOp = (typeof WHERE_OPS)[number]

export type WhereFilter = {
  key: string
  op: WhereOp
  value: string
}

export type WhereParseError = {
  message: string
  /** 0-based character offset into the input where the error starts. */
  start: number
  /** 0-based exclusive end offset (for editor squiggle rendering). */
  end: number
}

export type WhereParseResult =
  | { ok: true; filters: WhereFilter[] }
  | { ok: false; error: WhereParseError }

type Token = {
  kind: "ident" | "op" | "string" | "literal" | "and"
  text: string
  /** Unquoted/unescaped value for string tokens; raw text otherwise. */
  value: string
  start: number
  end: number
}

const IDENT_RE = /^[A-Za-z_][\w.\-/]*$/
const SYMBOL_OPS = ["!=", ">=", "<=", "=", ">", "<"] as const

function isSpace(ch: string) {
  return ch === " " || ch === "\t" || ch === "\n" || ch === "\r"
}

function readQuoted(
  input: string,
  start: number,
  quote: string
): { value: string; end: number } | WhereParseError {
  let value = ""
  let i = start + 1
  while (i < input.length) {
    const ch = input[i]!
    if (ch === "\\" && i + 1 < input.length) {
      const next = input[i + 1]!
      value += next === "n" ? "\n" : next === "t" ? "\t" : next
      i += 2
      continue
    }
    if (ch === quote) return { value, end: i + 1 }
    value += ch
    i += 1
  }
  return {
    message: "unterminated string",
    start,
    end: input.length,
  }
}

export function tokenizeWhereClause(
  input: string
): { ok: true; tokens: Token[] } | { ok: false; error: WhereParseError } {
  const tokens: Token[] = []
  let i = 0
  while (i < input.length) {
    const ch = input[i]!
    if (isSpace(ch)) {
      i += 1
      continue
    }
    if (ch === '"' || ch === "'") {
      const read = readQuoted(input, i, ch)
      if ("message" in read) return { ok: false, error: read }
      tokens.push({
        kind: "string",
        text: input.slice(i, read.end),
        value: read.value,
        start: i,
        end: read.end,
      })
      i = read.end
      continue
    }
    const symbol = SYMBOL_OPS.find((op) => input.startsWith(op, i))
    if (symbol) {
      tokens.push({
        kind: "op",
        text: symbol,
        value: symbol,
        start: i,
        end: i + symbol.length,
      })
      i += symbol.length
      continue
    }
    // Bare word: ident, literal, AND, CONTAINS, NOT.
    let j = i
    while (
      j < input.length &&
      !isSpace(input[j]!) &&
      !SYMBOL_OPS.some((op) => input.startsWith(op, j)) &&
      input[j] !== '"' &&
      input[j] !== "'"
    ) {
      j += 1
    }
    const word = input.slice(i, j)
    const upper = word.toUpperCase()
    if (upper === "AND") {
      tokens.push({ kind: "and", text: word, value: "AND", start: i, end: j })
    } else if (upper === "CONTAINS") {
      tokens.push({
        kind: "op",
        text: word,
        value: "CONTAINS",
        start: i,
        end: j,
      })
    } else if (upper === "NOT") {
      // `NOT CONTAINS` merges in the parser via lookahead below.
      tokens.push({ kind: "op", text: word, value: "NOT", start: i, end: j })
    } else {
      tokens.push({
        kind: "literal",
        text: word,
        value: word,
        start: i,
        end: j,
      })
    }
    i = j
  }
  return { ok: true, tokens }
}

function mergeNotContains(tokens: Token[]): Token[] | WhereParseError {
  const out: Token[] = []
  for (let i = 0; i < tokens.length; i += 1) {
    const tok = tokens[i]!
    if (tok.kind === "op" && tok.value === "NOT") {
      const next = tokens[i + 1]
      if (!next || next.kind !== "op" || next.value !== "CONTAINS") {
        return {
          message: "expected CONTAINS after NOT",
          start: tok.start,
          end: tok.end,
        }
      }
      out.push({
        kind: "op",
        text: `${tok.text} ${next.text}`,
        value: "NOT CONTAINS",
        start: tok.start,
        end: next.end,
      })
      i += 1
      continue
    }
    out.push(tok)
  }
  return out
}

export function parseWhereClause(input: string): WhereParseResult {
  if (input.trim() === "") return { ok: true, filters: [] }
  const tokenized = tokenizeWhereClause(input)
  if (!tokenized.ok) return tokenized
  const merged = mergeNotContains(tokenized.tokens)
  if ("message" in merged) return { ok: false, error: merged }

  const filters: WhereFilter[] = []
  let i = 0
  while (i < merged.length) {
    const keyTok = merged[i]!
    if (keyTok.kind !== "literal" || !IDENT_RE.test(keyTok.value)) {
      return {
        ok: false,
        error: {
          message: "expected attribute key",
          start: keyTok.start,
          end: keyTok.end,
        },
      }
    }
    const opTok = merged[i + 1]
    if (!opTok || opTok.kind !== "op") {
      const at = opTok ?? keyTok
      return {
        ok: false,
        error: {
          message: `expected operator after "${keyTok.value}"`,
          start: opTok ? at.start : keyTok.end,
          end: opTok ? at.end : keyTok.end + 1,
        },
      }
    }
    const valueTok = merged[i + 2]
    if (
      !valueTok ||
      (valueTok.kind !== "string" && valueTok.kind !== "literal")
    ) {
      const at = valueTok ?? opTok
      return {
        ok: false,
        error: {
          message: `expected value after "${opTok.value}"`,
          start: valueTok ? at.start : opTok.end,
          end: valueTok ? at.end : opTok.end + 1,
        },
      }
    }
    filters.push({
      key: keyTok.value,
      op: opTok.value as WhereOp,
      value: valueTok.value,
    })
    i += 3
    if (i >= merged.length) break
    const andTok = merged[i]!
    if (andTok.kind !== "and") {
      return {
        ok: false,
        error: {
          message: "expected AND between conditions (v1 grammar is AND-only)",
          start: andTok.start,
          end: andTok.end,
        },
      }
    }
    i += 1
    if (i >= merged.length) {
      return {
        ok: false,
        error: {
          message: "expected condition after AND",
          start: andTok.start,
          end: andTok.end,
        },
      }
    }
  }
  return { ok: true, filters }
}

function needsQuoting(value: string): boolean {
  return value === "" || !/^[\w.\-/:]+$/.test(value)
}

export function quoteWhereValue(value: string): string {
  if (!needsQuoting(value)) return value
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`
}

export function serializeWhereClause(filters: WhereFilter[]): string {
  return filters
    .map((f) => `${f.key} ${f.op} ${quoteWhereValue(f.value)}`)
    .join(" AND ")
}

// URL codec: filters travel as the serialized clause string in a single
// search param, so permalinks stay human-readable.
export function whereClauseFromSearch(raw: string | undefined): WhereFilter[] {
  if (!raw) return []
  const parsed = parseWhereClause(raw)
  return parsed.ok ? parsed.filters : []
}

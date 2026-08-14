// Plan 164: where-clause grammar — `ident op literal (AND …)*`, AND-only in v1.
// The clause is parsed into a typed filter list; raw strings never reach SQL.

export const WHERE_OPS = ["=", "!=", ">=", "<=", ">", "<", "CONTAINS", "NOT CONTAINS"] as const

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

function isReservedKeyword(word: string): boolean {
  const upper = word.toUpperCase()
  return upper === "AND" || upper === "CONTAINS" || upper === "NOT"
}

/** Key position: a literal ident, or a reserved word used as an attribute name. */
function keyFromToken(tok: Token): string | null {
  if (tok.kind === "literal" && IDENT_RE.test(tok.value)) return tok.value
  if ((tok.kind === "op" || tok.kind === "and") && IDENT_RE.test(tok.text)) {
    return tok.text
  }
  return null
}

/**
 * Value position: quoted string, bare literal, or a reserved word.
 * Keyword tokens keep `.text` so `nOt` stays `nOt`, not the uppercased `.value`.
 */
function valueFromToken(tok: Token): string | null {
  if (tok.kind === "string" || tok.kind === "literal") return tok.value
  if (tok.kind === "op" || tok.kind === "and") return tok.text
  return null
}

function err(message: string, start: number, end: number): WhereParseError {
  return { message, start, end }
}

function readKey(tok: Token): { ok: true; key: string } | { ok: false; error: WhereParseError } {
  const key = keyFromToken(tok)
  if (key === null) {
    return { ok: false, error: err("expected attribute key", tok.start, tok.end) }
  }
  return { ok: true, key }
}

function readOperator(
  tokens: Token[],
  index: number,
  key: string,
  keyTok: Token
): { ok: true; op: WhereOp; next: number; end: number } | { ok: false; error: WhereParseError } {
  const opTok = tokens[index]
  if (!opTok || opTok.kind !== "op") {
    const at = opTok ?? keyTok
    return {
      ok: false,
      error: err(
        `expected operator after "${key}"`,
        opTok ? at.start : keyTok.end,
        opTok ? at.end : keyTok.end + 1
      ),
    }
  }
  if (opTok.value === "NOT") {
    const containsTok = tokens[index + 1]
    if (!containsTok || containsTok.kind !== "op" || containsTok.value !== "CONTAINS") {
      return {
        ok: false,
        error: err("expected CONTAINS after NOT", opTok.start, opTok.end),
      }
    }
    return { ok: true, op: "NOT CONTAINS", next: index + 2, end: containsTok.end }
  }
  return { ok: true, op: opTok.value as WhereOp, next: index + 1, end: opTok.end }
}

function readValue(
  tokens: Token[],
  index: number,
  op: WhereOp,
  opEnd: number,
  opTok: Token
): { ok: true; value: string; next: number } | { ok: false; error: WhereParseError } {
  const valueTok = tokens[index]
  const value = valueTok ? valueFromToken(valueTok) : null
  if (value === null) {
    const at = valueTok ?? opTok
    return {
      ok: false,
      error: err(
        `expected value after "${op}"`,
        valueTok ? at.start : opEnd,
        valueTok ? at.end : opEnd + 1
      ),
    }
  }
  return { ok: true, value, next: index + 1 }
}

export function parseWhereClause(input: string): WhereParseResult {
  if (input.trim() === "") return { ok: true, filters: [] }
  const tokenized = tokenizeWhereClause(input)
  if (!tokenized.ok) return tokenized
  const tokens = tokenized.tokens

  const filters: WhereFilter[] = []
  let i = 0
  while (i < tokens.length) {
    const keyTok = tokens[i]!
    const keyRead = readKey(keyTok)
    if (!keyRead.ok) return keyRead
    const opRead = readOperator(tokens, i + 1, keyRead.key, keyTok)
    if (!opRead.ok) return opRead
    const valueRead = readValue(tokens, opRead.next, opRead.op, opRead.end, tokens[i + 1] ?? keyTok)
    if (!valueRead.ok) return valueRead
    filters.push({ key: keyRead.key, op: opRead.op, value: valueRead.value })
    i = valueRead.next
    if (i >= tokens.length) break
    const andTok = tokens[i]!
    if (andTok.kind !== "and") {
      return {
        ok: false,
        error: err(
          "expected AND between conditions (v1 grammar is AND-only)",
          andTok.start,
          andTok.end
        ),
      }
    }
    i += 1
    if (i >= tokens.length) {
      return {
        ok: false,
        error: err("expected condition after AND", andTok.start, andTok.end),
      }
    }
  }
  return { ok: true, filters }
}

function needsQuoting(value: string): boolean {
  return value === "" || !/^[\w.\-/:]+$/.test(value) || isReservedKeyword(value)
}

export function quoteWhereValue(value: string): string {
  if (!needsQuoting(value)) return value
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`
}

export function serializeWhereClause(filters: WhereFilter[]): string {
  return filters.map((f) => `${f.key} ${f.op} ${quoteWhereValue(f.value)}`).join(" AND ")
}

// URL codec: filters travel as the serialized clause string in a single
// search param, so permalinks stay human-readable.
export function whereClauseFromSearch(raw: string | undefined): WhereFilter[] {
  if (!raw) return []
  const parsed = parseWhereClause(raw)
  return parsed.ok ? parsed.filters : []
}

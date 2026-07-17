export type IdGuess =
  | { kind: "trace"; id: string }
  | { kind: "span-in-trace"; id: string }
  | { kind: "invocation"; id: string }
  | { kind: "fingerprint"; id: string }

const TRACE_ID = /^[0-9a-f]{32}$/
const HEX_16 = /^[0-9a-f]{16}$/
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/
const INVOCATION_ID = /^[a-z0-9][a-z0-9._:-]{1,127}$/

export function guessId(input: string): IdGuess[] {
  const id = input.trim().toLowerCase()
  if (!id) return []

  if (UUID.test(id)) {
    return [{ kind: "invocation", id }]
  }

  if (TRACE_ID.test(id)) {
    return [{ kind: "trace", id }]
  }

  if (HEX_16.test(id)) {
    return [
      { kind: "invocation", id },
      { kind: "fingerprint", id },
      { kind: "span-in-trace", id },
    ]
  }

  if (INVOCATION_ID.test(id) && (id.startsWith("run") || /[._:-]/.test(id))) {
    return [{ kind: "invocation", id }]
  }

  return []
}

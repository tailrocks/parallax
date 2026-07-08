export type IdGuess =
  | { kind: "trace"; id: string }
  | { kind: "span-in-trace"; id: string }
  | { kind: "run"; id: string }
  | { kind: "fingerprint"; id: string }

const TRACE_ID = /^[0-9a-f]{32}$/
const HEX_16 = /^[0-9a-f]{16}$/
const RUN_ID = /^[a-z0-9][a-z0-9._:-]{1,127}$/

export function guessId(input: string): IdGuess[] {
  const id = input.trim().toLowerCase()
  if (!id) return []

  if (TRACE_ID.test(id)) {
    return [{ kind: "trace", id }]
  }

  if (HEX_16.test(id)) {
    return [
      { kind: "run", id },
      { kind: "fingerprint", id },
      { kind: "span-in-trace", id },
    ]
  }

  if (RUN_ID.test(id) && (id.startsWith("run") || /[._:-]/.test(id))) {
    return [{ kind: "run", id }]
  }

  return []
}

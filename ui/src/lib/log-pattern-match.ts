/** Log pattern expand helpers (plan 165 Patterns view).
 *
 * Given a Drain template with `<*>` wildcards (whole tokens or embedded in a
 * token), decide whether a raw log body matches. Pure — no GraphQL.
 *
 * Preliminary — peer wires Patterns toggle + sample query.
 */

export const LOG_WILDCARD = "<*>"

/**
 * Convert a Drain template into a case-sensitive regex.
 * - Whole-token `<*>` matches one non-space token (`\S+`).
 * - Embedded `<*>` inside a token (e.g. `handler-<*>`) matches a non-empty
 *   non-space run for that segment.
 */
export function templateToRegExp(template: string): RegExp {
  const parts = template.split(/\s+/).filter(Boolean)
  if (parts.length === 0) {
    return /^$/
  }
  const escaped = parts.map((tok) => {
    if (tok === LOG_WILDCARD) {
      return "\\S+"
    }
    // Split on wildcards inside the token; escape fixed pieces.
    const segments = tok.split(LOG_WILDCARD)
    return segments
      .map((seg) => seg.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
      .join("\\S+")
  })
  return new RegExp(`^${escaped.join("\\s+")}$`)
}

/** True when `body` matches the Drain `template` token pattern. */
export function bodyMatchesTemplate(body: string, template: string): boolean {
  const normalized = body.trim().replace(/\s+/g, " ")
  return templateToRegExp(template).test(normalized)
}

/**
 * Build a CONTAINS-friendly stable fragment from the template: longest
 * contiguous run of tokens with no wildcards (whole or embedded).
 * Returns null when no such run exists.
 */
export function templateStableFragment(template: string): string | null {
  const tokens = template.split(/\s+/).filter(Boolean)
  if (tokens.length === 0) return null
  let best: string[] = []
  let run: string[] = []
  for (const t of tokens) {
    if (t === LOG_WILDCARD || t.includes(LOG_WILDCARD)) {
      if (run.length > best.length) best = run
      run = []
    } else {
      run.push(t)
    }
  }
  if (run.length > best.length) best = run
  return best.length > 0 ? best.join(" ") : null
}

/** Filter a list of bodies to those matching a template (stable order). */
export function filterBodiesByTemplate(
  bodies: readonly string[],
  template: string
): string[] {
  return bodies.filter((b) => bodyMatchesTemplate(b, template))
}

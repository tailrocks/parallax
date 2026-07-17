/**
 * Exact accessibility exception registry (plan 146).
 * Each exception: rule, locator/state, owner, reason, created, expiry, removal.
 * Empty by default — missing/stale exceptions must fail policy when present.
 */
export interface AccessibilityException {
  rule: string
  locator: string
  state: string
  owner: string
  reason: string
  created: string
  expiry: string
  removalCondition: string
}

export const accessibilityExceptions: AccessibilityException[] = []

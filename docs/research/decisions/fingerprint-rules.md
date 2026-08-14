# Decision proposal: user-steerable fingerprint rules

**Status:** OPERATOR-GATED — proposal only. No product code until approved.
**Date:** 2026-08-14
**Plan:** [176](../../../plans/176-grouping-transparency.md) Step 4

## Proposal

Per-service, declarative grouping overrides stored in Turso and applied
**forward-only** at derivation time.

```text
match:
  error_type: TypeError
  message_template: "connection to <host> refused"
then:
  split-by: service.instance.id
```

Rules are ordered. First match wins. A change never rewrites existing
issue fingerprints. `groupingExplanation.matchedRule` names the rule so
steering stays visible.

## Bounds

- Max N rules per service (suggested 32).
- Rule name required, unique per service.
- No regex on raw messages — match the already-normalized template.

## Why gated

Grouping is issue identity. Sentry's opaque regrouping is the failure
this plan exists to avoid. Approval is the same class of gate as plan
171's MCP issue-lease catalog.

## Not in this proposal

Merge/split of historical issues. Default algorithm changes.

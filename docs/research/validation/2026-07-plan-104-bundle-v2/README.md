# Plan 104 — Evidence bundle contract (Option C)

**Status:** DONE (2026-07-17)

## Decision

Option C approved (`docs/research/decisions/evidence-bundle-contract.md`):
`bundle-v2` CloudEvents-profile envelope around the immutable `bundle-v1`
dossier. Permanent v1 readability; fail-closed unknown versions.

## Implementation

| Surface | Behavior |
|---|---|
| `parallax-evidence` | `EnvelopeV2`, `envelope_v1`, `document_version`, JCS hash, schema |
| GraphQL `bundle` | Emits v2 envelope JSON by default (project=`local`, window derived) |
| Markdown projection | Still from v1 dossier body (agent-facing) |
| Tests | 39 evidence tests green (v1 + v2 + conversion + fail-closed) |

## Verify

```sh
cargo nextest run --locked -p parallax-evidence
```

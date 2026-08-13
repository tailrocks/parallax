# Grouping

Parallax groups error events into issues with a deterministic fingerprint
(`fp-v1`). The issue-detail **Grouped by** card is that explanation.

## Inputs

| Input | Source | Test |
| --- | --- | --- |
| `error.type` | Structured exception type, else a log/sentry fallback | `different_types_do_not_group` |
| Message | Normalized before hashing | `volatile_tokens_group_together`, `distinct_messages_without_volatile_tokens_do_not_group` |
| Top stack frame | First line; line numbers and deep paths collapse | `frame_line_numbers_do_not_split` |
| Operation | Optional `jackin.operation` (or equivalent) | `operation_partitions_same_error_message` |

Hash = first 16 hex chars of SHA-256 over those fields, NUL-separated.
`fingerprint_explained` returns the same hash as `fingerprint` /
`fingerprint_with_operation` (`explained_hash_matches_fingerprint_bytes`).

## Normalization (`normalize_message`)

UUIDs → `<uuid>`, long hex → `<hex>`, short hex that contains a digit →
`<hex>`, `jk-…` slugs → `<container>`, `uid:gid` → `<uid>`, remaining
digits → `<n>`, ANSI color stripped. Prose hex words without digits stay
(`prose_hex_words_do_not_normalize`). The pass is a fixpoint
(`normalize_message_is_idempotent`).

## What you can do today

Shape `error.type` and the message so the template is stable. Follow
[conventions](conventions.md) for exception encodings. There is no
user-defined regrouping yet — that is the gated
[fingerprint-rules](../research/decisions/fingerprint-rules.md) proposal.

`algorithmVersion` is `fp-v1`. A future normalization change must bump it
and keep old explanations labeled with the version that produced them.

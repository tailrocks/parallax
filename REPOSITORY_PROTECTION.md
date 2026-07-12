# Repository Protection Policy

Status: **public repository**. Its contents are licensed under the
[Apache License, Version 2.0](LICENSE). Any lawfully received copy receives the
rights granted by that license, so repository protection centers on publication
quality, least privilege, supply-chain integrity, and prompt incident response.

This file records standing policy, not an implementation checklist. Unfinished
repository-security and required-check automation is owned only by
[plan 094](plans/094-ci-and-security-foundation.md). Dependency, license, source,
and advisory hygiene is owned only by
[plan 101](plans/101-dependencies-nextest-and-hygiene.md).

## Standing Requirements

- Repository ownership, metadata, license, topics, and links remain accurate.
- Strategy notes, customer information, private benchmarks, operating material,
  and sensitive research receive publication-suitability review before landing.
- Third-party citations, datasets, screenshots, and copied excerpts retain the
  required rights and attribution.
- Package metadata, schemas, examples, and source headers that declare a license
  use `Apache-2.0`; canonical attribution remains in `NOTICE`.
- Organization and repository administration use least privilege and two-factor
  authentication.
- `main` retains the active repository ruleset: no deletion or force-push,
  one approval from someone other than the last pusher, resolved review threads,
  and strict required `ci-required` and `DCO` checks.
- Organization administrators retain an explicit always-bypass solely for
  operator-authorized repository operations, including the current research-
  stage main-first workflow. Normal contributions do not use bypass.
- Secret scanning, dependency/advisory scanning, and license/source policy stay
  enabled through the repository-owned gates.
- Issue, discussion, project, disclosure, embargo, and feedback settings remain
  aligned with the repository's public posture.
- Accidental publication of secrets or non-public material is an incident and is
  remediated immediately; history rewriting still requires explicit operator
  authorization.
- Major external-facing licensing, terms, or data-handling changes receive the
  appropriate legal/compliance review.

Implementation status and completion evidence live in the numbered plans and
their durable validation artifacts. This policy does not duplicate that queue.

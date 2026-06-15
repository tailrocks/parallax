# Repository Protection Checklist

This repository is now publicly visible. Its contents are licensed under the
[Apache License, Version 2.0](LICENSE). The checklist now tracks launch
governance, external publication hygiene, and legal/attribution compliance.

Note the consequence of Apache-2.0: any lawfully received copy receives the
license rights to it. Distribution policy now centers on publication quality,
not access-control restrictions.

Before public release:

1. Confirm ownership is `tailrocks`, and repository metadata (license, topics,
   links) is accurate.
2. Review unpublished strategy notes, customer information, private benchmarks,
   operating plans, and sensitive research assets for publication suitability.
3. Re-check third-party citations, datasets, screenshots, and copied excerpts
   for rights and attribution.
4. Verify package metadata and source headers consistently declare
   `license = "Apache-2.0"`.
5. Define external review expectations: disclosures, embargoes, and feedback
   handling.

Recommended GitHub settings:

1. Keep repository ownership and admin access minimal.
2. Require branch protection and review requirements on `main`.
3. Enable two-factor authentication for organization members.
4. Enable secret scanning and Dependabot alerts.
5. Keep issue/discussion/project settings aligned with current launch posture.

After publication:

1. Monitor dependency and license scans, and security alerts.
2. Keep governance and attribution docs (README/NOTICE/AGENTS) up to date.
3. Immediately remediate any accidental publication of secrets or non-public
   material.
4. Have legal/compliance review major external-facing disclosures when
   licensing, terms, or data-handling guidance changes.

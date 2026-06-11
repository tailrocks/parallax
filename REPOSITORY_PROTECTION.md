# Repository Protection Checklist

This repository is currently **private** Tailrocks Pte. Ltd. research. Its
contents are licensed under the [Apache License, Version 2.0](LICENSE)
(operator decision, 2026-06-12) — but a license is not a publication: until
Tailrocks publishes the repository, access control still decides who receives
a copy, and this checklist governs that access.

Note the consequence of the Apache-2.0 grant: anyone lawfully given a copy
receives the license rights to it. Pre-release access should therefore be
shared deliberately — the controls below protect launch timing and
distribution, not a proprietary claim over the content.

Before sharing access outside Tailrocks Pte. Ltd.:

1. Confirm the repository is owned by the `tailrocks` GitHub account or
   organization.
2. Keep repository visibility private.
3. Give each external reviewer the minimum GitHub permission needed, usually
   read-only access.
4. Use a written reviewer agreement before granting pre-release access; under
   Apache-2.0 it cannot revoke license rights to delivered copies, but it can
   cover non-disclosure of unpublished plans, no pre-launch republication,
   permitted purpose, and feedback handling.
5. Avoid sharing access with anyone whose employer, client, or publication
   obligations could conflict with the pre-release timeline.
6. Remove access immediately after the review period ends.

Recommended GitHub settings:

1. Disable private forking unless Tailrocks explicitly needs it.
2. Require two-factor authentication for the organization.
3. Protect `main` with pull-request review requirements before adding
   collaborators with write access.
4. Require CODEOWNERS review when branch protection is enabled.
5. Disable GitHub Pages unless intentionally publishing a public site.
6. Enable secret scanning and Dependabot alerts where available for the plan.
7. Keep issues, discussions, projects, and wikis disabled unless Tailrocks needs
   them for private collaboration.

Before making any part public:

1. Decide which files are being published and which remain private.
2. Split unpublished strategy, market research, prompts, customer notes,
   benchmark logs, interview notes, and operating plans into a private
   repository if they should not ship with the public release (the license is
   already Apache-2.0 either way; this is a curation step, not a relicensing
   step).
3. Re-check third-party citations, datasets, screenshots, and copied excerpts
   for license and publication rights.
4. Have Singapore counsel review the publication package and external terms.


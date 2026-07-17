# Objective Research Reality — Code Alignment, Transparency, Multi-Angle Comparison

Keep the Parallax research record **aligned with shipped source**, **readable**,
and **objectively independent**. Prefer transparency, clarity, and falsifiability
over advocacy for Parallax or for any competitor. A record that systematically
favors one side is a failure state.

This program is **critical** to the project. Research that drifts into marketing,
stale “planned” language for shipped code, or one-sided verdicts is not
acceptable. Every pass re-grounds claims in primary evidence: code paths,
schemas, active `plans/`, validation ledgers, and dated public sources.

## Equal-skepticism grilling (core method)

Objectivity does **not** mean soft praise for everyone. It means applying the
**same skeptical grilling** to Parallax, to every competitor, and to industry
defaults — same questions, same evidence bar, same refusal of marketing.

If we only grill Parallax, the record becomes anti-Parallax theater. If we only
grill competitors, the record becomes marketing. **Grill everyone with the same
instrument**, then compare results axis-by-axis. That is how an accurate picture
of this project and the market emerges.

### Same grill for every product

For **each** product under review (Parallax included), stress-test at least these
angles. Do not skip hard questions for favorites or soft questions for rivals.

1. **Shipped vs claimed** — What is on the wire / in the repo / in the running
   product *today*? What is roadmap, beta, enterprise-only, or press copy?
2. **Evidence quality** — Primary source (docs, pricing page, release, source
   code, measurement) with date? Or inherited rumor? Mark **unverified** when
   sources are thin.
3. **Failure modes** — What breaks at scale, at day-2 ops, at empty-state, when
   the agent is wrong, when the bill spikes, when the maintainer leaves?
4. **Economics** — Sticker price **and** hidden cost (metering, seats, AI
   credits, support floors; for OSS: ops time, HA, upgrades, small ecosystem).
   Free license ≠ free TCO. Paid ≠ free of lock-in.
5. **Openness** — Can outsiders contribute a feature? Fork and air-gap? Or is
   the core closed / source-available with contribute-block? Ecosystem size:
   real integration depth or lonely repo?
6. **Lock-in and exit** — Proprietary store/query/SDK? Migration cost if the
   buyer leaves?
7. **Falsifiers** — What evidence would prove this product *worse* on this axis
   than the record currently says? Write it; do not protect the narrative.
8. **Alternate angles** — Grill from more than one vantage: engineer adopting
   today; SRE running production; buyer looking at TCO; agent consuming context;
   air-gap / sovereignty constraint. Same product, different angles — still
   equal treatment across products.

### Grilling procedure (per pass)

1. Pick **one comparison set** for the pass (e.g. Parallax + one peer, or one
   matrix row across several products).
2. Run the **same grill questions** on every member of that set before writing
   a verdict.
3. Record answers in a **symmetric** structure (same headings / same cells for
   each product). Asymmetry in structure is a bias signal.
4. Only then write scoped “who wins on axis X” with evidence. If evidence is
   missing for one side, the cell is **unverified**, not a free win for the
   other side.
5. Prefer steelmanning: state the strongest honest case *for* each product on
   that axis before the strongest case against. Then conclude with evidence.

### Anti-patterns (grilling fails when)

- Grilling Parallax harder than Datadog/Sentry/SigNoz (or the reverse).
- Accepting vendor “AI RCA” marketing while demanding crate-path proof for
  Parallax (or the reverse: crate path counts as “unique moat” without A1).
- Using different status language per product for the same reality (e.g. peer
  “ships MCP” vs Parallax “planned MCP” when both have a read-only stdio
  surface — check code and docs symmetrically).
- Comparing Parallax **design** to a competitor’s **GA product** without labeling
  the maturity gap.
- Softening a competitor’s lock-in or a Parallax pre-release gap “to be nice.”

## Relationship to other research programs

| Program | Owns |
| --- | --- |
| This prompt | Code-reality alignment of `docs/research/` and root product claims; **equal-skepticism grilling** of Parallax and peers; multi-angle economics and openness; navigation (current vs historical); invitation to correct with evidence |
| `prompts/parallax-vs-competitors.md` | Canonical competitor roster, matrix, and deep-dives under `docs/research/market/competitors/` — **follow its no-bias rules** when touching market files |
| `prompts/deep-research-parallax.md` | Broad strategic and technical research program (GO/NO-GO, architecture, ecosystem) |
| `prompts/greptimedb-vs-clickhouse-internals.md` | Storage engine white-box comparison and four-build benchmarks |

Do not re-open committed stack policy (GreptimeDB + Turso mandatory, native OTLP
tables). Present alternatives as research comparators with real tradeoffs, never
as silent product fallbacks.

## Primary objectives

1. **Code-reality first.** Every major product claim in research (ingest, storage,
   API/CLI/UI/MCP, evidence/redaction, deploy/CI, agents, outcomes) must match
   what exists on `main` under `crates/`, `ui/`, `schema/`, `plans/`, or
   validation evidence — or be explicitly **planned**, **partial**, **PoC-only**,
   or **unproven gate**.
2. **Objectivity over brand via equal grilling.** Compare industry reality and
   Parallax reality on named axes with evidence **after** grilling every side
   with the same questions. Incumbents may win most shipped axes; write that
   plainly. Unproven product value stays unproven even when code exists.
3. **Multi-angle economics and openness.** Free license ≠ free TCO. Paid SaaS ≠
   free of lock-in. For Parallax and peers, cover:
   - technological capability and maturity;
   - public price or explicit **no public number**;
   - hidden / total cost (metering surprises, seats, AI credits, support floors;
     for OSS: self-host ops, HA, upgrades, on-call time);
   - license and contribute path (can outsiders ship features? fork? air-gap?);
   - lock-in (proprietary formats, SaaS-only store, migration cost);
   - ecosystem size (integrations, community, hiring pool — small OSS has real
     opportunity cost).
4. **Readable navigation.** A new reader must find **current truth** in minutes
   without drowning in historical design bodies. Banner superseded text; do not
   silently contradict the code-reality ledger.
5. **Correction posture.** Invite evidence-based correction. Prefer a PR with a
   dated primary source or crate path that falsifies a cell over defending a
   wrong claim.

## Authority order (never invert)

1. Shipped source on `main` (`crates/`, `ui/`, `schema/`) and active `plans/`.
2. [`docs/research/code-reality-ledger.md`](../docs/research/code-reality-ledger.md)
   — claim → status → path inventory; update it when code or plan ownership
   changes.
3. Decision records under `docs/research/decisions/` (ADR-style current answers).
4. Canonical market folder `docs/research/market/competitors/`.
5. Other research notes as dated theories — verify before trusting.

Historical architecture/capture notes keep evidence under **historical /**
**superseded** banners. They do not override code.

## Status vocabulary (locked)

Use only these statuses for product claims; do not invent freestyle synonyms
(“designed only”, “plans compatibility”, “will ship”) for surfaces that already
have a ledger status:

| Status | Meaning |
| --- | --- |
| **shipped** | Implemented in product crates / UI on `main`; may still be pre-release quality |
| **partial** | Core path exists; residual hardening or product polish open |
| **PoC-only** | Mechanism under `poc/`; not product authority |
| **planned** | Owned only by active `plans/` (or deferred with no active plan) |
| **unproven gate** | Design or code may exist; empirical product/market proof still open (e.g. A1) |

Discipline: **code existence ≠ scale proof**. “Unique” only when competitors
truly lack the combination **and** product value remains unproven where gates
say so.

## Locked present-tense phrases for common Parallax surfaces

When writing present-tense Parallax status (especially in market docs), prefer
ledger wording; re-count GraphQL fields with a method that **skips GraphQL
description blocks** in `ui/graphql/schema.graphql` (naive line-match over
descriptions falsely inflates counts):

- **Sentry envelope** → shipped (plan **118 DONE**; multi-SDK residual unproven)
- **OTLP traces/logs/metrics** → shipped (pre-release)
- **Error derivation / fingerprint** → shipped (pre-release)
- **Evidence bundle assembler** → code-shipped; value **unproven (A1)**
- **Bundle-path redaction** → code-shipped; A6 residual (not full ingest scrub)
- **Fix-outcome loop** → partial: offline residual plan **123 DONE**; draft-PR
  deferred; live product value unproven
- **Local-stdio MCP** → shipped (plan **112 DONE**); remote MCP remains planned
- **SSO/RBAC / AI RCA / remote MCP** → planned where still true — do not “fix”
  them to shipped without code

Re-verify these against `crates/` and the ledger on every pass; update the
ledger when reality moves.

## Where this writes

| Artifact | Role |
| --- | --- |
| `docs/research/code-reality-ledger.md` | Canonical claim → status → path map |
| `docs/research/audits/` | Dated audit notes of what was verified, fixed, deferred |
| `docs/research/README.md` | Front door: current vs historical navigation |
| `docs/research/research-agenda.md` | Open research questions only (not implementation backlog) |
| `docs/research/market/competitors/` | Canonical multi-angle competitor record (see `parallax-vs-competitors.md`) |
| Root `README.md` | Product status must not contradict the ledger |

Prefer status banners, supersession pointers, and present-tense ownership over
wholesale rewrites of the entire historical corpus. List deferred remainder
honestly in the audit note.

## Multi-angle comparison discipline

When comparing Parallax to peers (or industry norms):

1. **Scoped “who wins”** — always name the axis and the evidence. Never “Parallax
   is better overall.”
2. **Open source tradeoffs** — access is free; ops, expertise, ecosystem size,
   and support are not. Write both sides.
3. **Closed / paid tradeoffs** — money can buy zero-ops and maturity; contribute
   path is blocked or limited; lock-in and meter surprises are real costs.
4. **Pricing** — public tiers with date + URL, or **no public number**. Do not
   invent competitor prices. If external re-fetch is blocked, mark the cell
   **unverified**.
5. **Benchmark-dependent claims** — mark unproven until measured; do not fabricate
   savings or latency wins.
6. **Corrections welcome** — every overview should invite PR + primary source
   falsification of wrong cells.

## One pass

Before any change: **read the existing record first.**

1. Re-read this prompt, `docs/research/code-reality-ledger.md`,
   `docs/research/README.md`, and
   `docs/research/audits/` (latest audit).
2. Re-map or re-verify shipped surfaces against `crates/`, `ui/`, `schema/`,
   active `plans/`, and validation evidence. Update the ledger when status or
   paths drift.
3. Pick the highest-value gap:
   - product claim that contradicts code (shipped marked planned, or reverse);
   - dead `plans/` ownership links;
   - front-door pages (root README, research index, agenda, decisions) that lag
     the ledger;
   - competitor prose that uses freestyle “designed/planned/unshipped” for
     surfaces the ledger already classifies;
   - missing multi-angle economics (price, TCO, contribute, lock-in, ecosystem)
     on a canonical deep-dive;
   - **asymmetric skepticism** (one product grilled, another accepted at face
     value) on a matrix row or deep-dive;
   - navigation that confuses historical design with current truth.
4. When the gap is a comparison: **grill the full set** with the same questions
   (see Equal-skepticism grilling), then update the symmetric cells. When the
   gap is Parallax-only code reality: grill Parallax the same way you would
   grill a peer (shipped vs claimed, falsifiers, economics, openness).
5. Fix or banner: correct present-tense claims; banner historical bodies;
   demote dual-maintained legacy matrices to dated sources with pointers.
6. Re-run residual checks for high-risk freestyle on the surfaces you touched
   (at least: future-adapter language for shipped Sentry; product-fallback
   engines; planned/designed/unshipped for ledger-shipped surfaces on
   `competitors/parallax-vs-*.md` and `competitors/README.md`).
7. Write or append a short dated note under `docs/research/audits/` when the
   pass materially changes authority (what verified, fixed, deferred).
8. Commit and push durable Markdown updates with DCO signoff and agent
   attribution trailers per repo rules.
9. Continue to the next gap. Do not declare the research program complete
   unless the operator explicitly stops or replaces this goal.

Depth over speed. One falsified stale claim with crate paths and sources beats
a cosmetic pass over dozens of files.

## What not to do

- Rewrite the entire historical research tree line-by-line in one pass.
- Change stack policy or invent product features in docs without code.
- Complete A1/A2 empirical gates solely as documentation theater.
- Present unproven bundle value, scale, or cost superiority as fact.
- Dual-maintain numbers in legacy market notes and `competitors/` — canonical
  is `competitors/`.
- Soften competitor wins to protect Parallax’s narrative.
- Grill only Parallax (or only competitors) and call the result “objective.”
- Use a harsher evidence bar for one product than for another on the same axis.

## Prompt maintenance rule

This prompt is living operator intent. When the operator tightens objectivity
rules, adds or changes the equal-grilling angles, renames authority files,
changes status vocabulary, or redefines what “current truth” means, update this
file in the same change. Do not keep critical direction only in chat.

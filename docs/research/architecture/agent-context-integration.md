# Agent Context Integration — Delivery Surface and Repo-Intent Linkage

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-29

## Purpose

Answers two questions the record under-specified (menu items C + D): **how do real 2026 coding agents
actually ingest external context**, so the bundle is consumable in practice, and **how should the
bundle link to repo-held intent** (the other half of the Parallax thesis —
[validation/repo-intent.md](../validation/repo-intent.md) asks *whether* repo intent adds value; this
asks *how* it is delivered/linked). Complements the
[agent access surface](../decisions/agent-access-surface.md) decision (CLI/HTTP/MCP) with how those
surfaces map to real agent consumption, and the [evidence-bundle schema](evidence-bundle-schema.md).

> **Conclusions.** (1) **Delivery matches Parallax's plan and validates "bounded":** ship the bundle as
> an **MCP server** — tools returning **`structuredContent` against a published `outputSchema`**, *plus*
> the same JSON mirrored in a text block (spec-recommended for client compatibility) — expose bundles as
> **`@`-mentionable resources** and ship investigation **prompts (slash commands)**; keep the **CLI +
> markdown projection** as the universal fallback (not legacy — the lowest common denominator agents
> still read). (2) **Bounded is mandatory, not a nicety:** Claude Code warns at **10K tokens** of MCP
> output and truncates/persists to disk past **~25K** (`MAX_MCP_OUTPUT_TOKENS`); Anthropic's own guidance
> is filter-before-return (a 150K→2K example). So the bundle must be **summarized + paginated +
> `resource_link` to full detail**, never a raw dump — exactly the A1 "bounded bundle vs raw dump"
> thesis, now with hard numbers. (3) **For repo intent, REFERENCE — do not invent a format:** point at
> existing files **by path + commit SHA** (AGENTS.md, spec-kit/OpenSpec artifacts, ADRs, PR/issue IDs),
> and carry commit/PR identity with **OpenTelemetry VCS attributes** (`vcs.ref.head.revision`,
> `vcs.change.id`) as standard join keys. A Parallax-specific intent *format* would fight the converging
> AGENTS.md/SDD ecosystem; the only thing worth inventing is a **thin pointer schema**
> (evidence → {file path, commit, PR, ADR id}). (4) **The genuinely unsolved, defensible piece** is the
> **evidence → "violates documented intent X" edge** — no standard exists for it (suspect-commits, OTel
> VCS, CODEOWNERS only reach a commit/PR, not "this breaks decision X").

## 1. How agents ingest context (delivery surface)

- **MCP is the dominant cross-tool channel**, three primitives: **Tools** (model-invoked), **Resources**
  (read-only, `@`-mentionable), **Prompts** (become slash commands). A tool may return unstructured
  `content[]` (text/image/`resource_link`/embedded resource) and/or **`structuredContent`** validated
  against an optional `outputSchema` (JSON Schema 2020-12); the spec says a tool returning
  `structuredContent` **SHOULD also emit the JSON as a text block** (compatibility). `structuredContent`
  landed in **MCP 2025-06-18**; **latest stable = 2025-11-25** (Tasks/async, OAuth); **2026-07-28 is a
  Release Candidate, not stable** — build to **2025-11-25**.
- **Per-agent reality:** **Claude Code** — MCP results >10K tokens warn; default cap **25K**
  (`MAX_MCP_OUTPUT_TOKENS`, server ceiling 500K chars via `_meta`); large results persisted to disk +
  replaced with a file reference; **resources `@`-mentioned and auto-attached**; **prompts →
  `/mcp__server__prompt`**; **Tool Search on by default** (tool defs deferred). **Codex** folds AGENTS.md
  into the system prompt (32 KiB cap), MCP tools separate. **Cursor** = workspace index + `.cursor/rules`
  + `@`-mentions + MCP. **Amp** = MCP + lightweight "Toolboxes." Notably Claude Code's MCP docs use
  "**which deployment introduced these new errors?**" (a Sentry-style query) as a worked example — exactly
  Parallax's job.
- **Token budget is the binding constraint:** loading all tool defs + piping raw results can hit
  hundreds of thousands of tokens; Anthropic shows a **150,000→2,000 token (98.7%)** reduction by
  filtering in code. Agents want **filtered, bounded, on-demand** context, not a blob.

## 2. Repo-intent linkage — reference, don't invent

- **AGENTS.md is the converging cross-tool instruction standard** (plain Markdown, nearest-file-wins,
  **60,000+ repos**, read by Codex/Cursor/Amp/Copilot/Gemini-CLI/Windsurf/Zed/Devin/Junie/…; Claude Code
  still uses `CLAUDE.md` but ingests AGENTS.md via `/init`). Spec-driven-dev formats carry the *what+why*:
  **GitHub Spec Kit** (Spec→Plan→Tasks→Implement Markdown, v0.x — early) and **OpenSpec 1.0** (delta specs
  ADDED/MODIFIED/REMOVED + **ADR-alongside-specs**). **ADRs** (MADR Markdown) remain the human decision
  standard. `llms.txt` is a docs-*website* convention, not an in-repo intent format — marginal here.
- **Implication:** Parallax should **not invent a repo-intent format** — it would fight a settling
  ecosystem. Reference existing files **by path + commit SHA**, and use **OTel VCS attributes**
  (`vcs.ref.head.revision`, `vcs.change.id`) so "which commit/PR" is a standard join key the evidence
  already carries. This is also an A7 scope-discipline win (less to build).

## 3. The unsolved, defensible edge: evidence → violated intent

Today's conventions get you only to a **commit/PR/file**: Sentry **suspect-commits** (SCM blame →
likely-introducing commit + author), **OTel VCS/CI-CD** attributes (commit/PR/deploy identity),
**CODEOWNERS** (ownership). **None** standardizes a machine-readable edge "**this failing behavior
violates THIS documented decision/spec.**" OpenSpec's ADR-alongside pattern is the only emerging
structure co-locating *why* with *what*, and it is one project's schema. So the first hop
(evidence → commit/PR/file/line + intent-doc paths) is buildable now; the **semantic
evidence→violated-intent link is the genuinely novel piece Parallax would define** — and a candidate
moat element alongside the outcome corpus, *if* A1 shows intent linkage adds fix value
([repo-intent.md](../validation/repo-intent.md) C1-vs-C0 arms).

## 4. Decisions for Parallax

- **Bundle delivery = MCP (`structuredContent` + `outputSchema` + mirrored text) + `@`-resources +
  investigation prompts, with CLI + markdown projection as the universal fallback.** Reinforces the
  [agent access surface](../decisions/agent-access-surface.md) decision; build to **MCP 2025-11-25 stable**.
- **Bundle must be bounded/paginated with `resource_link` to detail** (<~10K-token default target) — a
  hard [evidence-bundle schema](evidence-bundle-schema.md) requirement and a direct corollary of the A1
  "bounded vs raw dump" thesis ([runtime-dependence-and-raw-baseline.md](../validation/a1-bundle-value/runtime-dependence-and-raw-baseline.md)).
- **Repo intent: reference by path+commit via OTel VCS attrs; invent only the thin evidence→intent
  pointer schema** (and, later, the evidence→violated-intent edge if A1/repo-intent proves it).
- **Trust boundary still applies:** `resource_link`/attachment content is attacker-influenceable — carry
  the per-field trust tier from
  [agent-trust-boundary-and-prompt-injection.md](agent-trust-boundary-and-prompt-injection.md); no exfil
  vector on the read path.

**Biggest uncertainty / watch:** MCP is fast-moving (2025-11-25 stable, 2026-07-28 RC) and SDD formats
are still settling (Spec Kit v0.x, OpenSpec 1.0, AGENTS.md v1.1 frontmatter) — build to today's stable,
treat `structuredContent` as reliable and frontmatter/SDD schemas as shifting targets.

## Sources (primary, 2026)

- AGENTS.md (standard + adopters): <https://agents.md/> · Codex AGENTS.md: <https://developers.openai.com/codex/guides/agents-md> · Claude Code memory/CLAUDE.md: <https://code.claude.com/docs/en/memory> · Cursor rules: <https://cursor.com/docs/rules>
- Spec Kit: <https://github.com/github/spec-kit> · OpenSpec: <https://github.com/Fission-AI/OpenSpec> · ADR-alongside-specs: <https://intent-driven.dev/blog/2026/04/29/spec-driven-development-with-adr/> · ADRs: <https://adr.github.io/>
- MCP tools/`structuredContent`/`outputSchema`: <https://modelcontextprotocol.io/specification/draft/server/tools> · MCP versions (2025-11-25 stable): <https://blog.modelcontextprotocol.io/posts/2025-11-25-first-mcp-anniversary/> · 2026-07-28 RC: <https://blog.modelcontextprotocol.io/posts/2026-07-28-release-candidate/>
- Claude Code MCP consumption (token caps, resources, prompts): <https://code.claude.com/docs/en/mcp> · filter-before-return (150K→2K): <https://www.anthropic.com/engineering/code-execution-with-mcp>
- Sentry suspect commits: <https://docs.sentry.io/product/issues/suspect-commits/> · OTel VCS attributes: <https://opentelemetry.io/docs/specs/semconv/attributes-registry/vcs/>

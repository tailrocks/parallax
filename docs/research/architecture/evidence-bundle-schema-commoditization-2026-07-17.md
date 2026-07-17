# Evidence-bundle schema commoditization recheck (2026-07-17)

<!-- markdownlint-disable MD013 -->

**Pass target:** research-agenda item **#3** — will an open standard commoditize
the Parallax evidence-bundle schema (especially an OTel
investigation/incident convention)?

**Status of prior claim (theory under test):**
[research-agenda.md](../research-agenda.md) / pass **48** (same day): still not
commoditized. Pass **85** re-opens the theory after concurrent wedge/pricing
work to catch any same-day OTel/OCSF movement. **Pass 123** re-fetches primary
API pins after A1–A6 / monetization / wedge passes. **Pass 157** re-polls the
same kill signals after pass 156 wedge work.

**Verdict (pass 48 + pass 85 + pass 123 + pass 157):** **Still not commoditized.**
No OpenTelemetry semantic convention ships a portable, versioned, redacted,
validator-backed investigation/evidence bundle comparable to Parallax's
`bundle-v1` / `envelope-v2`. Adjacent standards remain **attribute fragments**
(OTel) or **security-incident event shapes** (OCSF), not coding-agent fix-loop
artifacts. Confidence: **high** for "no OTel investigation schema GA";
**medium** for "field will not ship one in 12 months" (active GenAI/CI/CD
pressure, idle incident-attribute track).

**Pass 123 primary re-fetch (2026-07-17):**

| Source | Finding |
| --- | --- |
| [semconv #1185](https://github.com/open-telemetry/semantic-conventions/issues/1185) | Still **open**; `updated_at` still **2025-10-24T14:40:05Z** (no 2026 activity) |
| `model/` dirs (contents API) | Still **no** `incident` / `investigation` / `rca` / `postmortem`; `mcp` present (tool telemetry); `artifact` present (SLSA packages) |
| OCSF releases | Latest still **`1.8.0`** (2026-03-18) |

**Pass 157 primary re-fetch (2026-07-18):**

| Source | Finding |
| --- | --- |
| [semconv #1185](https://github.com/open-telemetry/semantic-conventions/issues/1185) | Still **open**; labels still `cicd:phase-2`, `triage:accepted:ready-with-sig`; `updated_at` still **2025-10-24T14:40:05Z** (**~9 months** idle into 2026) |
| [semconv #1081 alerts](https://github.com/open-telemetry/semantic-conventions/issues/1081) | Still **open**; `updated_at` still **2025-11-09**; experts-needed — alert **events**, not investigation packs |
| `model/` name filter (`incid|invest|rca|evidence|bundle|postmortem|forensic`) | **Empty** (contents API) |
| Code search `filename:incident` + `path:model investigation` | **total_count 0** each (authenticated search) |
| [genai#51 session.id](https://github.com/open-telemetry/semantic-conventions-genai/issues/51) | Still **open**; `updated_at` still **2026-05-05** (adjacent GenAI session pressure, **not** an investigation artifact) |
| OCSF releases | Latest still **`1.8.0`** (published **2026-03-18**) |

**Evidence class:** primary GitHub issue/tree + first-party release pages +
vendor product docs (desk recheck). Not a measurement of A1 value.

---

## What would falsify the "not commoditized" claim

Any of the following, observed in primary sources, reopens agenda item 3 as a
**kill or align-now** trigger:

1. An OTel (or CNCF) **stable** semantic convention + JSON Schema for a
   portable multi-signal investigation/incident **artifact** with provenance,
   redaction report, and cited evidence refs — adopted by ≥1 major agent/MCP
   surface.
2. A SigNoz / OpenObserve / Grafana / Datadog **open, versioned** investigation
   export that is schema-validatable, redaction-aware, and portable across
   vendors (not a product-bound UI dump).
3. An OCSF (or similar) class that adds **software-failure + fix-outcome**
   semantics and is consumed by coding agents as the default context contract.

Until then: keep the constructive strategy (profile over OTel + Sentry grouping
+ OCSF shape + CloudEvents envelope); do not invent a greenfield container.

---

## Primary sources checked (2026-07-17)

| Source | What was checked | Finding |
| --- | --- | --- |
| [OTel semconv #1185 — Add incident attributes](https://github.com/open-telemetry/semantic-conventions/issues/1185) | state, labels, body, `updated_at` | **Still open.** Labels: `cicd:phase-2`, `triage:accepted:ready-with-sig`. Last update **2025-10-24** (no 2026 activity). Scope = **attributes** (`outage.incident` suggested), not a bundle artifact. `incident.yaml` was **removed** from an earlier CI/CD PR for separate re-evaluation. **Pass 85:** API re-fetch identical (`updated_at` still 2025-10-24). |
| OTel `model/` tree (GitHub contents API) | directories under `model/` | Present: `cicd`, `error`, `event`, `exceptions`, `gen-ai`, `deployment`, `cli`, `mcp` (see below), `artifact` (SLSA package files — **not** investigation packs), `session`, … **No** `incident`, `investigation`, `rca`, or `postmortem` model directory. Recursive path search for those names: **empty** (pass 48 + pass 85). |
| Code search `incident` yaml in semconv repo | filename:yaml | **0 hits** for committed incident yaml (issue body still refers to deferred file). **Pass 85:** total_count still **0**. |
| OTel `model/mcp/` | tree listing | **Only `deprecated/`** (MCP attributes **moved**; registry/spans/metrics marked deprecated). MCP semconv is **agent tool-call telemetry**, not an investigation/evidence **artifact** schema. Do not confuse MCP attributes with a portable evidence bundle. |
| OTel `model/artifact/` | `registry.yaml` | **SLSA/package** artifact attributes (`artifact.filename`, `purl`, `hash`) — software supply-chain distribution objects, **not** Parallax-style failure evidence packs. |
| [OTel #1081 — Semantic conventions for alerts](https://github.com/open-telemetry/semantic-conventions/issues/1081) | state | **Still open** (`triage:accepted:needs-sig`, experts needed). Alert **events**, not investigation bundles. Last update 2025-11-09. |
| [OCSF schema](https://github.com/ocsf/ocsf-schema) | releases + browser version | Latest release tag still **1.8.0** (2026-03-18); schema.ocsf.io still serves **1.8.0** (pass 85). **`incident_finding`** remains security-domain. No newer OCSF GA line in this window. |
| [Datadog Bits Code blog](https://www.datadoghq.com/blog/bits-ai-dev-agent/) | merge policy language | PR generation + CI iterate; **"resulting pull request still goes through normal human review"** / **"agent proposes pull requests, but engineers decide what to merge."** |
| [Sentry Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer/) | PR creation / merge | Autofix → **PR Creation** (or handoff to external coding agent). Docs describe opening PRs / MRs; **human review/merge** path remains; org settings can **disable** default PR creation. |
| [GreptimeDB releases](https://github.com/GreptimeTeam/greptimedb/releases) | latest stable/nightly | Stable **v1.1.3** (2026-07-17). Nightly line **v1.2.0-nightly-20260706**. Agenda item 5's `v1.0.2` / stalled-nightly claim is **stale** (version pin only; no storage benchmark this pass). |

Secondary leads (not used as authority): vendor marketing for "investigation
format" / "evidence pack" without published JSON Schema remains **unfalsified
marketing**, same as the 2026-05 SigNoz check.

---

## What changed vs 2026-05-29

| Claim layer | 2026-05-29 | 2026-07-17 recheck |
| --- | --- | --- |
| OTel investigation/incident **bundle** schema | None on roadmap | **Still none.** Closest track is still #1185 attribute work — open, CI/CD phase-2, **idle since 2025-10**. |
| OTel adjacent pressure | GenAI/CI/CD growth noted elsewhere | GenAI conventions remain the **active** semantic surface (product competitors adopt GenAI semconv for agent timelines). That standardizes **agent spans**, not **failure evidence bundles**. |
| OCSF | Cited as shape precedent | Confirmed still the closest **standardized correlated-finding** shape; latest schema line **1.8.x**; still **security-domain**, not Parallax software-failure + outcome loop. |
| Incumbent "open investigation format" | SigNoz claim = no published schema | Unchanged desk conclusion for schema commoditization; per-product marketing rechecks live in [`market/competitors/`](../market/competitors/). |
| Constructive answer (profile, don't invent) | Adopted | **Retained.** Nothing in this recheck invents a reason to abandon CloudEvents + OTel + Sentry-grouping + OCSF-shape composition. |

### Nuance the old "none on the roadmap" phrasing overstated

There **is** an accepted OTel issue for **incident attributes** (#1185). Saying
"none on the roadmap" is too absolute. Precise claim:

> OTel has a **stale, accepted, attribute-level** incident work item under
> CI/CD phase-2. It does **not** define a portable investigation/evidence
> **artifact**. It has not advanced in public issue traffic since late 2025.

If #1185 lands, Parallax should **alias/map** any new attributes into the
bundle profile (same stance as `parallax.run.id` vs future run-id standards),
not wait for OTel to invent the whole moat layer.

---

## Strategic implication for Parallax (AI-native context engine)

1. **Schema moat is still *available*, not *proven*.** Absence of a standard
   does not prove Parallax's schema will win — A3 (external adoption) and A1
   (bundle value) remain the real gates. This pass only clears the
   "OTel already commoditized us" false alarm.
2. **Commoditization risk has shifted layer.** The field is standardizing
   **agent execution telemetry** (OTel GenAI) and shipping **fixer/MCP
   surfaces** (HolmesGPT, Dynatrace MCP, Honeycomb Auto-investigations,
   Observe MCP, Bits Code, Seer). Those compress the "agents need context"
   story; they do **not** ship open portable redacted prod-incident bundles
   with outcome history. Wedge = that artifact + outcome loop, or nothing.
3. **Closed-loop app-code auto-merge still unclaimed** (related agenda #4 /
   north-star claim, rechecked same day): Datadog Bits Code still requires
   human merge; Sentry Seer still stops at PR/MR creation. Infra remediation
   loops are a different product surface. See
   [north-star-autonomous-fix-loop.md](../00-vision/north-star-autonomous-fix-loop.md).

---

## Uncertainty register

| Item | Class | Notes |
| --- | --- | --- |
| Private OTel SIG drafts not in public issues | Unknown | Desk research cannot see private Google Docs / SIG notes. Falsify by next public PR to `model/`. |
| Vendor "investigation format" as unpublished JSON | Partial | Marketing language ≠ schema. Treat as 🟡 until `$id` + validator + fixtures exist. |
| OCSF expansion into software reliability | Unlikely near-term | Domain is security; watch remediation/finding classes only. |
| GreptimeDB v1.1 line performance vs old benches | Unmeasured | Version pins updated; **no** new storage performance claims without benchmark agent. |

---

## Actions taken in-repo this pass

- Pass 48: this note + agenda items 3–5 + related stamps.
- **Pass 85:** re-verified #1185 idle; confirmed `model/` still lacks
  investigation dirs; clarified `model/mcp` (deprecated tool telemetry) and
  `model/artifact` (SLSA packages) are **not** bundle commoditization;
  OCSF still **1.8.0**.

## Next highest-value research gaps (post this pass)

1. **A1 empirical** — still #1 product risk; desk research cannot close it.
2. **A2 interviews** — still #1 business risk.
3. **Wedge-closer** — Traceway commercial recheck done (pass 84); still watch
   Rustrak/Bugsink for portable bundle + OTLP combination.
4. **Consume** any new four-way GreptimeDB bench artifacts when the benchmark
   agent lands them against **v1.1.3** (not v1.0.2).

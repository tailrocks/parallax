# Parallax vs Bugsink

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (pass 40 —
> first canonical deep-dive; fills a gap on Parallax's own Sentry-envelope wedge).
> Sources: [github.com/bugsink/bugsink](https://github.com/bugsink/bugsink) (1,939★,
> Apache-style open-core w/ `ee/`, Python/Django, last push 2026-07-17 — active;
> latest **v2.4.0**, 2026-07-10), [bugsink.com](https://www.bugsink.com/)
> (error-tracking / built-to-self-host / Sentry-SDK-compatible / scalable).
>
> **Bottom line up front:** Bugsink is a **focused, self-hosted, Sentry-SDK-
> compatible error-tracking server** (Python/Django). It is the cleanest "just run
> your own Sentry" option in the set — and on **that narrow axis it is a *fuller*
> Sentry replacement than Parallax**: Bugsink is a real Sentry *server* (full issue
> lifecycle — grouping, assignment, resolve/regress, SDK-compatible inbound),
> whereas Parallax today only **ingests** Sentry envelopes as one input among many.
> On **error-tracking-specific maturity, simplicity ("install in minutes"), and
> Sentry-protocol completeness, Bugsink is ahead of pre-release Parallax.** Parallax's
> honest edges are **OTLP-native full-signal breadth** (Bugsink is error-only), the
> *unproven* error→bundle→outcome loop + bounded agent bundle (Bugsink is classic
> error tracking, no agent-context), **Rust vs Python**, and **GreptimeDB**.

## What each product is

- **Bugsink** (`bugsink/bugsink`) — **self-hosted error tracking**: a Python/Django
  server that speaks the **Sentry SDK / ingestion protocol** natively (point any
  Sentry SDK at it via DSN; it accepts Sentry events/envelopes and runs a real
  **issue lifecycle** — grouping/fingerprinting, assignment, resolve/regress,
  release tracking, alerting). Positioned as "the simplest way to self-host error
  tracking" / a self-hosted Highlight alternative. **1,939★, v2.4.0 (2026-07-10),
> very active (pushed 2026-07-17).** **Open-core:** core self-host code is
> source-available; `ee/` directory carries a separate (proprietary) EE license;
> the `sentry/` directory is BSD-3-Clause (forked Sentry reference code) — hence
> GitHub reports the license as `NOASSERTION` (mixed). Docker-first deploy.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable
  **execution-context engine**: OTLP-native ingest of traces/logs/metrics +
  CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a
> typed evidence graph, serves bounded/redacted evidence bundles to humans and
> coding agents. **Also ingests Sentry envelopes** (`sentry_http.rs`, shipped) —
> but as *one* signal feeding error-derivation, not a Sentry-server product.
> GreptimeDB + Turso. **Pre-release.**

**Crucial framing:** both touch the Sentry protocol, but at **different depths**.
Bugsink **is** a Sentry server (full issue-management product). Parallax **consumes**
Sentry envelopes as input to a broader error-derivation + agent-context engine.
They overlap on "self-hosted + Sentry-protocol-aware error tracking" but diverge
sharply on scope (Bugsink = error-only product; Parallax = full-signal + agent-context).

## Signal coverage

| Signal | Bugsink (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| Errors / exceptions | ✅ **(the entire product — Sentry-SDK-native issue lifecycle)** | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| Sentry event/envelope ingest | ✅ **full Sentry server** (grouping/lifecycle) | ✅ envelope ingest only (`sentry_http.rs`) |
| Traces (OTLP) | ❌ | ✅ OTLP traces (🏗) |
| Logs (OTLP) | ❌ | ✅ OTLP logs (🏗) |
| Metrics (OTLP) | ❌ | ✅ OTLP metrics (🏗) |
| Continuous profiling | ❌ | ❌ |
| RUM / session replay | ❌ | ❌ |
| LLM / agent spans | ❌ | 🟡🧪 (in code) |
| Evidence bundle / agent context | ❌ | ✅ (🏗, A1) |

**Verdict:** **Bugsink is error-only and deep; Parallax is multi-signal and
incomplete (pre-release).** On the Sentry/error-tracking axis Bugsink is the more
complete product today; on every other signal Parallax's *design* is broader.

## The Sentry-wedge honesty (no-bias)

This is the most bias-prone cell, so state it plainly: **on "self-hosted
Sentry-alternative," Bugsink is ahead of Parallax.** Bugsink ships a full
issue-management surface (grouping, assignment, resolve/regress, release health,
alerts) that Parallax does not have — Parallax's Sentry support is an *ingest
adapter* feeding error-derivation, not a Sentry product. If a user's need is
"replace Sentry, self-hosted, simply," Bugsink is the more direct answer today.

**Where the framing flips (also plainly):** Bugsink does **nothing** with the
telemetry beyond classic error tracking — no OTLP traces/metrics/logs, no
trace↔error↔deploy correlation across a full-stack incident, no agent-context
bundle, no fix-outcome loop. Parallax's thesis is that *deriving* a bounded
evidence bundle + outcome loop from full-signal telemetry (incl. Sentry errors)
beats a standalone error tracker for coding-agent incident fixes. That thesis is
**unproven (A1)** — and against a focused, mature, simple tool like Bugsink, the
burden is on Parallax to show the broader engine is worth it over "just run
Bugsink + your existing APM."

## Ingestion & transport

- **Bugsink:** **Sentry SDK protocol** (DSN-based; any Sentry SDK works — the
  explicit pitch is "connect any application"). No OTLP ingest.
- **Parallax:** OTLP gRPC+HTTP (all 3 signals) **+** shipped Sentry-envelope adapter.

**Verdict:** **Different ingest models.** Bugsink = Sentry-protocol-only (deep on
that one protocol); Parallax = OTLP-native + Sentry-as-one-input. For a Sentry
shop, Bugsink is drop-in; Parallax requires OTLP instrumentation (and Sentry
envelopes are a secondary path).

## Architecture & deployment

- **Bugsink:** **Python/Django**, Docker-first ("install in minutes"), single
  container. Django ORM backing store *(exact production DB recommendation not
  pinned this pass — Django default SQLite / PostgreSQL for scale; verify on
  bugsink.com/docs)*. Built explicitly to self-host.
- **Parallax:** Rust single-binary supervising GreptimeDB + embedded Turso
  metadata; Apache-2.0; air-gap-capable.

**Verdict:** **Both self-host-first**, both simple-ish deploy (Docker / single
binary). Bugsink's "2-minute install" is a genuine simplicity strength (the UX bar
Parallax must match). Rust-vs-Python is a substrate difference, not a user-facing
verdict. Bugsink's backing store is unproven head-to-head vs GreptimeDB
(benchmark-dependent, low-stakes at Bugsink's error-only scale).

## Openness, licensing & vendor lock-in

- **Bugsink:** **open-core, source-available** — core self-host code is open;
  `ee/` is proprietary (separate license); `sentry/` dir is BSD-3 (forked Sentry
  code). GitHub `NOASSERTION` (mixed). **Less open than Parallax's Apache-2.0**
  (Parallax has no `ee/` paywall — it's uniformly Apache-2.0). Hosted/Cloud option
  likely (bugsink.com) — *(confirm exact Cloud + EE pricing on bugsink.com).*
- **Parallax:** **Apache-2.0**, uniformly open, no EE paywall, OTLP-native,
  portable bundle.

**Verdict:** on **openness, Parallax wins** — Apache-2.0 (no `ee/` gating) vs
Bugsink's open-core/NOASSERTION. A real, if narrow, Parallax edge.

## Where Bugsink plainly wins

- **Self-hosted Sentry-alternative completeness** — full issue lifecycle
  (grouping/assignment/resolve/regress/release/alerts), Sentry-SDK-native.
- **Simplicity** — "install in minutes," single Docker container, focused.
- **Maturity on the error-tracking axis** — shipped, active (v2.4.0, 1,939★,
  pushed today), real product (not pre-release).
- **Drop-in for Sentry shops** — point existing Sentry SDKs at it.

## Where Parallax honestly edges Bugsink

- **Full-signal OTLP breadth** — traces/logs/metrics; Bugsink has none. *(Real, by design.)*
- **Error→bundle→outcome loop** — Bugsink is classic error tracking (no outcome);
  Parallax targets it. *(Thesis, **unproven** — A1.)*
- **Bounded, redacted, agent-safe bundle** — Bugsink has no agent-context
  surface. *(Thesis, **unproven** — A1.)*
- **Apache-2.0 (no `ee/` paywall)** vs Bugsink open-core. *(Real.)*
- **Rust + GreptimeDB** substrate. *(Design choice, unproven vs Bugsink's stack.)*

> **Honest summary:** Bugsink is a strong, focused, **self-hosted Sentry-SDK-
> compatible error-tracking server** — and on the narrow "replace Sentry, simply,
> self-hosted" axis it is **a fuller product than Parallax today** (real issue
> lifecycle vs Parallax's envelope-ingest adapter). Written plainly, not minimized.
> But Bugsink is **error-only** — no OTLP telemetry breadth, no agent-context, no
> outcome loop. Parallax's thesis (derive a bounded evidence bundle + outcome loop
> from *full-signal* telemetry) is a different, broader bet that is **unproven
> (A1)**; against Bugsink's "just run your own Sentry" simplicity, the burden is on
> Parallax to prove the broader engine earns its complexity. Both are self-hostable
> Sentry-protocol-aware; Bugsink more mature on the error-specific axis, Parallax
> broader but pre-release.

## Watch triggers — re-evaluate Bugsink if it:

- Adds **OTLP ingest** (traces/logs/metrics) — would broaden from error-only toward full-stack (collision with Parallax's breadth edge).
- Adds an **AI/agent-context** surface or **outcome tracking**.
- **Cloud/SaaS + EE pricing** materially changes the openness posture (confirm exact Cloud/EE terms on bugsink.com).

## Open questions / what measurement would settle

- **A1 gate vs Bugsink:** for a team that "just needs self-hosted error tracking,"
  does Parallax's full-signal engine + bundle beat "run Bugsink + existing APM" for
  coding-agent incident fixes? **Unproven** — and Bugsink's simplicity is the high bar.
- **Bugsink exact license terms** (core license + `ee/` terms) + Cloud/EE pricing — confirm on bugsink.com.

## Sources (accessed 2026-07-17)

- [github.com/bugsink/bugsink](https://github.com/bugsink/bugsink) — 1,939★, Python/Django, last push 2026-07-17 (GitHub API); latest **v2.4.0** (2026-07-10); license `NOASSERTION` (open-core: core + proprietary `ee/` + BSD-3 `sentry/` fork).
- [bugsink.com](https://www.bugsink.com/) — [error tracking](https://www.bugsink.com/error-tracking/), [built to self-host](https://www.bugsink.com/built-to-self-host/), [Sentry-SDK compatible](https://www.bugsink.com/connect-any-application/), [scalable and reliable](https://www.bugsink.com/scalable-and-reliable/).
- Parallax side: [Sentry-envelope ingest](parallax-vs-sentry.md) (`sentry_http.rs` shipped), [decisions/storage-engine.md](../../decisions/storage-engine.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
- Sibling (Sentry-alternative / error-tracking peers): [parallax-vs-sentry.md](parallax-vs-sentry.md) (the reference), [parallax-vs-highlight.md](parallax-vs-highlight.md) (wound down). Other small Sentry-alts referenced in legacy [alternatives-deep-analysis.md](../alternatives-deep-analysis.md) (Rustrak, edde746/bugs, Urgentry, GlitchTip) — not separately deep-dived yet.

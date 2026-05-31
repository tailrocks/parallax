# AI-Native Debugging & Agent Observability Tools

**Research date:** 2026-05-31
**Scope:** Open-source or source-available tools that help AI agents or coding agents debug, investigate, or fix software failures. NOT traditional observability platforms.

---

## Summary

This research covers tools across three overlapping categories:

1. **AI agent debugging frameworks** — debug the agent itself (trajectories, failures, costs)
2. **AI-powered incident investigation / SRE agents** — agents that investigate production incidents
3. **Coding agent observability / context tools** — tools that give coding agents better error context

A key differentiator for Parallax is whether a tool produces **evidence/context bundles** that another agent could consume, and whether it **tracks fix outcomes** (did the fix actually work?). Almost none do both.

---

## 1. Agent Debugging Frameworks

### 1.1 AgentRx (Microsoft Research)

| Field | Value |
|---|---|
| **URL** | https://aka.ms/AgentRx/Code |
| **GitHub** | https://github.com/microsoft/AgentRx (109 stars) |
| **What it does** | Automated diagnostic framework that pinpoints the "critical failure step" in failed AI agent trajectories. Synthesizes executable constraints from tool schemas and domain policies, then logs evidence-backed violations step-by-step. |
| **Language/stack** | Python |
| **License** | MIT |
| **Key features** | Trajectory IR normalization, static + dynamic invariant generation, step-by-step checker, LLM-based judge with 10-category failure taxonomy, auditable violation log |
| **Evidence bundles** | **Yes** — produces structured violation logs with evidence at each step; validation log is the core output |
| **Tracks fix outcomes** | No — diagnoses failures but does not track whether subsequent fixes succeed |
| **Notes** | Published March 2026. ICLR paper. 115 annotated failed trajectories across τ-bench, Flash, Magentic-One. Domain-agnostic. +23.6% failure localization over baselines. |

### 1.2 AgentLens (Auriel AI)

| Field | Value |
|---|---|
| **URL** | https://pypi.org/project/auriel-agentlens/ |
| **GitHub** | https://github.com/auriel-ai/agentlens (2 stars) |
| **What it does** | Offline debugging toolkit for AI agents: record, replay, failure analysis, and cost tracking. Local-first, no API credits needed for replay. |
| **Language/stack** | Python (LangChain, OpenAI integrations) |
| **License** | MIT |
| **Key features** | Decorator-based run recording, offline replay, automatic failure pattern detection (timeouts, empty outputs, hallucinations), token cost tracking, CLI |
| **Evidence bundles** | Partial — records inputs/outputs/tool calls/tokens to local JSONL, but not structured for agent consumption |
| **Tracks fix outcomes** | No |
| **Notes** | Very early (2 stars). Complements production monitors like LangSmith. Focus on dev-time iteration, not production debugging. |

### 1.3 OpenRCA (Microsoft Research)

| Field | Value |
|---|---|
| **URL** | https://aka.ms/openrca |
| **GitHub** | https://github.com/microsoft/OpenRCA (351 stars) |
| **What it does** | Benchmark + baseline agent (RCA-agent) for assessing LLMs' ability to locate root causes of software failures from telemetry data (KPIs, traces, logs). |
| **Language/stack** | Python |
| **License** | MIT |
| **Key features** | Multi-system telemetry (Telecom, Bank, Market), Python-based data retrieval to avoid long contexts, RCA-agent baseline |
| **Evidence bundles** | Partial — the RCA-agent uses Python to retrieve and analyze telemetry, producing structured root cause predictions |
| **Tracks fix outcomes** | No — benchmark/evaluation framework, not a production tool |
| **Notes** | ICLR 2025 paper. Research artifact, not a production debugging tool. Useful for understanding how agents reason about failures. |

---

## 2. AI-Powered Incident Investigation / SRE Agents

### 2.1 HolmesGPT (CNCF Sandbox)

| Field | Value |
|---|---|
| **URL** | https://holmesgpt.dev/ |
| **GitHub** | https://github.com/HolmesGPT/holmesgpt (2,500 stars) |
| **What it does** | Open-source AI SRE agent that investigates production incidents and finds root causes. Works with any stack — Kubernetes, VMs, cloud providers, databases, SaaS platforms. CNCF Sandbox project. Originally by Robusta.Dev, with major contributions from Microsoft. |
| **Language/stack** | Python (agentic loop, 30+ toolset integrations) |
| **License** | Apache 2.0 |
| **Key features** | 30+ data source integrations (K8s, Prometheus, Grafana, Datadog, AWS, Azure, GCP, etc.), petabyte-scale data handling, bidirectional alert integrations (PagerDuty, OpsGenie, Jira), any LLM provider, Operator Mode for 24/7 background monitoring, GitHub integration for auto-PRs |
| **Evidence bundles** | **Yes** — investigation produces structured root cause analysis with evidence from each tool queried |
| **Tracks fix outcomes** | Partial — Operator Mode can verify deployments and catch regressions; GitHub integration can open PRs, but no formal fix-outcome tracking loop |
| **Notes** | Most mature tool in this category. 125 releases, actively maintained. CNCF governance. The "Operator Mode" is closest to what Parallax needs — continuous health checks + automated investigation. |

### 2.2 Aurora / Arvo AI

| Field | Value |
|---|---|
| **URL** | https://www.arvoai.ca/ |
| **GitHub** | https://github.com/Arvo-AI/aurora (257 stars) |
| **What it does** | AI-powered incident management platform for SRE teams. LangGraph-orchestrated agents autonomously investigate incidents across AWS, Azure, GCP, Kubernetes using 30+ tools. Generates structured RCA with remediation recommendations and can open fix PRs. |
| **Language/stack** | Python (Flask, Celery, LangGraph) + Next.js frontend + Memgraph (graph DB) + Weaviate (vector) + PostgreSQL |
| **License** | Apache 2.0 |
| **Key features** | Infrastructure knowledge graph (Memgraph), blast radius analysis, AI-suggested code fixes with PR generation, automated postmortem generation, MCP server for IDE integration, 30+ integrations, multi-cloud, self-hosted |
| **Evidence bundles** | **Yes** — structured RCA with timeline, impact assessment, blast radius, remediation steps |
| **Tracks fix outcomes** | Partial — suggests code fixes and generates PRs, but no closed-loop verification that the fix resolved the incident |
| **Notes** | Helm chart for K8s deployment. Docker Compose for local. Most feature-complete SRE agent platform. Very active development (491 commits). |

### 2.3 OpenSRE (Tracer AI)

| Field | Value |
|---|---|
| **URL** | https://opensre.in/ |
| **GitHub** | (repo not found at github.com/Tracer-ai/opensre — likely private or different org) |
| **What it does** | Open-source AI SRE framework built on LangGraph. Agents investigate production incidents autonomously with episodic memory and knowledge graph. Self-hosted. |
| **Language/stack** | Python (LangGraph) + Neo4j (knowledge graph) |
| **License** | Apache 2.0 |
| **Key features** | Episodic memory with similarity-based retrieval, Neo4j service topology graph, 46 investigation skills, multi-agent architecture (planner + specialized sub-agents), progressive skill loading, real-time SSE streaming |
| **Evidence bundles** | **Yes** — produces structured root cause reports with confidence scores |
| **Tracks fix outcomes** | Partial — cross-incident learning from memory, but no explicit fix-verification loop |
| **Notes** | Positioned as alternative to Incident.io and PagerDuty. Toolkit approach rather than fixed product. Created by Swapnil Dahiphale. |

### 2.4 Coroot

| Field | Value |
|---|---|
| **URL** | https://coroot.com/ |
| **GitHub** | https://github.com/coroot/coroot (7,700 stars) |
| **What it does** | Open-source observability + APM with AI-powered root cause analysis. eBPF-based zero-instrumentation metrics, logs, traces, and profiling. Identifies 80%+ of issues automatically. |
| **Language/stack** | Go (backend) + Vue (frontend) |
| **License** | Apache 2.0 |
| **Key features** | eBPF zero-instrumentation, AI RCA, service maps, SLO tracking, deployment tracking, cost monitoring, predefined inspections |
| **Evidence bundles** | Partial — RCA produces actionable insights, but designed for human consumption, not agent consumption |
| **Tracks fix outcomes** | Partial — deployment tracking compares before/after, but not a closed fix loop |
| **Notes** | Very popular (7,700 stars, 25M+ downloads). Full observability stack, not just AI debugging. Has "Agentic-ready Observability" as a feature, suggesting API access for agents. |

### 2.5 Cerebro (Writer)

| Field | Value |
|---|---|
| **URL** | https://writer.com/engineering/cerebro-ai-security-alert-triage-system/ |
| **GitHub** | (announced open source, repo not yet public as of March 2026) |
| **What it does** | Open-source AI agent system for security alert triage. Gathers context across fragmented systems, builds infrastructure topology graphs, deduplicates alerts, makes evidence-based triage decisions. |
| **Language/stack** | Go (concurrency, cloud SDKs, static binaries) + Snowflake (data warehouse) |
| **License** | Open source (license TBD — repo not yet public) |
| **Key features** | Infrastructure resource graph, 7 enrichment tools, deduplication agent, triage agent with evidence-backed decisions, human-in-the-loop thresholds, deterministic validation, full traceability |
| **Evidence bundles** | **Yes** — every triage output includes which tools were called, what evidence was retrieved, confidence scores, and decision rationale |
| **Tracks fix outcomes** | Partial — tracks analyst override rate and false positive/negative rates, but focused on triage not remediation |
| **Notes** | Security-focused, not general SRE. Interesting model for evidence-based agent decisions with guardrails. 2% actionable signal from thousands of daily alerts → 90-second triage. |

---

## 3. Coding Agent Observability / Context Tools

### 3.1 Noctrace (Nyktora)

| Field | Value |
|---|---|
| **URL** | https://nyktora.github.io/noctrace/ |
| **GitHub** | https://github.com/nyktora/noctrace (4 stars) |
| **What it does** | Chrome DevTools Network-tab-style waterfall visualizer for AI coding agent workflows. Monitors tool calls, tracks token usage, detects context rot. Zero config, zero cloud, 100% local. |
| **Language/stack** | TypeScript (Express 5, React 19, Vite 8, Tailwind CSS 4, Zustand 5) |
| **License** | MIT |
| **Key features** | Waterfall timeline, sub-agent visibility, context health scoring (A-F grade), token cost tracking, loop detection, re-read detection, 8 efficiency patterns, 13 security patterns, session comparison, multi-provider (Claude Code, Codex CLI, Copilot Chat) |
| **Evidence bundles** | Partial — session export as standalone HTML; context health breakdowns with per-signal grades; but designed for human visualization, not agent consumption |
| **Tracks fix outcomes** | No — observes sessions but does not track whether the agent's fixes worked |
| **Notes** | Very new (4 stars) but feature-rich. The context health scoring (fill, compactions, re-reads, error rate, tool efficiency) is the most sophisticated agent-health diagnostic seen. Zero-config is a strong differentiator. |

### 3.2 Observal (BlazeUp AI)

| Field | Value |
|---|---|
| **URL** | https://observal.io/ |
| **GitHub** | (repo referenced from site, ~1,697 GitHub stars per site) |
| **What it does** | Open-source registry and observability platform for AI coding agents. Trace every tool call, score every session, produce actionable improvement reports. Self-hosted. |
| **Language/stack** | TypeScript/Python (Postgres + ClickHouse + Redis) |
| **License** | AGPL-3.0 |
| **Key features** | Agent registry (browse, install, publish agents), session tracing (every tool call, tokens, outcomes), agent insights (named issues with metrics, exact system prompt fixes), cost tracking, cross-platform (Claude Code, Cursor, Kiro, Copilot CLI, Gemini CLI) |
| **Evidence bundles** | **Yes** — structured traces with tool calls, token usage, and outcomes; insight reports with specific failures, metrics, and remediation |
| **Tracks fix outcomes** | Partial — compares sessions over time, shows error rates and efficiency metrics, but no formal "did this fix work?" loop |
| **Notes** | AGPL license is a constraint for commercial use. The "Agent Insights" feature (specific action report with system prompt changes) is unique and directly useful. "No sampling — every session, every tool call." |

### 3.3 Syncause (Syn-Cause)

| Field | Value |
|---|---|
| **URL** | https://syn-cause.com/ |
| **GitHub** | (repo not found — may be private or different name) |
| **What it does** | Repurposes OpenTelemetry for coding agents. Local, zero-config debugging context — a "flight recorder" that captures runtime data in a ring buffer and delivers it directly to AI agents in the IDE for time-travel debugging without reproduction. |
| **Language/stack** | TypeScript/JavaScript, Python, Java (VS Code extension) |
| **License** | Source-available (exact license unclear — VS Code extension, not on GitHub) |
| **Key features** | In-process OTel ring buffer capture, freeze-on-trigger snapshots, semantic matching (embedding search for relevant traces), local-only with PII sanitization, stack traces + local variables + heap objects |
| **Evidence bundles** | **Yes** — this is the closest to "evidence bundle for agents" in the entire landscape. Frozen ring buffer snapshots with filtered, relevant stack frames and variables delivered as structured JSON to the agent |
| **Tracks fix outcomes** | No — provides debugging context but does not verify fixes |
| **Notes** | Published Dec 2025. The "flight recorder" concept is exactly what Parallax needs for error context. Solves the "guess → log → restart" loop that plagues coding agents. Local-first, privacy-first. |

### 3.4 Context Hub (DeepLearning.AI / Andrew Ng)

| Field | Value |
|---|---|
| **URL** | https://github.com/context-hub (inferred) |
| **GitHub** | Referenced from MarkTechPost article — CLI tool `chub` |
| **What it does** | Open-source CLI tool that gives coding agents up-to-date API documentation. Prevents "agent drift" where agents use deprecated APIs. Agents can annotate docs with workarounds and rate documentation accuracy. |
| **Language/stack** | CLI tool (likely Node.js or Python) |
| **License** | Open source (license TBD) |
| **Key features** | `chub search`, `chub get` (fetch curated markdown docs), `chub annotate` (agents save technical notes for future sessions), `chub feedback` (crowdsourced doc quality), language-specific variants (--lang py/js) |
| **Evidence bundles** | No — provides reference documentation context, not error/debugging context |
| **Tracks fix outcomes** | No |
| **Notes** | Published March 2026. Solves a different but related problem: stale API knowledge. The `annotate` feature (persistent agent memory for technical nuances) is interesting. |

### 3.5 UpTrain

| Field | Value |
|---|---|
| **URL** | https://uptrain.ai/ |
| **GitHub** | https://github.com/uptrain-ai/uptrain (2,400 stars) |
| **What it does** | Open-source platform to evaluate and improve generative AI applications. 20+ preconfigured evaluations, root cause analysis on failure cases, and improvement insights. |
| **Language/stack** | Python (backend) + JavaScript (dashboard) |
| **License** | Apache 2.0 |
| **Key features** | 20+ evaluation checks (response completeness, factual accuracy, code hallucination, prompt injection, etc.), RCA on failure cases, LLM-as-judge grading, embedding model experimentation, local dashboard |
| **Evidence bundles** | Partial — RCA identifies which pipeline component failed, but structured for human dashboard consumption |
| **Tracks fix outcomes** | No — evaluates and diagnoses, but does not close the loop on fixes |
| **Notes** | Broader LLM evaluation platform, not specifically agent-debugging. Last release May 2024 (may be less active). |

---

## Comparison Matrix

| Tool | Category | Stars | Evidence Bundles | Fix Outcome Tracking | Agent-Consumable Output | License |
|---|---|---|---|---|---|---|
| **HolmesGPT** | SRE Agent | 2,500 | Yes | Partial | Structured RCA | Apache 2.0 |
| **Coroot** | Observability + RCA | 7,700 | Partial | Partial | API-accessible | Apache 2.0 |
| **Aurora** | SRE Agent | 257 | Yes | Partial | Structured RCA + PRs | Apache 2.0 |
| **OpenSRE** | SRE Framework | N/A | Yes | Partial | Structured reports | Apache 2.0 |
| **AgentRx** | Agent Debugging | 109 | **Yes** | No | Violation log + taxonomy | MIT |
| **OpenRCA** | RCA Benchmark | 351 | Partial | No | Root cause predictions | MIT |
| **Syncause** | Coding Agent Context | N/A | **Yes** | No | Structured JSON snapshots | Source-available |
| **Observal** | Agent Observability | ~1,697 | **Yes** | Partial | Traces + insights | AGPL-3.0 |
| **Noctrace** | Agent Observability | 4 | Partial | No | Session export (HTML) | MIT |
| **AgentLens** | Agent Debugging | 2 | Partial | No | JSONL recordings | MIT |
| **Cerebro** | Security Triage | N/A | **Yes** | Partial | Structured triage | TBD |
| **UpTrain** | LLM Evaluation | 2,400 | Partial | No | Dashboard | Apache 2.0 |
| **Context Hub** | Agent Context | N/A | No | No | CLI output | TBD |

---

## Key Gaps (Opportunities for Parallax)

1. **No tool tracks fix outcomes in a closed loop.** Every tool either diagnoses or suggests fixes, but none verify that a fix actually resolved the original failure. This is the biggest gap.

2. **Evidence bundles are not standardized.** Tools that produce evidence (HolmesGPT, Aurora, Syncause, Observal, Cerebro) each use different formats. No interoperable "error context bundle" standard exists.

3. **Coding agent context and production incident context are separate worlds.** Syncause captures runtime context for coding agents; HolmesGPT/Aurora investigate production incidents. No tool bridges both — capturing the production failure context and delivering it to a coding agent for fix.

4. **Agent health monitoring is nascent.** Noctrace and Observal are the only tools focused on monitoring the coding agent itself (context rot, token waste, tool failure patterns). Both are very early.

5. **Security-focused triage has the best evidence model.** Cerebro's approach — mandatory evidence before decision, schema-driven output, deterministic validation — is the most rigorous evidence framework found. Applicable beyond security.

6. **Memory and learning across incidents is rare.** OpenSRE (episodic memory + similarity retrieval) and Context Hub (agent annotations) are the only tools with cross-session learning. HolmesGPT has no built-in memory.

---

## Notable Blog Posts & Discussions (2025-2026)

- **Microsoft Research Blog (Mar 2026):** "Systematic debugging for AI agents: Introducing the AgentRx framework" — introduces constraint-synthesis approach to agent debugging
- **Syncause Blog (Dec 2025):** "Debug without reproducing: Repurpose OpenTelemetry for coding agents" — flight recorder concept for coding agents
- **Writer Engineering Blog (Mar 2026):** "Cerebro: An open source agentic system for security alert triage" — evidence-based triage with guardrails
- **MarkTechPost (Mar 2026):** "Andrew Ng's Team Releases Context Hub" — CLI for agent API documentation context
- **The Menon Lab Blog (Mar 2026):** "HolmesGPT: The Open-Source AI Agent That Finds Root Causes Before You" — deep dive into HolmesGPT capabilities
- **OpenAI Blog (Apr 2026):** "An open-source spec for Codex orchestration: Symphony" — issue-tracker-to-agent orchestration
- **LangChain Blog (Aug 2025):** "Introducing Open SWE: An Open-Source Asynchronous Coding Agent" — async coding agent with self-review
- **Stoneforge (Mar 2026):** "Open Source AI Coding Agents: The Complete List" — ecosystem overview of 12+ coding agents
- **dev.to (May 2026):** "OpenSRE: Build Your Own AI Incident-Investigation Agent" — LangGraph-based SRE agent framework
- **dev.to (May 2026):** "Open Source Toolkit for Building AI Agents in 2026" — context poisoning and tool-call accumulation problem

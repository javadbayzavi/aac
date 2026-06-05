# AgenticAsCode (AAC) — Vision

AAC is a **free, open-source (Apache-2.0)** tool that onboards *any existing
repository* into an agentic development workflow. It does this by composing
**development roles/personas × their interactions × stack-convention templates**
into a project's `.claude/` setup — driven by a **local MCP server**, so it works
alongside any MCP-capable LLM (Claude first).

It is built in the open and **dogfooded inside 3T**; not a commercial product.

---

## What it is — and isn't

**Is:** a way to declare, version, and apply an agentic *team setup* (developer,
product-manager, designer — solo or multi-agent) onto a real codebase, keeping
that setup consistent and current over time.

**Is not:**
- A from-scratch app generator (cf. MetaGPT/ChatDev) — it works *with your
  existing repo*, it doesn't invent one.
- Plain code/feature scaffolding (cf. AgiFlow scaffold-mcp) — it is
  context-aware and models *roles and interactions*, not just boilerplate.
- Commercial. It complements the ecosystem (MCP, GitHub Spec-Kit, Kiro) rather
  than competing with it.

**The gap it fills:** no single tool combines *onboard-existing* + *role/
interaction composition* (incl. PM & designer) + *stack conventions as a single
source of truth* + *MCP-native* + (eventually) *a visual UI*.

---

## Principles

**Artifact model**
1. **No hardcoded logic in tools** — behavior is driven by declarative artifacts.
2. **A single source of truth** for all agentic-setup artifacts.
3. **Context-aware, not copy-paste** — decisions are based on the target project.
4. **Controlled LLM communication** during setup (structured prompts/handoffs).
5. **Setup is a state machine:** Inspection → Onboarding → Configure.
6. **A declarative artifact format** — each stack/agent/template carries metadata
   (category, detection signals, persona/mode applicability, includes, version).
7. **Composable dimensions** — `persona × mode × stack` compose; no per-combo
   templates.

**Generation quality**
8. **Validate before write** — never emit a broken setup; fail loudly, write
   nothing on failure.
9. **Idempotent & resumable** — re-runs converge; phases survive interruption.
10. **Versioned artifacts + drift reconciliation** — record which artifact
    version produced a setup; offer controlled upgrades.
11. **Non-destructive & transparent** — back up before overwrite, write only
    under the target's `.claude/`, report every change.

**Intelligence**
12. **Evidence-based, explainable detection** — surface ambiguity and ask;
    don't guess.
13. **Plan / dry-run before mutate** — preview what will be written, and why.

**Boundaries**
14. **Trust boundary on the source of truth** — pinned version, artifacts are
    data (not executable), writes confined to declared locations.
15. **LLM-agnostic contract, Claude-optimized behavior** — tool schemas are the
    interface; the LLM is an optional driver.

---

## Architecture decisions

- **License:** Apache-2.0 (explicit patent grant; standard for company-backed OSS).
- **Single source of truth:** the Rust core + declarative artifacts are
  authoritative. The `CLAUDE.md`/markdown workflow is *generated* from them, not
  hand-maintained in parallel (resolving the "twin implementations" divergence).
- **Deterministic core:** factor detect → compose → validate → diff into a
  library + thin CLI (no LLM); the MCP/LLM layer sits on top. This is what makes
  CI integration and a future UI possible without a second implementation.

---

## North star: a visual UI

`UI ↔ LLM ↔ MCP`. A local UI to visually compose development roles, their
interactions/handoffs, and the stack templates attached to each — with a live
preview of the generated `.claude/` (dry-run) and an LLM co-pilot that proposes,
explains, and resolves ambiguity.

The UI is a **renderer over the same primitives**: the stack palette is the
declarative registry; the role/interaction graph is the composable model; the
preview is plan/dry-run; export is the deterministic core. It is a fast-follow,
not the moat — the moat is the model. Built only after the declarative artifacts
and core library land; run locally to keep project context private.

---

## CI integration

1. **Validate the source of truth** — on every PR, check artifact metadata
   parses, all includes resolve, references exist, and templates render across
   the `persona × mode × stack` matrix.
2. **Drift gate for consumer repos** — a headless `verify` mode + reusable
   GitHub Action that fails if a project's `.claude/` drifted from its pinned
   artifact version (lockfile-style).
3. **Auto-refresh PRs** — when the source of truth releases a new version,
   re-run setup headlessly and open a PR updating `.claude/` ("Dependabot for
   agentic setup").

---

## Roadmap

- **Phase 0 — OSS foundation:** license, this charter, README repositioning,
  contributing guide.
- **Phase 1 — Declarative artifacts (top priority):** artifact format → dynamic
  discovery (retire hardcoded registries) → validation → CI artifact checks.
  Also collapses the twin-implementation problem. Begin dogfooding a real 3T repo.
- **Phase 2 — Correct & context-aware:** composable assembly, evidence-based
  detection, plan/dry-run, version stamping.
- **Phase 3 — Lifecycle & headless core:** core library + CLI, versioned drift +
  reconciliation, idempotent/resumable state machine, trust/pinning, CI drift-gate.
- **Phase 4 — Reach:** auto-refresh PRs, richer PM/designer sub-agents, more
  stacks, UI MVP.

---

## Relationship to the ecosystem

AAC is a good citizen of the agentic-dev ecosystem. It is MCP-native, learns
from and interoperates with declarative-template tools (AgiFlow) and spec-driven
workflows (GitHub Spec-Kit, Amazon Kiro), and differentiates on **role/persona +
interaction composition for existing repositories**, with PM and designer as
first-class participants.

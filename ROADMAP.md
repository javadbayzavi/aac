# AAC Roadmap

The build plan toward the vision in [VISION.md](VISION.md). Work proceeds one
branch + PR per item. This is the post-vision backlog — the original repo-review
fixes (#1–#15 of that round) are already merged.

---

## Phases

- **Phase 0 — OSS foundation:** license, charter, README repositioning,
  contributing guide.
- **Phase 1 — Declarative artifacts (top priority):** the contribution
  multiplier — adding a stack/agent becomes "drop a file," no Rust change. Also
  collapses the twin-implementation problem. Begin dogfooding a real 3T repo.
- **Phase 2 — Correct & context-aware:** composable assembly, evidence-based
  detection, plan/dry-run, version stamping.
- **Phase 3 — Lifecycle & headless core:** core library + CLI, versioned drift +
  reconciliation, idempotent/resumable state machine, trust/pinning, CI drift-gate.
- **Phase 4 — Reach:** auto-refresh PRs, richer PM/designer agents, more stacks,
  the visual UI MVP (north star).

---

## Backlog

| # | Item | What it does | Phase | Status |
|---|---|---|---|---|
| 1 | VISION.md charter | The "why" / design charter | 0 | ✅ done |
| 2 | Declarative artifact format | Each stack/agent/template carries metadata (category, signals, persona/mode, includes, version) | 1 | todo |
| 3 | Dynamic artifact discovery | Build the registry by scanning artifacts — retire hardcoded `stack_path` / `available_stacks` / `stack_category` / `detect_stacks` | 1 | todo |
| 4 | Twin problem | Decided: core + artifacts are authoritative → *generate* the CLAUDE.md/markdown workflow from artifacts instead of hand-maintaining it | 1/3 | decided, impl pending |
| 5 | Deterministic core lib + CLI | Factor detect → compose → validate → diff into a headless library + CLI; LLM/MCP sit on top | 3 | todo |
| 6 | Composable persona × mode × stack | Assemble from composable pieces; no per-combo templates | 2 | todo |
| 7 | Validate before write | Valid YAML, includes resolved, refs exist; write nothing on failure | 1 | todo |
| 8 | Stamp SoT version into PROJECT.yaml | Record which artifact version produced the setup | 2 | todo |
| 9 | Evidence-based detection | Emit evidence + confidence; ask on ambiguity, don't guess | 2 | todo |
| 10 | Plan / dry-run mode | Preview what will be written, and why, before mutating | 2 | todo |
| 11 | Versioned drift + reconciliation | "You're at vX, SoT is vY — here's the diff + a controlled upgrade" | 3 | todo |
| 12 | Idempotent & resumable state machine | Re-runs converge; phases survive interruption and across sessions | 3 | todo |
| 13 | Pin & trust the source of truth | Version pin, artifacts-as-data, writes confined to declared locations | 3 | todo |
| 14 | CI: validate SoT artifacts | On PR: metadata parses, includes resolve, refs exist, templates render across the matrix | 1 | todo |
| 15 | CI: headless verify + drift-gate Action | `verify` mode + reusable GitHub Action; fails if `.claude/` drifted from the pinned version | 3 | todo |
| 16 | CI: auto-refresh PRs | New SoT version → headless re-run → opens PR updating `.claude/` ("Dependabot for agentic setup") | 4 | todo |
| 17 | Richer PM/designer sub-agents | Sprint-planning / design-discovery agents (the deferred multi-agent vision) | 4 | todo |
| 18 | Add missing stacks | Author `go-chi`, `python-django` (currently "not yet available") | 4 | todo |
| 19 | Visual UI MVP | `UI ↔ LLM ↔ MCP` — preview + wizard first, then canvas (north star) | 4 | todo |

Plus two small Phase-0 docs items not in the numbered backlog: **README
repositioning** and **CONTRIBUTING** (the latter best after Phase 1, once
contributing is frictionless).

---

## Dependency order

```
2 → 3 → (4, 7) → 14
        └─→ 6 / 8 / 9 / 10
                └─→ 5 / 11 / 12 / 13 / 15
                        └─→ 16 / 17 / 19
```

Content (#18) can land any time after #3.

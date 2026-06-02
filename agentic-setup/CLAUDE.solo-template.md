# {{project.name}} — Claude

**Purpose:** {{project.description}}

You are the sole executor for this project. You assume different **roles** as the workflow progresses — orchestrator, Product + AI Engineer, BE Developer, FE Developer, or DevOps Engineer. Switch roles explicitly; load that role's conventions before acting.

**Hard rules — no exceptions:**
- Follow the workflow sequence. Do not skip phases or combine roles in one step.
- Do not improvise. If a situation is not covered, stop and ask the user.
- For any intent that needs clarification: ask questions first, get answers, then load from the Knowledge Registry. Never pre-load files before understanding what is needed.
- Do not load any convention file until you are actively writing code for that role. Reading a role section does not trigger loading.
- Only assume roles listed in the Feature Plan's `involves[]`. Skip all others entirely — do not read their sections.
- Compact between heavy implementation phases if context grows large.

**Session continuity:** if closing mid-feature, save the approved Feature Plan to `.claude/active-plan.json` (do not track it via git). Next session: read `.claude/active-plan.json` first and resume from the next pending role in `involves[]` — skip planning entirely. Otherwise, always start from the Product + AI Engineer role and produce a fresh Feature Plan.

---

## What Is This Project

{{project.extended_description}}

Tech-stack: {{tech_stack}}

---

## Workflow

```
[Product + AI Engineer]  pick next item from board → produce Feature Plan → present to user
                           ↑ if open_questions non-empty: resolve with user, revise plan, loop
[User]                   approve plan  ← may loop with adjustments
[BE Developer]           implement + open PRs (if involves BE)
                           stop (PRs are ready for review)
[User]                   review PR → provide comments
[BE Developer]           fix review comments + push  ← may loop
[User]                   approve → merges PR(s)  (Claude opens the PR(s); user merges / or ask to merge)
[BE Developer]           update `.claude/active-plan.json` and ask user before switching to next role
[FE Developer]           implement + open PRs (if involves FE)
                           stop (PRs are ready for review)
[User]                   review PR → provide comments
[FE Developer]           fix review comments + push  ← may loop
[User]                   approve → merges PR(s)  (Claude opens the PR(s); user merges / or ask to merge)
[FE Developer]           update `.claude/active-plan.json` and ask user before switching to next role
[DevOps Engineer]        Similar sequence (if involves DevOps)
[Product + AI Engineer]  close board item - drain `.claude/active-plan.json` data and keep its structure - advance to next
```

---

## Current State

| Axis | Status |
|---|---|
| Backend | {{current_state.backend}} |
| Frontend | {{current_state.frontend}} |
| Domain model | {{current_state.domain_model}} |
| Testing | {{current_state.testing}} |
| Observability | {{current_state.observability}} |
| CI/CD | {{current_state.cicd}} |

---

## Knowledge Registry

Load a source only when needed for the active role. Do not pre-load. Never load `history` or `session-notes` unless the user explicitly asks.

| Topic | Source |
|---|---|
| `conventions/backend` | `.claude/stacks/backend.md` |
| `conventions/frontend` | `.claude/stacks/frontend.md` |
| `conventions/persistence` | `.claude/stacks/persistence.md` |
| `conventions/security` | `.claude/stacks/security.md` |
| `conventions/devops` | `.claude/stacks/devops.md` |
| `conventions/pr-workflow` | `.claude/stacks/pr-workflow.md` |

---

## Protocols (reference schemas)

Only these schemas are active. Use them as checklists — not message envelopes.

| Phase | Schema |
|---|---|
| Feature Plan produced | `.claude/protocols/FEATURE_PLAN.json` |

Other files in `.claude/protocols/` are legacy inter-agent schemas — ignore them.

---

## Roles

Read only the role section(s) listed in `involves[]`. Do not read other role sections.

### Role: Product + AI Engineer

Owns product intent, user-journey alignment, and LLM/prompt design. Plans only — does not implement.

Do not load `conventions/*` in this role. Load `product/journeys` only after clarifying what is needed.

Produce a **Feature Plan** matching `.claude/protocols/FEATURE_PLAN.json`. Do not include implementation tasks, data models, or technology choices. Do not read any source code even if mentioned in the item title or description.

Field IDs: see `.claude/PROJECT.yaml`.

---

### Developer shared rules (BE + FE + DevOps)

**Always load:** `conventions/security`, `conventions/pr-workflow`.

If the work requires a CI, deploy, or secrets change → also load `conventions/devops`.

Before editing a file, read only that file. Do not explore directories or read neighboring files unless you are directly calling or importing them.

If interrupted by a side question: answer it briefly, then explicitly state "Resuming at [exact point]" before continuing. Do not restart implementation.

---

### Role: BE Developer

**Load:** `conventions/backend`, `conventions/testing-be`.

If scaffolding a new feature module → also load `conventions/examples`.

---

### Role: FE Developer

**Load:** `conventions/frontend`, `conventions/testing-fe`.

If scaffolding a new feature module → also load `conventions/examples-fe`.

---

### Role: DevOps Engineer

**Load:** `conventions/devops`.

**Before finishing:**
- validate workflow YAML syntax (`actionlint` if available).

---

## Notes for Claude

- Generated by AgenticAsCode on {{onboarding.date}}. To update this setup, re-run the scaffold from the AAC repo.
- No feature implementation starts before {{project.lead}} approves the feature plan.
- One feature per branch and PR. No batching.
- Source of truth for tech-stack: `.claude/PROJECT.yaml`.
- {{project.operator_notes}}

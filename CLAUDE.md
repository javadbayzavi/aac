# AgenticAsCode (AAC)

Reusable framework that onboards any project into an agentic development workflow.
The target project lives elsewhere on the filesystem. This repo stays clean.

---

## Workflow

```
User provides project_path
        │
        ▼
[Inspector]  → NOT_ONBOARDED          → [Onboarding]
             → ONBOARDED, drift        → [Configurator]
             → ONBOARDED, complete     → nothing to do
```

---

## Roles

Switch roles explicitly as the workflow progresses. Do not pre-load any file.
Load a reference only when actively executing that role's task.

### Role: Inspector

Read-only audit. Never read file contents — OS search commands only.
**One designated write:** `inspection.json` is the sole file Inspector writes — it is a transient handoff artifact, not a project file. Onboarding deletes it after `PROJECT.yaml` is written.

**Intake — complete ALL three steps before running any check:**

1. Ask in plain text: "Please share your project path (absolute) and a one-line description of what it does."
2. Use AskUserQuestion (single-select) for persona: "Developer — I write code" | "Product Manager — I manage the backlog" | "Designer — I design the UI"
3. Use AskUserQuestion (single-select) for mode: "Solo — one Claude session, I switch roles, approval at each step" | "Multi-agent — orchestrator spawns specialist sub-agents, more autonomous"

Do not run any bash command or file check until all three steps are complete and all four fields are collected.

If `project_path` is missing or not absolute → halt with a single error message. Do not proceed to checks.

**Only after intake is complete — run checks in this order:**
1. Resume check: read `<project_path>/.claude/inspection.json` if it exists — pre-populate any fields already present.
2. Does `<project_path>` exist? If not → halt. Tell user: "Path not found: `<project_path>`."
3. Does `<project_path>/.claude/PROJECT.yaml` exist? → ONBOARDED or NOT_ONBOARDED
4. If ONBOARDED: check for drift by comparing source timestamps in this repo against the onboarding date in `PROJECT.yaml`:
   - For solo: run `find agentic-setup/stacks/ agentic-setup/CLAUDE.solo-template.md -name "*.md" -newer <project_path>/.claude/PROJECT.yaml`
   - For multi-agent: run `find agentic-setup/stacks/ agentic-setup/agents/ agentic-setup/CLAUDE-multi-agent-template.md -name "*.md" -newer <project_path>/.claude/PROJECT.yaml`
   - Any result means source files relevant to this project's mode were updated after onboarding → **drift detected**
   - Check each skill listed in `<project_path>/.claude/PROJECT.yaml` `tech_stack` exists in `agentic-setup/stacks/` — if any is missing → **drift detected**
   - Do NOT check whether template files exist in the target project — templates stay in this repo only.

**Output:** Present findings as a table including: Path, Description, Persona, Mode, Onboarding Status, Onboarding Date (from `PROJECT.yaml` `status` or file mtime), Project State, Tech Stack Signals, Drift details (if any). Then use AskUserQuestion based on status:

- `NOT_ONBOARDED` → single-select:
  - "Yes, onboard this project"
  - "Cancel"

- `ONBOARDED` (no drift) → single-select:
  - "Configure this project — add or remove skills or agents"
  - "Nothing, I was just checking" → delete `<project_path>/.claude/inspection.json` if it exists, then end

- `ONBOARDED, drift` → single-select:
  - "Configure this project — add or remove skills or agents"
  - "Re-onboard — regenerate CLAUDE.md and stacks from updated templates (runs Inspector intake again)"
  - "Nothing, I was just checking" → delete `<project_path>/.claude/inspection.json` if it exists, then end

On confirmation to proceed: write findings to `<project_path>/.claude/inspection.json` with this schema:

```json
{
  "inspected_at": "<ISO date>",
  "project_path": "<absolute path>",
  "description": "<user-provided description>",
  "persona": "developer | product-manager | designer",
  "mode": "solo | multi-agent",
  "onboarding_status": "NOT_ONBOARDED | ONBOARDED | DRIFT",
  "project_state": "GREENFIELD | EXISTING | PARTIAL",
  "tech_stack_signals": ["pom.xml", "package.json", "..."]
}
```

### Role: Onboarding
One-time setup for a new project. Surveys codebase, detects tech-stack,
writes `CLAUDE-multi-agent.md`, `PROJECT.yaml`, and if it's multi-agent, writes agent files to the target project otherwise embed this simple workflow to `CLAUDE.solo-template.md`
copy `docs/active-plan.json` to `<project_path>/.claude/docs/active-plan.json` for session continuity

.
**Read and follow:** `.claude/workflows/onboarding-discovery.md` — execute its steps in the current session. Also load `SKILLS-COMPOSITION.md` and matching skill files from `agentic-setup/stacks/` when needed.

### Role: Configurator
Adds or removes skills and agents on an already-onboarded project.
Always verify onboarding status before any write.
**Read and follow:** `.claude/workflows/project-configurator.md` — execute its steps in the current session. Also load matching skill files from `agentic-setup/stacks/` when needed.

---

## Hard Rules

- Never write to a target project except through Onboarding or Configurator
- Never skip Inspector — it costs nothing and prevents overwriting existing config
- Never reference a skill file that does not fully exist in `agentic-setup/stacks/`
- Never commit project-specific output to this repo
- Never create or modify any file inside agentic-setup/ — it is read-only at runtime. Onboarding and Configurator read from agentic-setup/ and write only to `<project_path>/.claude/`

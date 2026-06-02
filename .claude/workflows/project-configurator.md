# Project Configurator

Mutates an already-onboarded project's AAC configuration. Adds or removes tech-stack skills and agents. Always verifies onboarding status before any write.

---

## Input

If `project_path` is already known from the current session, use it directly — do not ask again.
Otherwise ask in plain text: "What is the absolute path to your project?"

Then use AskUserQuestion (single-select) to collect the operation:
- "Add a tech stack — inject a new skill into relevant agents"
- "Add an agent — instantiate a new agent for this project"
- "Remove a tech stack — drop a skill and update affected agents"
- "Remove an agent — delete an agent definition from this project"

Then use AskUserQuestion (single-select) for `target` based on the selected operation:
- add/remove tech stack → first `find agentic-setup/stacks/ -name "*.md"` to get the full list, then present each available skill as a selectable option (e.g. "backend/java-21-spring-boot", "collaboration/github-issues"). Max 4 options per prompt — if more than 4 exist, group by category and ask category first, then skill within that category.
- add/remove agent → first `find agentic-setup/agents/ -name "*.template.md"` to get the full list, then present each as a selectable option.

---

## Pre-Flight: Always Run First

Before any mutation:

1. Verify `<project_path>/.claude/PROJECT.yaml` exists.
   - If not → stop. Tell user: "Project is not onboarded. Run Inspector first."
2. Read `<project_path>/.claude/PROJECT.yaml` — load current config including `mode`.
3. If `mode: solo` and the operation involves agents → stop. Tell user: "This project uses solo mode. Agent files are embedded in CLAUDE.md, not in `.claude/agents/`. Re-onboard with `mode: multi-agent` to use individual agent files."
4. Use AskUserQuestion (single-select) to confirm the planned change:
   - "Yes, proceed"
   - "Cancel"

---

## Operations

### add-tech-stack

Add a new skill to the project config and re-instantiate affected agents.

1. Verify skill file exists at `agentic-setup/stacks/<target>.md`.
   - If not → stop. Report available skills in that category. Do not create a stub.
2. Identify which `tech_stack` key this skill belongs to (backend / frontend / persistence / devops / security).
3. Append `<skill-name>` to the appropriate list under `tech_stack` in `PROJECT.yaml`. Never overwrite the entire key — add to the existing list.
4. Re-instantiate every agent whose template references `{{include stacks/<category>...}}`:
   - Read the agent template from `agentic-setup/agents/<name>.template.md`
   - Replace `{{include stacks/<category>.md}}` with full content of the matching skill file from `agentic-setup/stacks/<category>/<skill>.md`
   - Substitute `{{project.*}}` variables
   - Write to `<project_path>/.claude/agents/<name>.md`
5. Report: skill added, agents updated.

---

### add-agent

Add a new agent definition to the project.

1. Verify template exists at `agentic-setup/agents/<target>.template.md`.
   - If not → stop. List available templates. Do not create a stub.
2. Check if `<project_path>/.claude/agents/<target>.md` already exists.
   - If yes → use AskUserQuestion (single-select):
     - "Overwrite — replace existing agent file"
     - "Cancel — stop, nothing written"
3. Read template, substitute:
   - `{{include stacks/<category>.md}}` → full content of matching skill file from `agentic-setup/stacks/`
   - `{{project.*}}` → values from PROJECT.yaml
   - `{{<role>.model}}`, `{{<role>.speed}}` → from PROJECT.yaml agents block
4. Write to `<project_path>/.claude/agents/<target>.md`.
5. Add agent entry to `agents` block in `PROJECT.yaml` with default model and effort from `agentic-setup/PROJECT.yaml` template.
6. Report: agent added, skills injected, PROJECT.yaml updated.

---

### remove-tech-stack

Remove a skill from the project config.

1. Read `PROJECT.yaml` → identify which `tech_stack` key holds `<target>`.
   - If not found → stop. Report current tech_stack entries.
2. Identify which agent templates reference this skill category.
3. Remove `<skill-name>` from the list under the relevant `tech_stack` category in `PROJECT.yaml`. If the list becomes empty, remove the category key entirely.
4. Re-instantiate affected agents (same process as add-tech-stack step 4).
5. Report: skill removed, agents updated.

---

### remove-agent

Remove an agent definition from the project.

1. Verify `<project_path>/.claude/agents/<target>.md` exists.
   - If not → stop. List currently defined agents.
2. Delete `<project_path>/.claude/agents/<target>.md`.
3. Remove the corresponding entry from the `agents` block in `PROJECT.yaml`.
4. Report: agent removed, PROJECT.yaml updated.

---

## After Each Operation

Use AskUserQuestion (single-select):
- "Make another change"
- "Done"

If "Make another change" → return to Input and repeat.
If "Done" → delete `<project_path>/.claude/inspection.json` if it exists, then end.
If mode change is needed (e.g. solo → multi-agent) → tell user: "Mode changes require re-onboarding. Run Inspector again for this project to start a fresh onboarding." → delete inspection.json, then end.

---

## Rules

- Never mutate without explicit user confirmation via AskUserQuestion
- Never delete or overwrite `PROJECT.yaml` entirely — only patch specific keys
- Never create skill stubs or agent stubs — only use files that exist in `agentic-setup/`
- Never create or modify any file inside `agentic-setup/` — read from it, never write to it
- Validate PROJECT.yaml after every write that touches it
- Report clearly what changed and what the user should verify after each operation

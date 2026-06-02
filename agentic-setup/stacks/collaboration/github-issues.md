# GitHub Issues Skill

Conventions for working with GitHub Issues, Projects, and Milestones in a product development workflow.

---

## Stack Overview

- **Issue tracking:** GitHub Issues (issues, milestones, labels, projects)
- **Project board:** GitHub Projects v2 (kanban, roadmap views)
- **Integration:** GitHub CLI (`gh`) and GitHub API via Bash

---

## Issue Hierarchy

```
Milestone
└── Issue (Epic label)       (product capability, ~1–2 weeks)
    └── Issue (Story label)  (user-facing outcome, ~1–3 days)
        └── Issue (Task label) (engineering unit, ~45 min – 4 hrs)
            └── Issue (Bug label) (regression against an accepted story)
```

- **Milestone:** A release or sprint boundary — groups related epics
- **Epic:** A product capability tracked as an issue with the `epic` label
- **Story:** A user-facing outcome linked to its parent epic via task list or reference
- **Task:** A concrete implementation unit assigned to a specialist
- **Bug:** Always references the story it regresses with `Closes #<story>`

---

## Issue Format: Story

```
Title: <verb> <outcome> as <persona>

## User Story
As a <persona>,
I want to <action>,
So that <outcome>.

## Acceptance Criteria
- [ ] Given <context>, When <action>, Then <result>
- [ ] ...

## Out of Scope
- ...

## References
- Epic: #<issue number>
- Design: <Figma link or handoff doc>
```

---

## Issue Format: Bug

```
Title: <component>: <what is wrong>

## Description
<What the user experiences vs. what they should experience>

## Steps to Reproduce
1. ...
2. ...

## Expected Behavior
<What should happen>

## Actual Behavior
<What actually happens>

## Regresses
- Story: #<issue number>

## Environment
- Branch: <branch name>
- Version: <tag or commit SHA>
```

---

## Required Fields on Every Issue

| Field | Rule |
|---|---|
| Title | Start with a verb; no passive voice |
| Body | At minimum: user story + acceptance criteria (stories), steps to reproduce (bugs) |
| Labels | At minimum: type label (`epic`, `story`, `task`, `bug`) + team label |
| Milestone | Every issue must belong to a milestone |
| Assignee | Set before moving to In Progress |
| Linked PR | Link PR in comments when one is opened |

---

## Labels: Recommended Baseline Set

| Label | Color | Purpose |
|---|---|---|
| `epic` | `#7C3AED` | Product capability grouping |
| `story` | `#2563EB` | User-facing outcome |
| `task` | `#059669` | Engineering implementation unit |
| `bug` | `#DC2626` | Regression or defect |
| `blocked` | `#F59E0B` | Waiting on dependency |
| `design` | `#EC4899` | Requires or awaiting design input |
| `needs-spec` | `#6B7280` | Acceptance criteria not yet defined |

---

## Status Workflow (GitHub Projects v2)

```
Backlog → Todo → In Progress → In Review → Done
```

- Move to **In Progress** only when actively worked
- Move to **In Review** when a PR is open — link PR in issue comments
- Move to **Done** only after PR is merged and acceptance criteria verified
- Use `blocked` label (not a column) to flag dependency issues without changing status

---

## GitHub CLI: Key Operations

```bash
# Create an issue
gh issue create --title "..." --body "..." --label "story" --milestone "Sprint 1" --assignee "@me"

# List open issues
gh issue list --label "story" --milestone "Sprint 1"

# View an issue
gh issue view <number>

# Close an issue
gh issue close <number> --comment "Resolved in PR #<number>"

# Link a PR to an issue (in PR body or comment)
# Use: Closes #<issue-number>
```

---

## GitHub Projects v2: Key Operations

```bash
# List projects
gh project list --owner <org-or-user>

# Add issue to project
gh project item-add <project-number> --owner <org-or-user> --url <issue-url>

# Update item status
gh project item-edit --project-id <id> --id <item-id> --field-id <field-id> --single-select-option-id <option-id>
```

---

## Common Pitfalls

- **Orphan issues:** Every story must reference a parent epic; every task must reference a parent story
- **Missing milestone:** Unassigned issues become invisible in planning — always set a milestone
- **Stale acceptance criteria:** Update the issue if scope changes — never let the ticket drift from reality
- **Duplicate issues:** Search before creating — `gh issue list --search "<keywords>"` first
- **PR not linked:** Always add `Closes #<issue>` in the PR body to auto-close and cross-link
- **Undated blocked issues:** If an issue is `blocked`, add a comment naming the dependency and expected unblock date

---

## See Also

- `docs/conventions/product.md` — user story format, acceptance criteria
- `agentic-setup/stacks/devops/github-actions.md` — CI/CD, branch protection, PR workflow
- `agentic-setup/agents/product-manager.template.md` — product manager agent behavior

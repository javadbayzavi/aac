# Atlassian Skill

Conventions for working with Jira and Confluence in a product development workflow.

---

## Stack Overview

- **Issue tracking:** Jira (projects, epics, stories, tasks, bugs)
- **Knowledge base:** Confluence (specs, decisions, meeting notes, status reports)
- **Integration:** Atlassian MCP server (`mcp__plugin_atlassian_atlassian__*`)

---

## Jira: Issue Hierarchy

```
Epic
└── Story         (user-facing outcome, ~1–3 days)
    └── Task      (engineering unit, ~45 min – 4 hrs)
        └── Bug   (regression or defect against an accepted story)
```

- **Epic:** A product capability (e.g., "User Authentication")
- **Story:** A user-facing outcome with acceptance criteria
- **Task:** A concrete implementation unit dispatched to a specialist
- **Bug:** Always links back to the story it regresses

---

## Jira: Story Format

```
Summary: <verb> <outcome> as <persona>

Description:
As a <persona>,
I want to <action>,
So that <outcome>.

Acceptance Criteria:
- Given <context>, When <action>, Then <result>

Labels: <team> <sprint> <persona>
Priority: <Highest | High | Medium | Low>
```

---

## Jira: Required Fields on Every Issue

| Field | Rule |
|---|---|
| Summary | Start with a verb; no passive voice |
| Description | At minimum: user story + acceptance criteria |
| Priority | Always set — never leave as default |
| Assignee | Set before moving to In Progress |
| Epic Link | Every story must belong to an epic |
| Labels | At minimum: team label |

---

## Jira: Status Workflow

```
Backlog → To Do → In Progress → In Review → Done
```

- Move to **In Progress** only when actively worked
- Move to **In Review** when a PR is open (link PR in comments)
- Move to **Done** only after PR is merged and acceptance criteria verified

---

## Confluence: Page Types

| Type | When to create |
|---|---|
| **Spec** | Before engineering starts on an epic |
| **Decision Record** | Any architectural or product decision worth preserving |
| **Meeting Notes** | After any decision-making meeting |
| **Status Report** | Weekly, per project, for stakeholders |

---

## Confluence: Spec Structure

```
# <Feature Name> Spec

## Problem
<What user pain does this solve?>

## Solution
<High-level approach>

## User Stories
<Link to Jira epic>

## Open Questions
<Unresolved items — owner + due date for each>

## Out of Scope
<Explicitly excluded — prevents scope creep>
```

---

## MCP Tool Usage

Always use `cloudId = "https://3tsoftwarelabs.atlassian.net"`.
Use `maxResults: 10` on all JQL and CQL searches.

**Key operations:**
- Search issues: `searchJiraIssuesUsingJql`
- Create issue: `createJiraIssue`
- Edit issue: `editJiraIssue`
- Transition status: `transitionJiraIssue`
- Read Confluence page: `getConfluencePage`
- Create page: `createConfluencePage`
- Update page: `updateConfluencePage`

---

## Common Pitfalls

- **Stale acceptance criteria:** Update the Jira story if scope changes during implementation — never let the ticket drift from reality
- **Orphan tasks:** Every task must link to a parent story; orphan tasks create invisible work
- **Undated open questions:** Every open question in a spec must have an owner and a due date
- **Duplicate stories:** Search Jira before creating — use `searchJiraIssuesUsingJql` with relevant keywords

---

## See Also

- `docs/conventions/product.md` — user story format, acceptance criteria
- `docs/conventions/design.md` — design handoff spec format

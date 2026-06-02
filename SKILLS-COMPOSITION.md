# Skills Composition

How skill files are selected and injected into agent definitions.

---

## What a Skill File Is

A self-contained set of conventions for one tech-stack layer — versions, architecture patterns, rules, pitfalls. Lives in `agentic-setup/stacks/<category>/`. Never modified per-project — read and injected verbatim.

---

## Skill Categories

| Category      | Path                          | Purpose                                      |
|---------------|-------------------------------|----------------------------------------------|
| backend       | `stacks/backend/`             | Language + framework conventions             |
| frontend      | `stacks/frontend/`            | UI framework conventions                     |
| persistence   | `stacks/persistence/`         | Database and ORM conventions                 |
| devops        | `stacks/devops/`              | CI/CD, deployment, infrastructure            |
| security      | `stacks/security/`            | Cross-cutting security rules (always)        |
| collaboration | `stacks/collaboration/`       | Issue tracking, design tools, team workflows |

---

## Skill → Agent Injection Map

Which skill categories are injected into which agents, per persona.

### Developer

| Agent               | Skills Injected                                        |
|---------------------|--------------------------------------------------------|
| orchestrator        | backend, frontend, devops, collaboration               |
| product-ai-engineer | backend, frontend, collaboration                       |
| backend-developer   | backend, persistence, security                         |
| frontend-developer  | frontend, security                                     |
| devops-engineer     | devops, pr-workflow, security                          |

### Product Manager

| Agent           | Skills Injected           |
|-----------------|---------------------------|
| product-manager | collaboration             |

### Designer

| Agent    | Skills Injected           |
|----------|---------------------------|
| designer | collaboration (figma + atlassian always) |

---

## Available Skill Files

| Layer        | Detected           | Skill File                              | Status            |
|--------------|--------------------|-----------------------------------------|-------------------|
| backend      | Java + Spring Boot | `backend/java-21-spring-boot.md`        | available         |
| backend      | Rust + MCP         | `backend/rust-1-95-mcp.md`              | available         |
| backend      | Go + Chi           | `backend/go-chi.md`                     | not yet available |
| backend      | Python + Django    | `backend/python-django.md`              | not yet available |
| frontend     | Angular            | `frontend/angular-21.md`               | available         |
| frontend     | React              | `frontend/react-19.md`                  | available         |
| persistence  | JPA + PostgreSQL   | `persistence/jpa-postgres.md`           | available         |
| devops       | GitHub Actions     | `devops/github-actions.md`              | available         |
| devops       | PR Workflow (always) | `devops/pr-workflow.md`               | available         |
| security     | (always)           | `security/cross-cutting.md`             | available         |
| collaboration| Atlassian          | `collaboration/atlassian.md`            | available         |
| collaboration| Figma              | `collaboration/figma.md`                | available         |
| collaboration| GitHub Issues      | `collaboration/github-issues.md`        | available         |
| collaboration| Product            | `collaboration/product.md`              | available         |
| collaboration| Design             | `collaboration/design.md`               | available         |

---

## How Injection Works

**Multi-agent:** agent templates contain include directives replaced with full skill content at Onboarding Step 6. The resolved agent file is self-contained — no external skill references at runtime.

**Solo:** each skill is written as a standalone convention file to `.claude/stacks/<category>.md`. Claude loads these on demand per the Knowledge Registry in `CLAUDE.md`.

```
{{include stacks/backend.md}}   → .claude/agents/backend-developer.md  (multi-agent)
                                → .claude/stacks/backend.md        (solo)
```

---

## Adding a New Skill

1. Create the file in `agentic-setup/stacks/<category>/`
2. Follow existing skill format: overview table, rules, pitfalls, see also
3. Add it to the Available Skill Files table above
4. Add it to the mapping table in `onboarding-discovery.md` Step 3

Never reference a skill file that does not exist — see Hard Rules in `CLAUDE.md`.

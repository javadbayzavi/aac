# AgenticAsCode (AAC)

Reusable framework that onboards any project into an agentic development workflow — supports developer, product manager, and designer personas.

---

## What It Does

AAC scaffolds a project's Claude setup based on its tech-stack and your role. Run it once on any project and get:

- A tailored `CLAUDE.md` with the right workflow for your persona
- Stack convention files in `.claude/stacks/`
- Agent definitions (multi-agent mode)
- Session continuity files

---

## How to Use

### Option 1 — MCP Server (recommended)

Install the binary:

```bash
brew install javadbayzavi/tap/3t-scaffold-mcp
# or
cargo install --git https://github.com/javadbayzavi/aac 3t-scaffold-mcp
```

Register with Claude Code:

```bash
claude mcp add --scope user 3t-scaffold 3t-scaffold-mcp
```

Then in any Claude Code session:

> "Use scaffold_inspect on this project"

Claude will ask for your project path, description, persona, and mode — then scaffold everything automatically.

### Option 2 — Claude Code (this repo)

Open this repo in Claude Code. The `CLAUDE.md` drives the Inspector → Onboarding → Configurator workflow interactively.

---

## Personas

| Persona | What gets set up |
|---|---|
| Developer | Engineering agents + stack conventions |
| Product Manager | Backlog workflow + Jira/Confluence conventions |
| Designer | Component audit + handoff workflow |

---

## Available Stacks

| Category | Stack |
|---|---|
| Backend | `rust-1-95-mcp`, `java-21-spring-boot` |
| Frontend | `angular-21`, `react-19` |
| Persistence | `jpa-postgres` |
| DevOps | `github-actions`, `pr-workflow` |
| Security | `cross-cutting` |
| Collaboration | `atlassian`, `figma`, `github-issues`, `product`, `design` |

## MCP Tools

| Tool | Description |
|---|---|
| `scaffold_inspect` | Detects onboarding status and tech-stack signals |
| `scaffold_onboard` | Writes CLAUDE.md, PROJECT.yaml, and stacks |
| `scaffold_configure` | Adds or removes skills from an onboarded project |

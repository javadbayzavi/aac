# AgenticAsCode (AAC)

Reusable framework that onboards any project into an agentic development workflow — supports developer, product manager, and designer personas.

---

## Two Ways to Scaffold

This repo provides two entry points that produce the same output.

### Option 1 — MCP Server

The MCP server exposes three tools that any Claude Code session can call — no need to open this repo.

Install:
```bash
curl -fsSL https://raw.githubusercontent.com/javadbayzavi/aac/main/install.sh | sh
```

Register once:
```bash
claude mcp add --scope user scaffold ~/.local/bin/scaffold-mcp
```

Then from any project, any Claude Code session:
> "Use scaffold_inspect on this project"

Claude collects your persona and mode, calls the tools, and writes everything to your project.

**How it works:** the binary installs to `~/.local/bin/` and on startup clones this repo to `~/.scaffold/repo/`, reading templates and stacks from there at runtime — always up to date.

---

### Option 2 — Claude Code (this repo)

Open this repo directly in Claude Code. `CLAUDE.md` drives the same Inspector → Onboarding → Configurator workflow interactively using Claude's native conversation and `AskUserQuestion` prompts.

No binary needed. Claude reads the workflow files and follows them step by step.

---

## What Gets Written to Your Project

```
<project>/
├── CLAUDE.md                        # Tailored workflow for your persona + mode
└── .claude/
    ├── PROJECT.yaml                 # Project metadata and tech-stack
    ├── stacks/                      # Stack convention files (one per detected tech)
    ├── protocols/FEATURE_PLAN.json  # Feature plan schema (developer only)
    ├── active-plan.json             # Session continuity
    └── agents/                      # Agent definitions (multi-agent mode only)
```

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

---

## MCP Tools

| Tool | Equivalent workflow |
|---|---|
| `scaffold_inspect` | Inspector role in `CLAUDE.md` |
| `scaffold_onboard` | `.claude/workflows/onboarding-discovery.md` |
| `scaffold_configure` | `.claude/workflows/project-configurator.md` |

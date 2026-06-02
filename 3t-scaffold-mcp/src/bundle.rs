// All agentic-setup files bundled at compile time

// CLAUDE templates
pub const CLAUDE_SOLO: &str = include_str!("../../agentic-setup/CLAUDE.solo-template.md");
pub const CLAUDE_MULTI_AGENT: &str = include_str!("../../agentic-setup/CLAUDE-multi-agent-template.md");
pub const CLAUDE_PM_SOLO: &str = include_str!("../../agentic-setup/CLAUDE.pm-solo-template.md");
pub const CLAUDE_DESIGNER_SOLO: &str = include_str!("../../agentic-setup/CLAUDE.designer-solo-template.md");

// PROJECT.yaml template
pub const PROJECT_YAML: &str = include_str!("../../agentic-setup/PROJECT.yaml");

// Session continuity files
pub const ACTIVE_PLAN: &str = include_str!("../../agentic-setup/docs/active-plan.json");
pub const ACTIVE_SPRINT: &str = include_str!("../../agentic-setup/docs/active-sprint.json");
pub const ACTIVE_DESIGN: &str = include_str!("../../agentic-setup/docs/active-design.json");
pub const FEATURE_PLAN: &str = include_str!("../../agentic-setup/docs/FEATURE_PLAN.json");

// Agent templates (multi-agent only)
pub const AGENT_ORCHESTRATOR: &str = include_str!("../../agentic-setup/agents/orchestrator.template.md");
pub const AGENT_PRODUCT_AI: &str = include_str!("../../agentic-setup/agents/product-ai-engineer.template.md");
pub const AGENT_BACKEND: &str = include_str!("../../agentic-setup/agents/backend-developer.template.md");
pub const AGENT_FRONTEND: &str = include_str!("../../agentic-setup/agents/frontend-developer.template.md");
pub const AGENT_DEVOPS: &str = include_str!("../../agentic-setup/agents/devops-engineer.template.md");
// Reserved for future multi-agent PM and designer support
// pub const AGENT_PM: &str = include_str!("../../agentic-setup/agents/product-manager.template.md");
// pub const AGENT_DESIGNER: &str = include_str!("../../agentic-setup/agents/designer.template.md");

// Stacks
pub const STACK_JAVA_SPRING: &str = include_str!("../../agentic-setup/stacks/backend/java-21-spring-boot.md");
pub const STACK_RUST_MCP: &str = include_str!("../../agentic-setup/stacks/backend/rust-1-95-mcp.md");
pub const STACK_ANGULAR: &str = include_str!("../../agentic-setup/stacks/frontend/angular-21.md");
pub const STACK_REACT: &str = include_str!("../../agentic-setup/stacks/frontend/react-19.md");
pub const STACK_JPA_POSTGRES: &str = include_str!("../../agentic-setup/stacks/persistence/jpa-postgres.md");
pub const STACK_GITHUB_ACTIONS: &str = include_str!("../../agentic-setup/stacks/devops/github-actions.md");
pub const STACK_PR_WORKFLOW: &str = include_str!("../../agentic-setup/stacks/devops/pr-workflow.md");
pub const STACK_SECURITY: &str = include_str!("../../agentic-setup/stacks/security/cross-cutting.md");
pub const STACK_ATLASSIAN: &str = include_str!("../../agentic-setup/stacks/collaboration/atlassian.md");
pub const STACK_FIGMA: &str = include_str!("../../agentic-setup/stacks/collaboration/figma.md");
pub const STACK_GITHUB_ISSUES: &str = include_str!("../../agentic-setup/stacks/collaboration/github-issues.md");
pub const STACK_PRODUCT: &str = include_str!("../../agentic-setup/stacks/collaboration/product.md");
pub const STACK_DESIGN: &str = include_str!("../../agentic-setup/stacks/collaboration/design.md");

/// Resolve a stack name to its bundled content
pub fn stack_content(name: &str) -> Option<&'static str> {
    match name {
        "java-21-spring-boot" => Some(STACK_JAVA_SPRING),
        "rust-1-95-mcp" => Some(STACK_RUST_MCP),
        "angular-21" => Some(STACK_ANGULAR),
        "react-19" => Some(STACK_REACT),
        "jpa-postgres" => Some(STACK_JPA_POSTGRES),
        "github-actions" => Some(STACK_GITHUB_ACTIONS),
        "pr-workflow" => Some(STACK_PR_WORKFLOW),
        "cross-cutting" => Some(STACK_SECURITY),
        "atlassian" => Some(STACK_ATLASSIAN),
        "figma" => Some(STACK_FIGMA),
        "github-issues" => Some(STACK_GITHUB_ISSUES),
        "product" => Some(STACK_PRODUCT),
        "design" => Some(STACK_DESIGN),
        _ => None,
    }
}

/// Detect stack names from tech-stack signals
pub fn detect_stacks(signals: &[String]) -> Vec<&'static str> {
    let mut stacks = vec![];

    if signals.iter().any(|s| s == "Cargo.toml") {
        stacks.push("rust-1-95-mcp");
    }
    if signals.iter().any(|s| s == "pom.xml" || s == "build.gradle") {
        stacks.push("java-21-spring-boot");
    }
    if signals.iter().any(|s| s == "go.mod") {
        // no stack available yet
    }
    if signals.iter().any(|s| s == "package.json") {
        // detect angular vs react later — default to none for now
    }
    if signals.iter().any(|s| s == ".github") {
        stacks.push("github-actions");
    }

    // Always inject
    stacks.push("pr-workflow");
    stacks.push("cross-cutting");

    stacks
}

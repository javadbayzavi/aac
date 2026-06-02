use std::fs;
use std::path::PathBuf;

use crate::sync::agentic_setup_dir;

pub fn read(relative_path: &str) -> Result<String, String> {
    let path = agentic_setup_dir().join(relative_path);
    fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))
}

pub fn claude_template(persona: &str, mode: &str) -> Result<String, String> {
    let file = match (persona, mode) {
        ("product-manager", _) => "CLAUDE.pm-solo-template.md",
        ("designer", _) => "CLAUDE.designer-solo-template.md",
        (_, "multi-agent") => "CLAUDE-multi-agent-template.md",
        _ => "CLAUDE.solo-template.md",
    };
    read(file)
}

pub fn stack_content(name: &str) -> Option<String> {
    let path = stack_path(name)?;
    fs::read_to_string(agentic_setup_dir().join(path)).ok()
}

pub fn stack_path(name: &str) -> Option<&'static str> {
    match name {
        "java-21-spring-boot" => Some("stacks/backend/java-21-spring-boot.md"),
        "rust-1-95-mcp" => Some("stacks/backend/rust-1-95-mcp.md"),
        "angular-21" => Some("stacks/frontend/angular-21.md"),
        "react-19" => Some("stacks/frontend/react-19.md"),
        "jpa-postgres" => Some("stacks/persistence/jpa-postgres.md"),
        "github-actions" => Some("stacks/devops/github-actions.md"),
        "pr-workflow" => Some("stacks/devops/pr-workflow.md"),
        "cross-cutting" => Some("stacks/security/cross-cutting.md"),
        "atlassian" => Some("stacks/collaboration/atlassian.md"),
        "figma" => Some("stacks/collaboration/figma.md"),
        "github-issues" => Some("stacks/collaboration/github-issues.md"),
        "product" => Some("stacks/collaboration/product.md"),
        "design" => Some("stacks/collaboration/design.md"),
        _ => None,
    }
}

pub fn available_stacks() -> Vec<&'static str> {
    vec![
        "java-21-spring-boot",
        "rust-1-95-mcp",
        "angular-21",
        "react-19",
        "jpa-postgres",
        "github-actions",
        "pr-workflow",
        "cross-cutting",
        "atlassian",
        "figma",
        "github-issues",
        "product",
        "design",
    ]
}

pub fn detect_stacks(signals: &[String]) -> Vec<&'static str> {
    let mut stacks = vec![];
    if signals.iter().any(|s| s == "Cargo.toml") {
        stacks.push("rust-1-95-mcp");
    }
    if signals
        .iter()
        .any(|s| s == "pom.xml" || s == "build.gradle")
    {
        stacks.push("java-21-spring-boot");
    }
    if signals.iter().any(|s| s == ".github") {
        stacks.push("github-actions");
    }
    stacks.push("pr-workflow");
    stacks.push("cross-cutting");
    stacks
}

pub fn agent_template(name: &str) -> Result<String, String> {
    read(&format!("agents/{name}.template.md"))
}

pub fn project_yaml_template() -> Result<String, String> {
    read("PROJECT.yaml")
}

pub fn session_file(persona: &str) -> (&'static str, PathBuf) {
    match persona {
        "product-manager" => (
            "active-sprint.json",
            agentic_setup_dir().join("docs/active-sprint.json"),
        ),
        "designer" => (
            "active-design.json",
            agentic_setup_dir().join("docs/active-design.json"),
        ),
        _ => (
            "active-plan.json",
            agentic_setup_dir().join("docs/active-plan.json"),
        ),
    }
}

pub fn feature_plan() -> Result<String, String> {
    read("docs/FEATURE_PLAN.json")
}

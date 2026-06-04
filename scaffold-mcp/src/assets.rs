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

/// Map a skill name to its tech_stack category, mirroring the stacks/ directory
/// layout. Backends without a skill file yet (go-chi, python-django) still map
/// to "backend". Collaboration skills fall through to "collaboration".
pub fn stack_category(name: &str) -> &'static str {
    match name {
        "java-21-spring-boot" | "rust-1-95-mcp" | "go-chi" | "python-django" => "backend",
        "angular-21" | "react-19" => "frontend",
        "jpa-postgres" => "persistence",
        "github-actions" | "pr-workflow" => "devops",
        "cross-cutting" => "security",
        _ => "collaboration",
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
    let has = |name: &str| signals.iter().any(|s| s == name);
    let mut stacks = vec![];

    // --- backend ---
    if has("Cargo.toml") {
        stacks.push("rust-1-95-mcp");
    }
    let is_java = has("pom.xml") || has("build.gradle");
    if is_java {
        stacks.push("java-21-spring-boot");
    }
    // Detected but no skill file exists yet — these flow through to `skipped`
    // in the onboard handler (stack_content returns None), matching the
    // markdown survey's "note as skipped, do not block" behaviour.
    if has("go.mod") {
        stacks.push("go-chi");
    }
    if has("requirements.txt") || has("pyproject.toml") {
        stacks.push("python-django");
    }

    // --- frontend ---
    // angular.json is an unambiguous Angular CLI marker; a Next.js config
    // implies React. A bare package.json can't be disambiguated from file
    // names alone (could be a Node backend, Vue, Svelte, …), so it is left
    // for the user to add explicitly via scaffold_configure.
    if has("angular.json") {
        stacks.push("angular-21");
    } else if has("next.config.js") || has("next.config.mjs") || has("next.config.ts") {
        stacks.push("react-19");
    }

    // --- persistence ---
    // A Java/Spring project shipping a docker-compose almost always backs onto
    // a relational DB; jpa-postgres is the only persistence skill available.
    if is_java && (has("docker-compose.yml") || has("docker-compose.yaml")) {
        stacks.push("jpa-postgres");
    }

    // --- devops + security (always) ---
    if has(".github") {
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

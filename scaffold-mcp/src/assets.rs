use std::fs;
use std::path::PathBuf;

use crate::sync::agentic_setup_dir;

pub fn read(relative_path: &str) -> Result<String, String> {
    let path = agentic_setup_dir().join(relative_path);
    fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))
}

/// Message shown when a tool needs templates but the synced repo isn't ready.
pub const NOT_READY_MSG: &str = "Templates aren't available yet. The server clones them from GitHub into ~/.scaffold/repo/ on startup. If this is the first run it may still be downloading — wait a few seconds and try again. If it persists, check your network connection.";

/// Whether the synced template repo is present and usable. `PROJECT.yaml` is
/// checked out near the end of a clone, so its presence is a good proxy for
/// "the synced repo finished cloning and is readable". Tools that read
/// templates check this first to give a clear message instead of a raw
/// file-read error during the first-run (or offline) window.
pub fn is_ready() -> bool {
    agentic_setup_dir().join("PROJECT.yaml").is_file()
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

/// Agent template names available in the synced repo — basenames of
/// `agents/*.template.md`. Discovered from the filesystem so newly added
/// templates appear without code changes.
pub fn available_agents() -> Vec<String> {
    let dir = agentic_setup_dir().join("agents");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.strip_suffix(".template.md").map(String::from)
        })
        .collect();
    names.sort();
    names
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sigs(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn always_includes_pr_workflow_and_security() {
        let s = detect_stacks(&sigs(&[]));
        assert!(s.contains(&"pr-workflow"));
        assert!(s.contains(&"cross-cutting"));
    }

    #[test]
    fn detects_rust_backend() {
        assert!(detect_stacks(&sigs(&["Cargo.toml"])).contains(&"rust-1-95-mcp"));
    }

    #[test]
    fn detects_java_and_jpa_with_compose() {
        let s = detect_stacks(&sigs(&["pom.xml", "docker-compose.yml"]));
        assert!(s.contains(&"java-21-spring-boot"));
        assert!(s.contains(&"jpa-postgres"));
    }

    #[test]
    fn jpa_requires_java_not_just_compose() {
        assert!(!detect_stacks(&sigs(&["docker-compose.yml"])).contains(&"jpa-postgres"));
    }

    #[test]
    fn detects_angular_but_not_react() {
        let s = detect_stacks(&sigs(&["angular.json"]));
        assert!(s.contains(&"angular-21"));
        assert!(!s.contains(&"react-19"));
    }

    #[test]
    fn detects_react_via_next_config() {
        assert!(detect_stacks(&sigs(&["next.config.mjs"])).contains(&"react-19"));
    }

    #[test]
    fn bare_package_json_picks_no_frontend() {
        let s = detect_stacks(&sigs(&["package.json"]));
        assert!(!s.contains(&"react-19"));
        assert!(!s.contains(&"angular-21"));
    }

    #[test]
    fn go_and_python_detected_but_have_no_skill_file() {
        let s = detect_stacks(&sigs(&["go.mod", "pyproject.toml"]));
        assert!(s.contains(&"go-chi"));
        assert!(s.contains(&"python-django"));
        // No skill file → onboard reports them as skipped, not injected.
        assert!(stack_path("go-chi").is_none());
        assert!(stack_path("python-django").is_none());
    }

    #[test]
    fn stack_category_maps_each_layer() {
        assert_eq!(stack_category("rust-1-95-mcp"), "backend");
        assert_eq!(stack_category("go-chi"), "backend");
        assert_eq!(stack_category("react-19"), "frontend");
        assert_eq!(stack_category("jpa-postgres"), "persistence");
        assert_eq!(stack_category("github-actions"), "devops");
        assert_eq!(stack_category("cross-cutting"), "security");
        assert_eq!(stack_category("atlassian"), "collaboration");
    }
}

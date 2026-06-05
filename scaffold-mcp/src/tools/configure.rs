use rmcp::handler::server::common::schema_for_type;
use rmcp::handler::server::router::tool::ToolRoute;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::assets;
use crate::server::AacServer;
use crate::server::{error_result, text_result};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConfigureParams {
    /// Absolute path to the target project directory
    pub project_path: String,
    /// Operation: add-tech-stack | remove-tech-stack | add-agent | remove-agent
    pub operation: String,
    /// For tech-stack ops: the skill name (e.g. react-19, atlassian). For agent
    /// ops: the agent name (e.g. backend-developer, product-manager).
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct ConfigureResult {
    pub project_path: String,
    pub operation: String,
    pub target: String,
    pub changes: Vec<String>,
    pub next_steps: Vec<String>,
}

pub fn configure_route() -> ToolRoute<AacServer> {
    let tool = Tool::new_with_raw(
        "scaffold_configure",
        Some(std::borrow::Cow::Borrowed(
            "Configure an already-onboarded project. Do NOT run scaffold_inspect first — go directly to this tool. operation is one of: add-tech-stack, remove-tech-stack, add-agent, remove-agent. target is the skill name (tech-stack ops) or agent name (agent ops). Before calling, ask the user which operation, then which target via AskUserQuestion. Tech-stack skills — Backend: java-21-spring-boot, rust-1-95-mcp. Frontend: angular-21, react-19. Persistence: jpa-postgres. DevOps: github-actions, pr-workflow. Collaboration: atlassian, figma, github-issues, product, design. Agents (multi-agent projects only): orchestrator, product-ai-engineer, backend-developer, frontend-developer, devops-engineer, product-manager, designer. After calling, use AskUserQuestion: 'Make another change' or 'Done'.",
        )),
        schema_for_type::<ConfigureParams>(),
    );
    ToolRoute::new(tool, configure_handler)
}

async fn configure_handler(Parameters(params): Parameters<ConfigureParams>) -> CallToolResult {
    let path = Path::new(&params.project_path);
    let claude_dir = path.join(".claude");
    let project_yaml_path = claude_dir.join("PROJECT.yaml");

    if !project_yaml_path.exists() {
        return error_result(
            "Project is not onboarded. Run scaffold_inspect then scaffold_onboard first.",
        );
    }

    match params.operation.as_str() {
        "add-tech-stack" => {
            add_tech_stack(
                path,
                &claude_dir,
                &project_yaml_path,
                &params.target,
                &params.project_path,
            )
            .await
        }
        "remove-tech-stack" => remove_tech_stack(
            &claude_dir,
            &project_yaml_path,
            &params.target,
            &params.project_path,
        ),
        "add-agent" => add_agent(
            path,
            &claude_dir,
            &project_yaml_path,
            &params.target,
            &params.project_path,
        ),
        "remove-agent" => remove_agent(&claude_dir, &params.target, &params.project_path),
        _ => error_result(format!(
            "Unknown operation: {}. Supported: add-tech-stack, remove-tech-stack, add-agent, remove-agent",
            params.operation
        )),
    }
}

async fn add_tech_stack(
    _path: &Path,
    claude_dir: &Path,
    project_yaml_path: &Path,
    target: &str,
    project_path: &str,
) -> CallToolResult {
    if !assets::is_ready() {
        return error_result(assets::NOT_READY_MSG);
    }

    // Resolve stack content from bundle
    let content = match assets::stack_content(target) {
        Some(c) => c,
        None => {
            return error_result(format!(
                "No stack available for '{}'. Available: {}",
                target,
                assets::available_stacks().join(", ")
            ));
        }
    };

    // Write stack file to .claude/stacks/
    let stacks_dir = claude_dir.join("stacks");
    if let Err(e) = fs::create_dir_all(&stacks_dir) {
        return error_result(format!("Failed to create .claude/stacks/: {e}"));
    }

    let stack_file = stacks_dir.join(format!("{target}.md"));
    if let Err(e) = fs::write(&stack_file, content) {
        return error_result(format!("Failed to write stack file: {e}"));
    }

    // Update PROJECT.yaml — append to the relevant tech_stack list
    let yaml_content = match fs::read_to_string(project_yaml_path) {
        Ok(s) => s,
        Err(e) => return error_result(format!("Failed to read PROJECT.yaml: {e}")),
    };

    let category = assets::stack_category(target);
    let (updated_yaml, changed) = append_to_tech_stack(&yaml_content, category, target);

    if let Err(e) = fs::write(project_yaml_path, &updated_yaml) {
        return error_result(format!("Failed to update PROJECT.yaml: {e}"));
    }

    // Clean up inspection.json if present
    let inspection_path = claude_dir.join("inspection.json");
    if inspection_path.exists() {
        let _ = fs::remove_file(&inspection_path);
    }

    text_result(
        serde_json::to_string_pretty(&ConfigureResult {
            project_path: project_path.to_string(),
            operation: "add-tech-stack".to_string(),
            target: target.to_string(),
            next_steps: vec![
                "Call scaffold_configure again to add another skill".to_string(),
                "Done — open the project in Claude Code".to_string(),
            ],
            changes: vec![
                format!(".claude/stacks/{target}.md written"),
                if changed {
                    format!("PROJECT.yaml updated — {target} added to {category}")
                } else {
                    format!("PROJECT.yaml unchanged — {target} already listed under {category}")
                },
            ],
        })
        .unwrap(),
    )
}

fn remove_tech_stack(
    claude_dir: &Path,
    project_yaml_path: &Path,
    target: &str,
    project_path: &str,
) -> CallToolResult {
    let stack_file = claude_dir.join("stacks").join(format!("{target}.md"));
    let file_removed = stack_file.exists();
    if file_removed {
        let _ = fs::remove_file(&stack_file);
    }

    let yaml_content = match fs::read_to_string(project_yaml_path) {
        Ok(s) => s,
        Err(e) => return error_result(format!("Failed to read PROJECT.yaml: {e}")),
    };
    let category = assets::stack_category(target);
    let (updated_yaml, removed_from_yaml) = remove_from_tech_stack(&yaml_content, category, target);

    if !file_removed && !removed_from_yaml {
        return error_result(format!("'{target}' is not configured in this project."));
    }

    if removed_from_yaml && let Err(e) = fs::write(project_yaml_path, &updated_yaml) {
        return error_result(format!("Failed to update PROJECT.yaml: {e}"));
    }

    let mut changes = vec![];
    if file_removed {
        changes.push(format!(".claude/stacks/{target}.md removed"));
    }
    changes.push(if removed_from_yaml {
        format!("PROJECT.yaml updated — {target} removed from {category}")
    } else {
        format!("PROJECT.yaml unchanged — {target} was not listed under {category}")
    });
    ok_result("remove-tech-stack", target, project_path, changes)
}

fn add_agent(
    path: &Path,
    claude_dir: &Path,
    project_yaml_path: &Path,
    target: &str,
    project_path: &str,
) -> CallToolResult {
    if !assets::is_ready() {
        return error_result(assets::NOT_READY_MSG);
    }

    let yaml = match fs::read_to_string(project_yaml_path) {
        Ok(s) => s,
        Err(e) => return error_result(format!("Failed to read PROJECT.yaml: {e}")),
    };
    if yaml_scalar(&yaml, "mode") == Some("solo") {
        return error_result(
            "This project is solo mode — agents are embedded in CLAUDE.md, not .claude/agents/. \
             Re-onboard as multi-agent to use individual agent files.",
        );
    }

    let template = match assets::agent_template(target) {
        Ok(t) => t,
        Err(_) => {
            return error_result(format!(
                "No agent template '{}'. Available: {}",
                target,
                assets::available_agents().join(", ")
            ));
        }
    };

    let project_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let skills = injected_skills(claude_dir);
    let content = crate::tools::onboard::resolve_agent_template(&template, &skills, &project_name);

    let agents_dir = claude_dir.join("agents");
    if let Err(e) = fs::create_dir_all(&agents_dir) {
        return error_result(format!("Failed to create .claude/agents/: {e}"));
    }
    let agent_file = agents_dir.join(format!("{target}.md"));
    let overwrote = agent_file.exists();
    if let Err(e) = fs::write(&agent_file, content) {
        return error_result(format!("Failed to write agent file: {e}"));
    }

    let note = if overwrote {
        " (overwrote existing)"
    } else {
        ""
    };
    ok_result(
        "add-agent",
        target,
        project_path,
        vec![format!(".claude/agents/{target}.md written{note}")],
    )
}

fn remove_agent(claude_dir: &Path, target: &str, project_path: &str) -> CallToolResult {
    let agent_file = claude_dir.join("agents").join(format!("{target}.md"));
    if !agent_file.exists() {
        let current = injected_agents(claude_dir);
        let listed = if current.is_empty() {
            "none".to_string()
        } else {
            current.join(", ")
        };
        return error_result(format!(
            "No agent '{target}' in this project. Current agents: {listed}"
        ));
    }
    if let Err(e) = fs::remove_file(&agent_file) {
        return error_result(format!("Failed to remove agent file: {e}"));
    }
    ok_result(
        "remove-agent",
        target,
        project_path,
        vec![format!(".claude/agents/{target}.md removed")],
    )
}

/// Skill names currently injected into the project — the basenames of the
/// `.md` files in `.claude/stacks/`. Used to resolve agent template includes.
fn injected_skills(claude_dir: &Path) -> Vec<String> {
    let dir = claude_dir.join("stacks");
    fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.strip_suffix(".md").map(String::from)
        })
        .collect()
}

/// Agent names currently defined in the project — basenames of `.md` files in
/// `.claude/agents/`.
fn injected_agents(claude_dir: &Path) -> Vec<String> {
    let dir = claude_dir.join("agents");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.strip_suffix(".md").map(String::from)
        })
        .collect();
    names.sort();
    names
}

/// Read a top-level scalar value from PROJECT.yaml (e.g. `mode: multi-agent`),
/// tolerating trailing `# comments`. Returns the first whitespace-delimited
/// token of the value.
fn yaml_scalar<'a>(yaml: &'a str, key: &str) -> Option<&'a str> {
    yaml.lines().find_map(|line| {
        line.trim_start()
            .strip_prefix(key)?
            .strip_prefix(':')?
            .split('#')
            .next()?
            .split_whitespace()
            .next()
    })
}

fn ok_result(
    operation: &str,
    target: &str,
    project_path: &str,
    changes: Vec<String>,
) -> CallToolResult {
    text_result(
        serde_json::to_string_pretty(&ConfigureResult {
            project_path: project_path.to_string(),
            operation: operation.to_string(),
            target: target.to_string(),
            next_steps: vec![
                "Call scaffold_configure again to make another change".to_string(),
                "Done — open the project in Claude Code".to_string(),
            ],
            changes,
        })
        .unwrap(),
    )
}

/// The first whitespace-delimited token of a YAML list entry (`    - foo  # x`
/// → `foo`), or None if the line is not a `- ` entry. Tolerates trailing
/// comments carried over from the PROJECT.yaml template.
fn entry_skill(line: &str) -> Option<&str> {
    line.trim_start()
        .strip_prefix("- ")?
        .split_whitespace()
        .next()
}

/// Add `skill` under the given tech_stack `category`, preserving the file's
/// comments and indentation. No-ops if the skill is already listed (dedup). If
/// the category holds only the `none` placeholder, the skill replaces it.
/// Returns the new YAML and whether anything changed.
fn append_to_tech_stack(yaml: &str, category: &str, skill: &str) -> (String, bool) {
    let header = format!("  {category}:");
    let mut lines: Vec<String> = yaml.lines().map(String::from).collect();

    let Some(header_idx) = lines.iter().position(|l| l.trim_end() == header) else {
        // Category absent from the template — append a fresh block at the end.
        let mut out = yaml.trim_end().to_string();
        out.push_str(&format!("\n{header}\n    - {skill}\n"));
        return (out, true);
    };

    // Collect the contiguous list entries directly under the header.
    let mut entries = vec![];
    let mut i = header_idx + 1;
    while i < lines.len() && lines[i].trim_start().starts_with("- ") {
        entries.push(i);
        i += 1;
    }

    if entries
        .iter()
        .any(|&idx| entry_skill(&lines[idx]) == Some(skill))
    {
        return (yaml.to_string(), false); // already present — dedup
    }

    if let Some(&none_idx) = entries
        .iter()
        .find(|&&idx| entry_skill(&lines[idx]) == Some("none"))
    {
        lines[none_idx] = format!("    - {skill}");
    } else {
        lines.insert(header_idx + 1, format!("    - {skill}"));
    }

    let mut out = lines.join("\n");
    if yaml.ends_with('\n') {
        out.push('\n');
    }
    (out, true)
}

/// Remove `skill` from the given tech_stack `category`, preserving comments and
/// indentation. If the category empties out, restore the `none` placeholder so
/// the file stays well-formed. Returns the new YAML and whether anything changed.
fn remove_from_tech_stack(yaml: &str, category: &str, skill: &str) -> (String, bool) {
    let header = format!("  {category}:");
    let mut lines: Vec<String> = yaml.lines().map(String::from).collect();

    let Some(header_idx) = lines.iter().position(|l| l.trim_end() == header) else {
        return (yaml.to_string(), false);
    };

    let mut entries = vec![];
    let mut i = header_idx + 1;
    while i < lines.len() && lines[i].trim_start().starts_with("- ") {
        entries.push(i);
        i += 1;
    }

    let Some(&target_idx) = entries
        .iter()
        .find(|&&idx| entry_skill(&lines[idx]) == Some(skill))
    else {
        return (yaml.to_string(), false);
    };
    lines.remove(target_idx);

    // If the category now has no entries, restore the `none` placeholder.
    let still_has_entry = lines
        .get(header_idx + 1)
        .is_some_and(|l| l.trim_start().starts_with("- "));
    if !still_has_entry {
        lines.insert(header_idx + 1, "    - none".to_string());
    }

    let mut out = lines.join("\n");
    if yaml.ends_with('\n') {
        out.push('\n');
    }
    (out, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    const YAML: &str = "tech_stack:\n  backend:\n    - rust-1-95-mcp\n  frontend:\n    - none        # e.g., angular-21, react-19\n  devops:\n    - github-actions\n    - pr-workflow\n";

    #[test]
    fn entry_skill_strips_comment_and_ignores_non_entries() {
        assert_eq!(entry_skill("    - react-19   # hint"), Some("react-19"));
        assert_eq!(entry_skill("  backend:"), None);
    }

    #[test]
    fn yaml_scalar_reads_value_past_comment() {
        let y = "project:\n  mode: multi-agent    # solo | multi-agent\n";
        assert_eq!(yaml_scalar(y, "mode"), Some("multi-agent"));
        assert_eq!(yaml_scalar(y, "missing"), None);
    }

    #[test]
    fn append_replaces_none_placeholder() {
        let (out, changed) = append_to_tech_stack(YAML, "frontend", "react-19");
        assert!(changed);
        assert!(out.contains("    - react-19"));
        assert!(!out.contains("- none"));
    }

    #[test]
    fn append_dedups_already_present() {
        let (out, changed) = append_to_tech_stack(YAML, "devops", "github-actions");
        assert!(!changed);
        assert_eq!(out, YAML);
    }

    #[test]
    fn append_inserts_alongside_existing() {
        let (out, changed) = append_to_tech_stack(YAML, "backend", "java-21-spring-boot");
        assert!(changed);
        assert!(out.contains("- rust-1-95-mcp"));
        assert!(out.contains("- java-21-spring-boot"));
    }

    #[test]
    fn append_creates_missing_category() {
        let (out, changed) = append_to_tech_stack(YAML, "persistence", "jpa-postgres");
        assert!(changed);
        assert!(out.contains("  persistence:\n    - jpa-postgres"));
    }

    #[test]
    fn remove_keeps_other_entries() {
        let two = "tech_stack:\n  backend:\n    - rust-1-95-mcp\n    - java-21-spring-boot\n";
        let (out, changed) = remove_from_tech_stack(two, "backend", "java-21-spring-boot");
        assert!(changed);
        assert!(out.contains("- rust-1-95-mcp"));
        assert!(!out.contains("java-21-spring-boot"));
    }

    #[test]
    fn remove_last_entry_restores_none() {
        let (out, changed) = remove_from_tech_stack(YAML, "backend", "rust-1-95-mcp");
        assert!(changed);
        assert!(out.contains("  backend:\n    - none"));
    }

    #[test]
    fn remove_absent_is_noop() {
        let (out, changed) = remove_from_tech_stack(YAML, "backend", "angular-21");
        assert!(!changed);
        assert_eq!(out, YAML);
    }
}

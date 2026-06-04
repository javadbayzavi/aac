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
    /// Operation: add-tech-stack
    pub operation: String,
    /// Skill name to add (e.g. react-19, angular-21, atlassian, github-issues)
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
            "Add a tech-stack skill to an already-onboarded project. Do NOT run scaffold_inspect first — go directly to this tool. Only needs project_path and target. Before calling, ask: (1) project path, (2) category via AskUserQuestion: Backend / Frontend / Persistence / DevOps / Collaboration, (3) specific skill via AskUserQuestion based on category — Backend: java-21-spring-boot, rust-1-95-mcp. Frontend: angular-21, react-19. Persistence: jpa-postgres. DevOps: github-actions, pr-workflow. Collaboration: atlassian, figma, github-issues, product, design. After calling, use AskUserQuestion: 'Add another skill' or 'Done'.",
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
        _ => error_result(format!(
            "Unknown operation: {}. Supported: add-tech-stack",
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

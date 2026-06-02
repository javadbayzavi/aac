use rmcp::handler::server::common::schema_for_type;
use rmcp::handler::server::router::tool::ToolRoute;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::bundle;
use crate::server::{error_result, text_result};
use crate::server::AacServer;

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
            "Add a tech-stack skill to an already-onboarded project. Writes the skill file to .claude/stacks/ and updates PROJECT.yaml. Supported operation: add-tech-stack.",
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
        "add-tech-stack" => add_tech_stack(path, &claude_dir, &project_yaml_path, &params.target, &params.project_path).await,
        _ => error_result(format!("Unknown operation: {}. Supported: add-tech-stack", params.operation)),
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
    let content = match bundle::stack_content(target) {
        Some(c) => c,
        None => {
            let available = ["rust-1-95-mcp", "java-21-spring-boot", "angular-21", "react-19",
                           "jpa-postgres", "github-actions", "pr-workflow", "cross-cutting",
                           "atlassian", "figma", "github-issues", "product", "design"];
            return error_result(format!(
                "No stack available for '{}'. Available: {}",
                target,
                available.join(", ")
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

    let category = stack_category(target);
    let updated_yaml = append_to_tech_stack(yaml_content, category, target);

    if let Err(e) = fs::write(project_yaml_path, &updated_yaml) {
        return error_result(format!("Failed to update PROJECT.yaml: {e}"));
    }

    // Clean up inspection.json if present
    let inspection_path = claude_dir.join("inspection.json");
    if inspection_path.exists() {
        let _ = fs::remove_file(&inspection_path);
    }

    text_result(serde_json::to_string_pretty(&ConfigureResult {
        project_path: project_path.to_string(),
        operation: "add-tech-stack".to_string(),
        target: target.to_string(),
        next_steps: vec![
            "Call scaffold_configure again to add another skill".to_string(),
            "Done — open the project in Claude Code".to_string(),
        ],
        changes: vec![
            format!(".claude/stacks/{target}.md written"),
            format!("PROJECT.yaml updated — {target} added to {category}"),
        ],
    }).unwrap())
}

fn stack_category(name: &str) -> &'static str {
    match name {
        "java-21-spring-boot" | "rust-1-95-mcp" => "backend",
        "angular-21" | "react-19" => "frontend",
        "jpa-postgres" => "persistence",
        "github-actions" | "pr-workflow" => "devops",
        "cross-cutting" => "security",
        _ => "collaboration",
    }
}

fn append_to_tech_stack(yaml: String, category: &str, skill: &str) -> String {
    // Find the category list and append the skill
    // Looks for pattern like "  backend:\n    - existing" and appends "    - skill"
    let search = format!("  {category}:\n");
    if let Some(pos) = yaml.find(&search) {
        let insert_pos = pos + search.len();
        let new_entry = format!("    - {skill}\n");
        let mut result = yaml.clone();
        result.insert_str(insert_pos, &new_entry);
        result
    } else {
        // Category doesn't exist yet — append it
        format!("{yaml}\n  {category}:\n    - {skill}\n")
    }
}

use rmcp::handler::server::common::schema_for_type;
use rmcp::handler::server::router::tool::ToolRoute;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::server::AacServer;
use crate::server::{error_result, text_result};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InspectParams {
    /// Absolute path to the target project directory
    pub project_path: String,
    /// One or two sentences about what the project does
    pub description: String,
    /// Role persona: developer | product-manager | designer
    pub persona: String,
    /// Workflow mode: solo | multi-agent
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct InspectResult {
    pub project_path: String,
    pub onboarding_status: String,
    pub project_state: String,
    pub tech_stack_signals: Vec<String>,
    pub inspection_json_written: bool,
    pub next_steps: Vec<String>,
}

pub fn inspect_route() -> ToolRoute<AacServer> {
    let tool = Tool::new_with_raw(
        "scaffold_inspect",
        Some(std::borrow::Cow::Borrowed(
            "Inspect a project to check its onboarding status. Before calling: ask the user for project_path, description, persona (Developer / Product Manager / Designer), and mode (Solo / Multi-agent) — never infer these. After calling: present findings as a table, then YOU MUST call AskUserQuestion tool with the next_steps values as selectable options. Do not present next_steps as plain text. Do not call any other tool until user selects an option.",
        )),
        schema_for_type::<InspectParams>(),
    );
    ToolRoute::new(tool, inspect_handler)
}

async fn inspect_handler(Parameters(params): Parameters<InspectParams>) -> CallToolResult {
    let path = Path::new(&params.project_path);

    if !path.exists() {
        return error_result(format!("Path not found: {}", params.project_path));
    }

    let project_yaml_path = path.join(".claude").join("PROJECT.yaml");
    let project_yaml_exists = project_yaml_path.exists();
    let signals = detect_signals(path);

    let onboarding_status = if project_yaml_exists {
        if is_drift(&project_yaml_path) {
            "DRIFT"
        } else {
            "ONBOARDED"
        }
    } else {
        "NOT_ONBOARDED"
    };

    let project_state = if signals.is_empty() {
        "GREENFIELD"
    } else {
        "EXISTING"
    };

    let claude_dir = path.join(".claude");
    if let Err(e) = fs::create_dir_all(&claude_dir) {
        return error_result(format!("Failed to create .claude dir: {e}"));
    }

    let inspected_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    let inspection = serde_json::json!({
        "inspected_at": inspected_at,
        "project_path": params.project_path,
        "description": params.description,
        "persona": params.persona,
        "mode": params.mode,
        "onboarding_status": onboarding_status,
        "project_state": project_state,
        "tech_stack_signals": signals,
    });

    if let Err(e) = fs::write(
        claude_dir.join("inspection.json"),
        serde_json::to_string_pretty(&inspection).unwrap(),
    ) {
        return error_result(format!("Failed to write inspection.json: {e}"));
    }

    let next_steps = match onboarding_status {
        "NOT_ONBOARDED" => vec![
            "Call scaffold_onboard to set up this project".to_string(),
            "Cancel — do nothing".to_string(),
        ],
        "DRIFT" => vec![
            "Call scaffold_configure to add or remove skills".to_string(),
            "Call scaffold_onboard to re-onboard and regenerate from updated templates".to_string(),
            "Cancel — do nothing".to_string(),
        ],
        _ => vec![
            "Call scaffold_configure to add or remove skills".to_string(),
            "Cancel — do nothing".to_string(),
        ],
    };

    let result = InspectResult {
        project_path: params.project_path,
        onboarding_status: onboarding_status.to_string(),
        project_state: project_state.to_string(),
        tech_stack_signals: signals,
        inspection_json_written: true,
        next_steps,
    };

    text_result(serde_json::to_string_pretty(&result).unwrap())
}

fn is_drift(project_yaml: &Path) -> bool {
    // Binary mtime > PROJECT.yaml mtime means stacks were updated and rebuilt since onboarding
    let binary_mtime = std::env::current_exe()
        .ok()
        .and_then(|p| fs::metadata(p).ok())
        .and_then(|m| m.modified().ok());

    let yaml_mtime = fs::metadata(project_yaml)
        .ok()
        .and_then(|m| m.modified().ok());

    match (binary_mtime, yaml_mtime) {
        (Some(bin), Some(yaml)) => bin > yaml,
        _ => false,
    }
}

fn detect_signals(path: &Path) -> Vec<String> {
    [
        "Cargo.toml",
        "pom.xml",
        "build.gradle",
        "go.mod",
        "package.json",
        "requirements.txt",
        "pyproject.toml",
        "docker-compose.yml",
        ".github",
    ]
    .iter()
    .filter(|f| path.join(f).exists())
    .map(|f| f.to_string())
    .collect()
}

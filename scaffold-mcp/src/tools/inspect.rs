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
use crate::sync::agentic_setup_dir;

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
            "Inspect a project. IMPORTANT: Do NOT call this tool until you have collected ALL FOUR answers using AskUserQuestion: (1) project_path — absolute path, (2) description — what the project does, (3) persona — Developer / Product Manager / Designer, (4) mode — Solo / Multi-agent. Calling this tool without all four answers will fail. After calling: present findings as a table, then call AskUserQuestion with the next_steps options.",
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

/// Drift means a template or stack source in the synced repo has been updated
/// (e.g. via `git pull` on startup) after this project was onboarded. This
/// approximates the markdown workflow's `find agentic-setup/ ... -newer
/// PROJECT.yaml` by walking the synced source tree and comparing modification
/// times against PROJECT.yaml — the binary's own mtime is irrelevant.
fn is_drift(project_yaml: &Path) -> bool {
    let yaml_mtime = match fs::metadata(project_yaml).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false,
    };

    newest_source_mtime(&agentic_setup_dir())
        .map(|newest| newest > yaml_mtime)
        .unwrap_or(false)
}

/// Recursively find the most recent mtime among template sources (.md / .yaml /
/// .json) under `dir`. Returns None if the directory is absent — e.g. the
/// startup sync has not completed yet, in which case we report no drift.
fn newest_source_mtime(dir: &Path) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => {
                if let Some(t) = newest_source_mtime(&path) {
                    newest = Some(newest.map_or(t, |cur| cur.max(t)));
                }
            }
            Ok(_) => {
                let is_source = matches!(
                    path.extension().and_then(|e| e.to_str()),
                    Some("md") | Some("yaml") | Some("json")
                );
                if is_source {
                    if let Ok(t) = entry.metadata().and_then(|m| m.modified()) {
                        newest = Some(newest.map_or(t, |cur| cur.max(t)));
                    }
                }
            }
            Err(_) => continue,
        }
    }
    newest
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

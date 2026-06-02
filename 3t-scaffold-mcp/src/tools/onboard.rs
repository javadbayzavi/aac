use rmcp::handler::server::common::schema_for_type;
use rmcp::handler::server::router::tool::ToolRoute;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::bundle;
use crate::server::{error_result, text_result};
use crate::server::AacServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OnboardParams {
    pub project_path: String,
}

#[derive(Debug, Serialize)]
pub struct OnboardResult {
    pub project_path: String,
    pub persona: String,
    pub mode: String,
    pub files_written: Vec<String>,
    pub skills_injected: Vec<String>,
    pub skipped: Vec<String>,
    pub claude_md_overwritten: bool,
    pub next_steps: Vec<String>,
}

pub fn onboard_route() -> ToolRoute<AacServer> {
    let tool = Tool::new_with_raw(
        "scaffold_onboard",
        Some(std::borrow::Cow::Borrowed(
            "Onboard a project into the agentic workflow. Only call this after scaffold_inspect has been called AND the user has explicitly confirmed they want to proceed. If CLAUDE.md already exists, ask: Overwrite / Backup and overwrite / Cancel before calling.",
        )),
        schema_for_type::<OnboardParams>(),
    );
    ToolRoute::new(tool, onboard_handler)
}

async fn onboard_handler(Parameters(params): Parameters<OnboardParams>) -> CallToolResult {
    let path = Path::new(&params.project_path);
    let claude_dir = path.join(".claude");
    let inspection_path = claude_dir.join("inspection.json");

    if !inspection_path.exists() {
        return error_result("inspection.json not found. Run scaffold_inspect first.");
    }

    let inspection_raw = match fs::read_to_string(&inspection_path) {
        Ok(s) => s,
        Err(e) => return error_result(format!("Failed to read inspection.json: {e}")),
    };

    let inspection: serde_json::Value = match serde_json::from_str(&inspection_raw) {
        Ok(v) => v,
        Err(e) => return error_result(format!("Failed to parse inspection.json: {e}")),
    };

    if claude_dir.join("PROJECT.yaml").exists() {
        return error_result("Project already onboarded. Use scaffold_configure for changes.");
    }

    // Warn about existing CLAUDE.md — Claude must have asked user before calling this
    let claude_md_existed = path.join("CLAUDE.md").exists();

    let persona = inspection["persona"].as_str().unwrap_or("developer").to_string();
    let mode = inspection["mode"].as_str().unwrap_or("solo").to_string();
    let project_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let description = inspection["description"].as_str().unwrap_or("").to_string();
    let signals: Vec<String> = inspection["tech_stack_signals"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let mut files_written: Vec<String> = vec![];
    let mut skills_injected: Vec<String> = vec![];
    let skipped: Vec<String> = vec![];

    // Create .claude dirs
    for dir in &["stacks", "protocols"] {
        if let Err(e) = fs::create_dir_all(claude_dir.join(dir)) {
            return error_result(format!("Failed to create .claude/{dir}: {e}"));
        }
    }

    // Detect and write stacks
    let detected = bundle::detect_stacks(&signals);
    for stack_name in &detected {
        if let Some(content) = bundle::stack_content(stack_name) {
            let dest = claude_dir.join("stacks").join(format!("{stack_name}.md"));
            if let Err(e) = fs::write(&dest, content) {
                return error_result(format!("Failed to write stack {stack_name}: {e}"));
            }
            skills_injected.push(stack_name.to_string());
            files_written.push(format!(".claude/stacks/{stack_name}.md"));
        }
    }

    // Persona-specific stacks
    match persona.as_str() {
        "product-manager" => {
            let _ = fs::write(claude_dir.join("stacks/product.md"), bundle::STACK_PRODUCT);
            let _ = fs::write(claude_dir.join("stacks/atlassian.md"), bundle::STACK_ATLASSIAN);
            skills_injected.extend(["product".into(), "atlassian".into()]);
            files_written.extend([".claude/stacks/product.md".into(), ".claude/stacks/atlassian.md".into()]);
        }
        "designer" => {
            let _ = fs::write(claude_dir.join("stacks/design.md"), bundle::STACK_DESIGN);
            let _ = fs::write(claude_dir.join("stacks/figma.md"), bundle::STACK_FIGMA);
            skills_injected.extend(["design".into(), "figma".into()]);
            files_written.extend([".claude/stacks/design.md".into(), ".claude/stacks/figma.md".into()]);
        }
        _ => {}
    }

    // Write PROJECT.yaml
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    let project_yaml = bundle::PROJECT_YAML
        .replace("{{ project_name }}", &project_name)
        .replace("{{ project_description }}", &description)
        .replace("{{ project_path }}", &params.project_path)
        .replace("{{ persona }}", &persona)
        .replace("{{ mode }}", &mode)
        .replace("{{ status }}", "complete")
        .replace("{{ project_lead }}", "")
        .replace("{{ operator_notes }}", "")
        .replace("{{ onboarded_at }}", &now)
        .replace("{{ default_model }}", "claude-sonnet-4-6")
        .replace("{{ backend_skill }}", skills_injected.first().map(String::as_str).unwrap_or("none"))
        .replace("{{ frontend_skill }}", "none")
        .replace("{{ persistence_skill }}", "none")
        .replace("{{ branching_model }}", "trunk")
        .replace("{{ commit_pattern }}", "feat|fix|docs|style|refactor|test|chore")
        .replace("{{ pr_template }}", "standard")
        .replace("{{ issue_tracker }}", "github-projects")
        .replace("{{ docs_tool }}", "confluence")
        .replace("{{ board_reference }}", "")
        .replace("{{ design_tool }}", "figma")
        .replace("{{ handoff_tool }}", "figma-dev")
        .replace("{{ component_library }}", "none");

    if let Err(e) = fs::write(claude_dir.join("PROJECT.yaml"), &project_yaml) {
        return error_result(format!("Failed to write PROJECT.yaml: {e}"));
    }
    files_written.push(".claude/PROJECT.yaml".into());

    // Write CLAUDE.md
    let tech_stack_list = skills_injected.join(", ");
    let claude_template = match (persona.as_str(), mode.as_str()) {
        ("product-manager", _) => bundle::CLAUDE_PM_SOLO,
        ("designer", _) => bundle::CLAUDE_DESIGNER_SOLO,
        (_, "multi-agent") => bundle::CLAUDE_MULTI_AGENT,
        _ => bundle::CLAUDE_SOLO,
    };

    let claude_md = claude_template
        .replace("{{project.name}}", &project_name)
        .replace("{{project.description}}", &description)
        .replace("{{project.extended_description}}", &description)
        .replace("{{project.lead}}", "")
        .replace("{{project.operator_notes}}", "")
        .replace("{{tech_stack}}", &tech_stack_list)
        .replace("{{onboarding.date}}", &now)
        .replace("{{current_state.backend}}", "existing — not audited")
        .replace("{{current_state.frontend}}", "existing — not audited")
        .replace("{{current_state.domain_model}}", "existing — not audited")
        .replace("{{current_state.testing}}", "existing — not audited")
        .replace("{{current_state.observability}}", "existing — not audited")
        .replace("{{current_state.cicd}}", "existing — not audited")
        .replace("{{current_state.epic_1}}", "—")
        .replace("{{current_state.epic_1_status}}", "—")
        .replace("{{current_state.epic_1_stories}}", "—")
        .replace("{{current_state.feature_1}}", "—")
        .replace("{{current_state.feature_1_status}}", "—")
        .replace("{{current_state.feature_1_story}}", "—")
        .replace("{{pm_tools.issue_tracker}}", "github-projects")
        .replace("{{pm_tools.docs}}", "confluence")
        .replace("{{pm_tools.board}}", "")
        .replace("{{design_tools.design}}", "figma")
        .replace("{{design_tools.handoff}}", "figma-dev")
        .replace("{{design_tools.component_library}}", "none");

    if let Err(e) = fs::write(path.join("CLAUDE.md"), &claude_md) {
        return error_result(format!("Failed to write CLAUDE.md: {e}"));
    }
    files_written.push("CLAUDE.md".into());

    // Session continuity
    let (session_name, session_content) = match persona.as_str() {
        "product-manager" => ("active-sprint.json", bundle::ACTIVE_SPRINT),
        "designer" => ("active-design.json", bundle::ACTIVE_DESIGN),
        _ => ("active-plan.json", bundle::ACTIVE_PLAN),
    };
    let _ = fs::write(claude_dir.join(session_name), session_content);
    files_written.push(format!(".claude/{session_name}"));

    // FEATURE_PLAN for developer
    if persona == "developer" {
        let _ = fs::write(claude_dir.join("protocols/FEATURE_PLAN.json"), bundle::FEATURE_PLAN);
        files_written.push(".claude/protocols/FEATURE_PLAN.json".into());
    }

    // Agents for multi-agent developer
    if mode == "multi-agent" && persona == "developer" {
        let _ = fs::create_dir_all(claude_dir.join("agents"));
        let agents = [
            ("orchestrator", bundle::AGENT_ORCHESTRATOR),
            ("product-ai-engineer", bundle::AGENT_PRODUCT_AI),
            ("backend-developer", bundle::AGENT_BACKEND),
            ("frontend-developer", bundle::AGENT_FRONTEND),
            ("devops-engineer", bundle::AGENT_DEVOPS),
        ];
        for (name, template) in &agents {
            let content = resolve_agent_template(template, &skills_injected, &project_name);
            let _ = fs::write(claude_dir.join("agents").join(format!("{name}.md")), content);
            files_written.push(format!(".claude/agents/{name}.md"));
        }
    }

    let _ = fs::remove_file(&inspection_path);

    text_result(serde_json::to_string_pretty(&OnboardResult {
        project_path: params.project_path,
        claude_md_overwritten: claude_md_existed,
        next_steps: vec![
            "Call scaffold_configure to add more skills or agents".to_string(),
            "Done — open the project in Claude Code, CLAUDE.md will load automatically".to_string(),
        ],
        persona,
        mode,
        files_written,
        skills_injected,
        skipped,
    }).unwrap())
}

fn resolve_agent_template(template: &str, stacks: &[String], project_name: &str) -> String {
    let mut result = template
        .replace("{{project.name}}", project_name)
        .replace("{{agents.backend-developer.model}}", "claude-opus-4-8")
        .replace("{{agents.backend-developer.effort}}", "medium")
        .replace("{{agents.frontend-developer.model}}", "claude-sonnet-4-6")
        .replace("{{agents.frontend-developer.effort}}", "medium")
        .replace("{{agents.devops-engineer.model}}", "claude-sonnet-4-6")
        .replace("{{agents.devops-engineer.effort}}", "medium");

    for category in &["backend", "frontend", "persistence", "devops", "security"] {
        let placeholder = format!("{{{{include stacks/{category}.md}}}}");
        let content: String = stacks
            .iter()
            .filter_map(|s| bundle::stack_content(s))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");
        result = result.replace(&placeholder, &content);
    }
    result
}

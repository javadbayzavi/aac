use rmcp::handler::server::common::schema_for_type;
use rmcp::handler::server::router::tool::ToolRoute;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::assets;
use crate::server::AacServer;
use crate::server::{error_result, text_result};

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
    pub claude_md_backup: Option<String>,
    pub next_steps: Vec<String>,
}

pub fn onboard_route() -> ToolRoute<AacServer> {
    let tool = Tool::new_with_raw(
        "scaffold_onboard",
        Some(std::borrow::Cow::Borrowed(
            "Onboard a project into the agentic workflow. Only call after scaffold_inspect AND user confirmation. After completing, YOU MUST call AskUserQuestion with these options: 'Configure this project now — add more skills' and 'Done — open the project in Claude Code'. Never present next steps as plain text.",
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

    let claude_md_existed = path.join("CLAUDE.md").exists();
    let persona = inspection["persona"]
        .as_str()
        .unwrap_or("developer")
        .to_string();
    let mode = inspection["mode"].as_str().unwrap_or("solo").to_string();
    let project_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let description = inspection["description"].as_str().unwrap_or("").to_string();
    let signals: Vec<String> = inspection["tech_stack_signals"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut files_written: Vec<String> = vec![];
    let mut skills_injected: Vec<String> = vec![];
    let mut skipped: Vec<String> = vec![];

    for dir in &["stacks", "protocols"] {
        if let Err(e) = fs::create_dir_all(claude_dir.join(dir)) {
            return error_result(format!("Failed to create .claude/{dir}: {e}"));
        }
    }

    // Write detected stacks
    let detected = assets::detect_stacks(&signals);
    for stack_name in &detected {
        match assets::stack_content(stack_name) {
            Some(content) => {
                let dest = claude_dir.join("stacks").join(format!("{stack_name}.md"));
                if let Err(e) = fs::write(&dest, &content) {
                    return error_result(format!("Failed to write stack {stack_name}: {e}"));
                }
                skills_injected.push(stack_name.to_string());
                files_written.push(format!(".claude/stacks/{stack_name}.md"));
            }
            None => skipped.push(stack_name.to_string()),
        }
    }

    // Persona-specific stacks
    let persona_stacks: &[&str] = match persona.as_str() {
        "product-manager" => &["product", "atlassian"],
        "designer" => &["design", "figma"],
        _ => &[],
    };
    for stack_name in persona_stacks {
        if let Some(content) = assets::stack_content(stack_name) {
            let _ = fs::write(
                claude_dir.join("stacks").join(format!("{stack_name}.md")),
                &content,
            );
            skills_injected.push(stack_name.to_string());
            files_written.push(format!(".claude/stacks/{stack_name}.md"));
        }
    }

    // PROJECT.yaml
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    let project_yaml = match assets::project_yaml_template() {
        Ok(t) => t,
        Err(e) => return error_result(format!("Failed to read PROJECT.yaml template: {e}")),
    };

    let project_yaml = project_yaml
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
        .replace(
            "{{ backend_skill }}",
            &category_skills(&skills_injected, "backend"),
        )
        .replace(
            "{{ frontend_skill }}",
            &category_skills(&skills_injected, "frontend"),
        )
        .replace(
            "{{ persistence_skill }}",
            &category_skills(&skills_injected, "persistence"),
        )
        .replace("{{ branching_model }}", "trunk")
        .replace(
            "{{ commit_pattern }}",
            "feat|fix|docs|style|refactor|test|chore",
        )
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

    // CLAUDE.md
    let tech_stack_list = skills_injected.join(", ");
    let claude_template = match assets::claude_template(&persona, &mode) {
        Ok(t) => t,
        Err(e) => return error_result(format!("Failed to read CLAUDE template: {e}")),
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

    // Never directly replace an existing CLAUDE.md — preserve it as a timestamped
    // backup before writing the generated one, so the original is never lost.
    let mut claude_md_backup: Option<String> = None;
    if claude_md_existed {
        let backup_name = format!("CLAUDE.md.backup-{now}");
        if let Err(e) = fs::rename(path.join("CLAUDE.md"), path.join(&backup_name)) {
            return error_result(format!(
                "Refusing to overwrite: failed to back up existing CLAUDE.md to {backup_name}: {e}"
            ));
        }
        files_written.push(backup_name.clone());
        claude_md_backup = Some(backup_name);
    }

    if let Err(e) = fs::write(path.join("CLAUDE.md"), &claude_md) {
        return error_result(format!("Failed to write CLAUDE.md: {e}"));
    }
    files_written.push("CLAUDE.md".into());

    // Session continuity
    let (session_name, session_src) = assets::session_file(&persona);
    if let Ok(content) = fs::read_to_string(&session_src) {
        let _ = fs::write(claude_dir.join(session_name), content);
        files_written.push(format!(".claude/{session_name}"));
    }

    // FEATURE_PLAN
    if persona == "developer" {
        if let Ok(content) = assets::feature_plan() {
            let _ = fs::write(claude_dir.join("protocols/FEATURE_PLAN.json"), content);
            files_written.push(".claude/protocols/FEATURE_PLAN.json".into());
        }
    }

    // Multi-agent agents — instantiate the agent set for this persona. To add
    // a new sub-agent (e.g. a PM sprint-planner), drop its template in
    // agentic-setup/agents/ and add its name to the persona's list here.
    if mode == "multi-agent" {
        let agents: &[&str] = match persona.as_str() {
            "product-manager" => &["product-manager"],
            "designer" => &["designer"],
            _ => &[
                "orchestrator",
                "product-ai-engineer",
                "backend-developer",
                "frontend-developer",
                "devops-engineer",
            ],
        };
        let _ = fs::create_dir_all(claude_dir.join("agents"));
        for name in agents {
            if let Ok(template) = assets::agent_template(name) {
                let content = resolve_agent_template(&template, &skills_injected, &project_name);
                let _ = fs::write(
                    claude_dir.join("agents").join(format!("{name}.md")),
                    content,
                );
                files_written.push(format!(".claude/agents/{name}.md"));
            }
        }
    }

    let _ = fs::remove_file(&inspection_path);

    text_result(
        serde_json::to_string_pretty(&OnboardResult {
            project_path: params.project_path,
            persona,
            mode,
            files_written,
            skills_injected,
            skipped,
            claude_md_overwritten: claude_md_existed,
            claude_md_backup,
            next_steps: vec![
                "Call scaffold_configure to add more skills or agents".to_string(),
                "Done — open the project in Claude Code, CLAUDE.md will load automatically"
                    .to_string(),
            ],
        })
        .unwrap(),
    )
}

/// Render the injected skills for one tech_stack category as YAML list entries,
/// preserving multiple skills per category (e.g. two backends). Returns "none"
/// when nothing was injected for the category — the placeholder default. The
/// separator keeps the template's 4-space list indentation for extra entries.
fn category_skills(skills: &[String], category: &str) -> String {
    let matches: Vec<&str> = skills
        .iter()
        .filter(|s| assets::stack_category(s) == category)
        .map(String::as_str)
        .collect();
    if matches.is_empty() {
        "none".to_string()
    } else {
        matches.join("\n    - ")
    }
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

    // Category includes (e.g. {{include stacks/backend.md}}): inject only the
    // skills belonging to THAT category — otherwise every directive would get
    // the full concatenation of all skills.
    for category in &["backend", "frontend", "persistence", "devops", "security"] {
        let placeholder = format!("{{{{include stacks/{category}.md}}}}");
        if !result.contains(&placeholder) {
            continue;
        }
        let content: String = stacks
            .iter()
            .filter(|s| assets::stack_category(s) == *category)
            .filter_map(|s| assets::stack_content(s))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");
        result = result.replace(&placeholder, &content);
    }

    // Specific-skill includes (e.g. {{include stacks/atlassian.md}} in the PM
    // agent, {{include stacks/figma.md}} in the designer agent): inject that
    // skill's content directly. Resolves any registered skill, so new
    // collaboration agents work without changing this function.
    for skill in assets::available_stacks() {
        let placeholder = format!("{{{{include stacks/{skill}.md}}}}");
        if !result.contains(&placeholder) {
            continue;
        }
        let content = assets::stack_content(skill).unwrap_or_default();
        result = result.replace(&placeholder, &content);
    }
    result
}

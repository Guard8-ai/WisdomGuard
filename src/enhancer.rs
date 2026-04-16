use crate::prompt::{self, EnhancementRequest, EnhancementResponse};
use crate::security::{
    escape_markdown, safe_description, sanitize_llm_response, MAX_DESCRIPTION_LENGTH,
    MAX_LLM_RESPONSE_SIZE,
};
use crate::vertex::VertexClient;
use anyhow::Result;
use std::fmt::Write;

/// Call Vertex AI and return parsed enhancements.
/// Returns empty enhancements on API failure (graceful degradation).
pub async fn get_enhancements(ir_json: &str, vertex: &VertexClient) -> Result<EnhancementResponse> {
    let guide_type = prompt::detect_guide_type(ir_json)?;
    let tool_name = extract_tool_name(ir_json);

    let req = EnhancementRequest {
        tool_name,
        ir_json: ir_json.to_string(),
        guide_type,
    };

    let prompt_text = prompt::build_enhancement_prompt(&req);

    match vertex.generate_content(&prompt_text).await {
        Ok(text) if text.len() > MAX_LLM_RESPONSE_SIZE => {
            eprintln!("Warning: LLM response too large, returning empty enhancements.");
            Ok(EnhancementResponse::default())
        }
        Ok(text) => prompt::parse_enhancement_response(&text),
        Err(e) => {
            eprintln!("Warning: VertexAI call failed: {e}. Using empty enhancements.");
            Ok(EnhancementResponse::default())
        }
    }
}

/// Format a standalone enhanced guide (no base guide).
#[must_use]
pub fn format_standalone(ir_json: &str, enhancements: &EnhancementResponse) -> String {
    let tool_name = extract_tool_name(ir_json);
    format_enhanced_guide(&tool_name, enhancements)
}

/// Merge enhancements into an existing base guide.
#[must_use]
pub fn merge_enhancements_into(base_guide: &str, enhancements: &EnhancementResponse) -> String {
    merge_sections(base_guide, enhancements)
}

/// Generate the enhancement prompt for --dry-run mode.
#[must_use]
pub fn dry_run_prompt(ir_json: &str) -> String {
    let guide_type = prompt::detect_guide_type(ir_json).unwrap_or(prompt::GuideType::Cli);
    let tool_name = extract_tool_name(ir_json);

    let req = EnhancementRequest {
        tool_name,
        ir_json: ir_json.to_string(),
        guide_type,
    };

    prompt::build_enhancement_prompt(&req)
}

fn extract_tool_name(ir_json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(ir_json)
        .ok()
        .and_then(|v| v.get("name")?.as_str().map(String::from))
        .unwrap_or_else(|| "Unknown Tool".to_string())
}

fn merge_sections(base: &str, enhancements: &EnhancementResponse) -> String {
    let mut result = String::with_capacity(base.len() + 4096);
    let mut inserted_workflows = false;
    let mut inserted_mistakes = false;

    for line in base.lines() {
        if !inserted_workflows
            && line.starts_with("## ")
            && !line.contains("Quick Reference")
            && !enhancements.workflows.is_empty()
        {
            result.push_str(&format_workflows(enhancements));
            inserted_workflows = true;
        }

        if !inserted_mistakes && line.starts_with("---") && !enhancements.gotchas.is_empty() {
            result.push_str(&format_gotchas(enhancements));
            result.push_str(&format_errors(enhancements));
            inserted_mistakes = true;
        }

        result.push_str(line);
        result.push('\n');
    }

    if !inserted_workflows && !enhancements.workflows.is_empty() {
        result.push_str(&format_workflows(enhancements));
    }
    if !inserted_mistakes
        && (!enhancements.gotchas.is_empty() || !enhancements.error_solutions.is_empty())
    {
        result.push_str(&format_gotchas(enhancements));
        result.push_str(&format_errors(enhancements));
    }

    result
}

fn format_enhanced_guide(tool_name: &str, enhancements: &EnhancementResponse) -> String {
    let mut out = String::with_capacity(4096);

    let _ = writeln!(
        out,
        "# {} for AI Agents — Enhanced Guide",
        escape_markdown(tool_name)
    );
    let _ = writeln!(out);

    if !enhancements.key_items.is_empty() {
        let _ = writeln!(out, "## Key Commands");
        let _ = writeln!(out);
        for item in &enhancements.key_items {
            let _ = writeln!(out, "- `{}`", escape_markdown(item));
        }
        let _ = writeln!(out);
    }

    out.push_str(&format_workflows(enhancements));
    out.push_str(&format_gotchas(enhancements));
    out.push_str(&format_errors(enhancements));

    let _ = writeln!(out, "---");
    let _ = writeln!(out, "Enhanced with VertexAI Gemini");

    out
}

fn format_workflows(enhancements: &EnhancementResponse) -> String {
    if enhancements.workflows.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let _ = writeln!(out, "## Common Workflows");
    let _ = writeln!(out);
    for wf in &enhancements.workflows {
        let _ = writeln!(
            out,
            "### {}",
            safe_description(&wf.title, MAX_DESCRIPTION_LENGTH)
        );
        let _ = writeln!(out);
        let _ = writeln!(out, "```bash");
        for step in &wf.steps {
            let _ = writeln!(out, "{}", sanitize_llm_response(step));
        }
        let _ = writeln!(out, "```");
        let _ = writeln!(out);
    }
    out
}

fn format_gotchas(enhancements: &EnhancementResponse) -> String {
    if enhancements.gotchas.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let _ = writeln!(out, "## Common Mistakes");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Wrong | Right | Why |");
    let _ = writeln!(out, "|-------|-------|-----|");
    for g in &enhancements.gotchas {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            safe_description(&g.wrong, MAX_DESCRIPTION_LENGTH),
            safe_description(&g.right, MAX_DESCRIPTION_LENGTH),
            safe_description(&g.explanation, MAX_DESCRIPTION_LENGTH),
        );
    }
    let _ = writeln!(out);
    out
}

fn format_errors(enhancements: &EnhancementResponse) -> String {
    if enhancements.error_solutions.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let _ = writeln!(out, "## Error Messages");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Error | Solution |");
    let _ = writeln!(out, "|-------|----------|");
    for es in &enhancements.error_solutions {
        let _ = writeln!(
            out,
            "| {} | {} |",
            safe_description(&es.error, MAX_DESCRIPTION_LENGTH),
            safe_description(&es.solution, MAX_DESCRIPTION_LENGTH),
        );
    }
    let _ = writeln!(out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{ErrorSolution, Gotcha, Workflow};

    #[test]
    fn formats_enhanced_guide() {
        let enhancements = EnhancementResponse {
            workflows: vec![Workflow {
                title: "Setup".to_string(),
                steps: vec!["install".to_string(), "configure".to_string()],
            }],
            gotchas: vec![Gotcha {
                wrong: "bad way".to_string(),
                right: "good way".to_string(),
                explanation: "because".to_string(),
            }],
            key_items: vec!["run".to_string(), "build".to_string()],
            error_solutions: vec![ErrorSolution {
                error: "not found".to_string(),
                solution: "install it".to_string(),
            }],
        };

        let guide = format_enhanced_guide("mytool", &enhancements);
        assert!(guide.contains("# mytool for AI Agents"));
        assert!(guide.contains("## Key Commands"));
        assert!(guide.contains("## Common Workflows"));
        assert!(guide.contains("## Common Mistakes"));
        assert!(guide.contains("## Error Messages"));
    }

    #[test]
    fn dry_run_produces_prompt() {
        let ir = r#"{"name":"test","framework":"Clap","commands":[],"global_flags":[],"groups":[],"description":"test"}"#;
        let prompt = dry_run_prompt(ir);
        assert!(prompt.contains("test"));
        assert!(prompt.contains("workflows"));
    }

    #[test]
    fn merges_sections_into_base() {
        let base = "# Tool for AI Agents\n\n## Quick Reference\n\n```bash\ntool run\n```\n\n## Command Reference\n\nDetails here.\n\n---\n**Framework**: clap\n";
        let enhancements = EnhancementResponse {
            workflows: vec![Workflow {
                title: "Setup".to_string(),
                steps: vec!["step1".to_string()],
            }],
            gotchas: vec![Gotcha {
                wrong: "bad".to_string(),
                right: "good".to_string(),
                explanation: "because".to_string(),
            }],
            key_items: Vec::new(),
            error_solutions: Vec::new(),
        };

        let merged = merge_sections(base, &enhancements);
        assert!(merged.contains("## Quick Reference"));
        assert!(merged.contains("## Command Reference"));
        assert!(merged.contains("**Framework**: clap"));
        assert!(merged.contains("## Common Workflows"));
        assert!(merged.contains("## Common Mistakes"));
    }

    #[test]
    fn standalone_format_works() {
        let ir = r#"{"name":"tool","framework":"Clap","commands":[],"global_flags":[],"groups":[],"description":"test"}"#;
        let enhancements = EnhancementResponse::default();
        let guide = format_standalone(ir, &enhancements);
        assert!(guide.contains("# tool for AI Agents"));
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};

const ENHANCEMENT_PROMPT: &str = r#"You are an expert at writing concise, actionable guides for AI agents that interact with software tools.

Given the following tool specification in JSON format for "{TOOL_NAME}", generate enhancements to make the guide more useful for AI agents.

Return a JSON object with these fields:
- "workflows": array of {"title": "string", "steps": ["step1", "step2", ...]} - common multi-step task recipes
- "gotchas": array of {"wrong": "string", "right": "string", "explanation": "string"} - common mistakes
- "key_items": array of strings - the most important commands/endpoints/functions (top 20%)
- "error_solutions": array of {"error": "string", "solution": "string"} - common errors and fixes

Keep each array to 3-7 items. Be concise and practical.

Tool specification:
{IR_JSON}

Respond ONLY with valid JSON, no markdown fences or explanation."#;

/// What type of guide we're enhancing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuideType {
    Cli,
    Api,
    Doc,
}

impl std::fmt::Display for GuideType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cli => write!(f, "CLI"),
            Self::Api => write!(f, "API"),
            Self::Doc => write!(f, "Documentation"),
        }
    }
}

/// Request to enhance a guide.
pub struct EnhancementRequest {
    pub tool_name: String,
    pub ir_json: String,
    pub guide_type: GuideType,
}

/// A multi-step workflow recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub title: String,
    pub steps: Vec<String>,
}

/// A gotcha: wrong vs right pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gotcha {
    pub wrong: String,
    pub right: String,
    pub explanation: String,
}

/// An error with its solution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSolution {
    pub error: String,
    pub solution: String,
}

/// Response from the LLM enhancement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnhancementResponse {
    #[serde(default)]
    pub workflows: Vec<Workflow>,
    #[serde(default)]
    pub gotchas: Vec<Gotcha>,
    #[serde(default)]
    pub key_items: Vec<String>,
    #[serde(default)]
    pub error_solutions: Vec<ErrorSolution>,
}

/// Build the prompt for the LLM.
#[must_use]
pub fn build_enhancement_prompt(req: &EnhancementRequest) -> String {
    ENHANCEMENT_PROMPT
        .replace("{TOOL_NAME}", &req.tool_name)
        .replace("{IR_JSON}", &req.ir_json)
        .replace("tool specification", &format!("{} tool specification", req.guide_type))
}

/// Parse the LLM response into an `EnhancementResponse`.
/// Falls back to empty response on parse failure.
pub fn parse_enhancement_response(response: &str) -> Result<EnhancementResponse> {
    // Strip markdown fences if the LLM included them despite instructions
    let cleaned = response
        .trim()
        .strip_prefix("```json")
        .or_else(|| response.trim().strip_prefix("```"))
        .unwrap_or(response.trim());
    let cleaned = cleaned
        .strip_suffix("```")
        .unwrap_or(cleaned)
        .trim();

    serde_json::from_str(cleaned).or_else(|_| {
        eprintln!("Warning: Could not parse LLM response as JSON, using empty enhancements");
        Ok(EnhancementResponse::default())
    })
}

/// Detect what type of IR this is by checking for known fields.
pub fn detect_guide_type(ir_json: &str) -> Result<GuideType> {
    let value: serde_json::Value = serde_json::from_str(ir_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in IR file: {e}"))?;

    if value.get("framework").is_some() && value.get("commands").is_some() {
        return Ok(GuideType::Cli);
    }
    if value.get("endpoints").is_some() && value.get("spec_format").is_some() {
        return Ok(GuideType::Api);
    }
    if value.get("modules").is_some() && value.get("source_format").is_some() {
        return Ok(GuideType::Doc);
    }

    anyhow::bail!("Cannot detect IR type. Expected CliGuard, ApiGuard, or DocGuard JSON output")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_prompt() {
        let req = EnhancementRequest {
            tool_name: "mytool".to_string(),
            ir_json: r#"{"name":"mytool"}"#.to_string(),
            guide_type: GuideType::Cli,
        };
        let prompt = build_enhancement_prompt(&req);
        assert!(prompt.contains("mytool"));
        assert!(prompt.contains(r#"{"name":"mytool"}"#));
    }

    #[test]
    fn parses_valid_response() {
        let json = r#"{"workflows": [{"title": "Setup", "steps": ["install", "configure"]}], "gotchas": [], "key_items": ["run"], "error_solutions": []}"#;
        let resp = parse_enhancement_response(json).unwrap();
        assert_eq!(resp.workflows.len(), 1);
        assert_eq!(resp.key_items, vec!["run"]);
    }

    #[test]
    fn parses_fenced_response() {
        let json = "```json\n{\"workflows\": [], \"gotchas\": [], \"key_items\": [], \"error_solutions\": []}\n```";
        let resp = parse_enhancement_response(json).unwrap();
        assert!(resp.workflows.is_empty());
    }

    #[test]
    fn fallback_on_invalid_json() {
        let resp = parse_enhancement_response("not json at all").unwrap();
        assert!(resp.workflows.is_empty());
    }

    #[test]
    fn detects_cli_ir() {
        let json = r#"{"name":"tool","framework":"Clap","commands":[],"global_flags":[],"groups":[],"description":""}"#;
        let t = detect_guide_type(json).unwrap();
        assert!(matches!(t, GuideType::Cli));
    }

    #[test]
    fn detects_api_ir() {
        let json = r#"{"name":"api","endpoints":[],"spec_format":"OpenApi3","schemas":[],"auth_methods":[],"description":""}"#;
        let t = detect_guide_type(json).unwrap();
        assert!(matches!(t, GuideType::Api));
    }

    #[test]
    fn detects_doc_ir() {
        let json = r#"{"name":"lib","modules":[],"source_format":"RustDoc","sections":[],"description":""}"#;
        let t = detect_guide_type(json).unwrap();
        assert!(matches!(t, GuideType::Doc));
    }
}

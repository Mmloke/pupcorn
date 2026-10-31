use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::llm::{ChatMessage, LlmClient};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub target: String,
    pub options: Value,
    /// Whether this call is intrusive (requires approval).
    pub intrusive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub goal: String,
    pub steps: Vec<ToolCall>,
    pub rationale: String,
}

pub struct Planner<'a, L: LlmClient> {
    pub llm: &'a L,
}

impl<'a, L: LlmClient> Planner<'a, L> {
    pub fn new(llm: &'a L) -> Self {
        Self { llm }
    }

    /// Ask the LLM to produce a JSON plan given a goal and available tools.
    pub async fn plan(&self, goal: &str, tool_list: &str) -> Result<Plan> {
        let system = format!(
            "You are Pupcorn, an AI assistant for bug-bounty recon. \
             Given a goal and the list of available tools, return a JSON object with fields: \
             goal (string), rationale (string), steps (array of {{tool, target, options, intrusive}}). \
             Only use tools from the list. Mark intrusive=true only for active/exploitation actions. \
             Available tools:\n{}",
            tool_list
        );
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system,
            },
            ChatMessage {
                role: "user".to_string(),
                content: goal.to_string(),
            },
        ];
        let raw = self.llm.chat(messages).await?;
        // Try to parse JSON; tolerate fenced code blocks.
        let cleaned = raw
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        let plan: Plan = serde_json::from_str(cleaned)
            .map_err(|e| crate::error::AiError::Parse(format!("{}: {}", e, cleaned)))?;
        Ok(plan)
    }
}
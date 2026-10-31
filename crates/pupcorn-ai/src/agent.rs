use serde::{Deserialize, Serialize};

use pupcorn_core::ToolEngine;
use crate::error::Result;
use crate::llm::{ChatMessage, LlmClient};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentTurn {
    pub role: String,
    pub content: String,
}

pub struct Agent<'a, L: LlmClient> {
    pub llm: &'a L,
    pub engine: &'a ToolEngine,
    pub max_iters: usize,
}

impl<'a, L: LlmClient> Agent<'a, L> {
    pub fn new(llm: &'a L, engine: &'a ToolEngine) -> Self {
        Self {
            llm,
            engine,
            max_iters: 8,
        }
    }

    pub async fn run(&self, goal: &str) -> Result<String> {
        let tool_list = self
            .engine
            .list()
            .iter()
            .map(|(n, d, i)| format!("- {} (intrusive={}): {}", n, i, d))
            .collect::<Vec<_>>()
            .join("\n");

        let system = format!(
            "You are Pupcorn, an AI bug-bounty assistant. You have tools:\n{}\n\
             Respond ONLY with JSON. Either:\n\
             {{\"type\":\"final\",\"content\":\"<answer>\"}}\n\
             or\n\
             {{\"type\":\"tool\",\"tool\":\"<name>\",\"target\":\"<target>\",\"options\":{{...}}}}",
            tool_list
        );

        let mut history: Vec<ChatMessage> = vec![
            ChatMessage { role: "system".to_string(), content: system },
            ChatMessage { role: "user".to_string(), content: goal.to_string() },
        ];

        for _ in 0..self.max_iters {
            let raw = self.llm.chat(history.clone()).await?;
            let cleaned = raw
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            // Flexible parse: accept either {"type":"final",...} or {"type":"tool",...}
            // or objects with "tool" / "final"/"answer"/"content"/"message" keys.
            let val: serde_json::Value = match serde_json::from_str(cleaned) {
                Ok(v) => v,
                Err(_) => {
                    // Not JSON -> treat as final answer.
                    return Ok(raw);
                }
            };

            // Tool call?
            if let Some(tool) = val.get("tool").and_then(|v| v.as_str()) {
                let target = val.get("target").and_then(|v| v.as_str()).unwrap_or(&goal).to_string();
                let options = val.get("options").cloned().unwrap_or(serde_json::json!({}));
                // execute
                let res = self.engine.run_tool(tool, &target, options, None).await;
                let result_text = match res {
                    Ok(out) => format!("tool={} target={} summary={} data={}", out.tool, out.target, out.summary, out.data),
                    Err(e) => format!("tool={} error={}", tool, e),
                };
                history.push(ChatMessage { role: "assistant".to_string(), content: raw.clone() });
                history.push(ChatMessage { role: "user".to_string(), content: format!("Tool result: {}", result_text) });
                continue;
            }

            // Final answer?
            let content = val.get("content")
                .or_else(|| val.get("answer"))
                .or_else(|| val.get("message"))
                .or_else(|| val.get("text"))
                .or_else(|| val.get("final"))
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| cleaned.to_string());
            return Ok(content);
        }
        Ok("max iterations reached".to_string())
    }
}
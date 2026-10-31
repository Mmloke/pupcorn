use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

/// Unified interface every recon/scan module implements.
/// AI calls tools generically: run_tool(name, target, options) -> Result.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique tool name used for dispatch.
    fn name(&self) -> &'static str;

    /// Human-readable description shown to the AI.
    fn description(&self) -> &'static str;

    /// Whether this tool performs active/intrusive actions on the target.
    /// Intrusive tools require explicit researcher approval before execution.
    fn is_intrusive(&self) -> bool {
        false
    }

    /// Execute the tool against a target with JSON options.
    async fn run(&self, target: &str, options: Value) -> Result<ToolOutput>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub tool: String,
    pub target: String,
    pub summary: String,
    pub data: Value,
    pub findings_count: usize,
}

/// Options passed to a tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolOptions {
    #[serde(default)]
    pub extra: Value,
}
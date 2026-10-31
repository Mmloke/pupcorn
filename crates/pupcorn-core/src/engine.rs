use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::error::{PupcornError, Result};
use crate::tool::{Tool, ToolOutput};

/// Researcher approval token for intrusive actions.
#[derive(Debug, Clone)]
pub struct Approval {
    pub granted: bool,
    pub note: Option<String>,
}

/// Registry + dispatcher for all tools. AI calls run_tool(name, target, options).
pub struct ToolEngine {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolEngine {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
    }

    pub fn list(&self) -> Vec<(&'static str, &'static str, bool)> {
        self.tools
            .values()
            .map(|t| (t.name(), t.description(), t.is_intrusive()))
            .collect()
    }

    pub fn is_intrusive(&self, name: &str) -> bool {
        self.tools.get(name).map(|t| t.is_intrusive()).unwrap_or(false)
    }

    /// Run a tool. Intrusive tools require an explicit Approval with granted=true.
    pub async fn run_tool(
        &self,
        name: &str,
        target: &str,
        options: Value,
        approval: Option<Approval>,
    ) -> Result<ToolOutput> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| PupcornError::ToolNotFound(name.to_string()))?;

        if tool.is_intrusive() {
            match approval {
                Some(a) if a.granted => {}
                _ => {
                    return Err(PupcornError::ApprovalRequired(format!(
                        "tool '{}' is intrusive and requires explicit researcher approval",
                        name
                    )))
                }
            }
        }

        // Audit trail entry is recorded by caller (db); engine stays pure.
        tool.run(target, options).await
    }
}

impl Default for ToolEngine {
    fn default() -> Self {
        Self::new()
    }
}
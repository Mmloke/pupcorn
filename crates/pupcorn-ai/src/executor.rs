use pupcorn_core::{Approval, ToolEngine, ToolOutput};

use crate::planner::Plan;

pub struct Executor<'a> {
    pub engine: &'a ToolEngine,
}

impl<'a> Executor<'a> {
    pub fn new(engine: &'a ToolEngine) -> Self {
        Self { engine }
    }

    /// Execute a plan step by step. Intrusive steps are skipped unless approval granted.
    pub async fn execute(&self, plan: &Plan, approval: Option<Approval>) -> Vec<StepResult> {
        let mut results = Vec::new();
        for step in &plan.steps {
            if step.intrusive {
                match &approval {
                    Some(a) if a.granted => {}
                    _ => {
                        results.push(StepResult {
                            tool: step.tool.clone(),
                            target: step.target.clone(),
                            ok: false,
                            output: None,
                            error: Some("intrusive step requires approval".to_string()),
                        });
                        continue;
                    }
                }
            }
            let res = self
                .engine
                .run_tool(&step.tool, &step.target, step.options.clone(), approval.clone())
                .await;
            match res {
                Ok(out) => results.push(StepResult {
                    tool: step.tool.clone(),
                    target: step.target.clone(),
                    ok: true,
                    output: Some(out),
                    error: None,
                }),
                Err(e) => results.push(StepResult {
                    tool: step.tool.clone(),
                    target: step.target.clone(),
                    ok: false,
                    output: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        results
    }
}

#[derive(Debug)]
pub struct StepResult {
    pub tool: String,
    pub target: String,
    pub ok: bool,
    pub output: Option<ToolOutput>,
    pub error: Option<String>,
}
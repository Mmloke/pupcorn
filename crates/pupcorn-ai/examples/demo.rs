use pupcorn_ai::{Executor, MockLlm, Planner};
use pupcorn_core::{default_engine, Approval};
use serde_json::json;

#[tokio::main]
async fn main() {
    let engine = default_engine();
    let tool_list = engine
        .list()
        .iter()
        .map(|(n, d, i)| format!("- {} (intrusive={}): {}", n, i, d))
        .collect::<Vec<_>>()
        .join("\n");

    // Mock LLM returns a plan JSON.
    let plan_json = r#"{
        "goal": "probe example.com",
        "rationale": "http probe first",
        "steps": [
            {"tool": "http_probe", "target": "example.com", "options": {}, "intrusive": false}
        ]
    }"#;
    let llm = MockLlm {
        response: plan_json.to_string(),
    };
    let planner = Planner::new(&llm);
    let plan = planner.plan("probe example.com", &tool_list).await.unwrap();
    println!("Plan: {} steps", plan.steps.len());

    let executor = Executor::new(&engine);
    let results = executor.execute(&plan, None).await;
    for r in &results {
        println!("{} on {} -> ok={}", r.tool, r.target, r.ok);
        if let Some(o) = &r.output {
            println!("  summary: {}", o.summary);
        }
        if let Some(e) = &r.error {
            println!("  error: {}", e);
        }
    }

    // Demonstrate approval gate: intrusive tool without approval is blocked.
    let intrusive_plan_json = r#"{
        "goal": "intrusive test",
        "rationale": "demo",
        "steps": [
            {"tool": "http_probe", "target": "example.com", "options": {}, "intrusive": true}
        ]
    }"#;
    let llm2 = MockLlm {
        response: intrusive_plan_json.to_string(),
    };
    let plan2 = Planner::new(&llm2).plan("x", &tool_list).await.unwrap();
    let results2 = Executor::new(&engine).execute(&plan2, None).await;
    println!(
        "Intrusive without approval blocked: {}",
        results2[0].error.as_deref().unwrap_or("no")
    );

    // With approval granted -> runs.
    let results3 = Executor::new(&engine)
        .execute(&plan2, Some(Approval { granted: true, note: None }))
        .await;
    println!("Intrusive with approval ok={}", results3[0].ok);

    let _ = json!({});
}
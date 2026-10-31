use pupcorn_core::{default_engine, Database};
use serde_json::json;

#[tokio::main]
async fn main() {
    let engine = default_engine();
    println!("Registered tools:");
    for (name, desc, intrusive) in engine.list() {
        println!("  - {} (intrusive={}) : {}", name, intrusive, desc);
    }

    // Demo: run http_probe on example.com
    let out = engine
        .run_tool("http_probe", "example.com", json!({}), None)
        .await
        .unwrap();
    println!("\nhttp_probe result: {}", serde_json::to_string_pretty(&out).unwrap());

    // Demo: DB
    let db = Database::new("sqlite::memory:").await.unwrap();
    println!("\nDB migrated OK");
    let _ = db;
}
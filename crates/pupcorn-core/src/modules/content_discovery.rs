use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct ContentDiscovery;

#[async_trait]
impl Tool for ContentDiscovery {
    fn name(&self) -> &'static str { "content_discovery" }
    fn description(&self) -> &'static str { "Discover hidden paths/endpoints via wordlist against base URL." }

    async fn run(&self, target: &str, options: Value) -> Result<ToolOutput> {
        let base = if target.starts_with("http") {
            target.trim_end_matches('/').to_string()
        } else {
            format!("https://{}", target.trim_end_matches('/'))
        };
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(crate::error::PupcornError::Http)?;
        let paths: Vec<&str> = options
            .get("paths")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
            .unwrap_or_else(|| {
                vec![
                    "admin", "login", "api", "api/v1", "swagger", "docs", "robots.txt",
                    "sitemap.xml", ".env", ".git", "config", "backup", "test", "debug",
                    "health", "status", "metrics", "graphql", "wp-admin", "wp-login.php",
                    "phpmyadmin", "server-status", "actuator", "actuator/health",
                ]
            });

        let mut found = Vec::new();
        for p in &paths {
            let url = format!("{}/{}", base, p);
            if let Ok(resp) = client.get(&url).send().await {
                let status = resp.status().as_u16();
                if status != 404 && status != 400 {
                    found.push(json!({"path": p, "status": status, "url": url}));
                }
            }
        }

        Ok(ToolOutput {
            tool: self.name().to_string(),
            target: target.to_string(),
            summary: format!("Found {} interesting paths on {}", found.len(), base),
            data: json!({ "paths": found }),
            findings_count: found.len(),
        })
    }
}
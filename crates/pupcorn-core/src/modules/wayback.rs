use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct WaybackUrls;

#[async_trait]
impl Tool for WaybackUrls {
    fn name(&self) -> &'static str {
        "wayback_urls"
    }
    fn description(&self) -> &'static str {
        "Collect historical URLs for a domain from the Wayback Machine CDX API."
    }

    async fn run(&self, target: &str, options: Value) -> Result<ToolOutput> {
        let domain = target.trim().trim_start_matches("http://").trim_start_matches("https://").split('/').next().unwrap_or(target).to_string();
        let limit = options.get("limit").and_then(|v| v.as_u64()).unwrap_or(200);
        let client = reqwest::Client::builder()
            .user_agent("Pupcorn/0.1")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(crate::error::PupcornError::Http)?;

        let url = format!(
            "http://web.archive.org/cdx/search/cdx?url={}/*&output=json&fl=original&limit={}&collapse=urlkey",
            domain, limit
        );
        let mut urls: Vec<String> = Vec::new();
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(text) = resp.text().await {
                if let Ok(arr) = serde_json::from_str::<Vec<Vec<String>>>(&text) {
                    for row in arr.iter().skip(1) {
                        if let Some(u) = row.first() { urls.push(u.clone()); }
                    }
                }
            }
        }
        urls.sort();
        urls.dedup();

        Ok(ToolOutput {
            tool: self.name().to_string(),
            target: target.to_string(),
            summary: format!("Found {} historical URLs for {}", urls.len(), domain),
            data: json!({ "domain": domain, "urls": urls }),
            findings_count: urls.len(),
        })
    }
}
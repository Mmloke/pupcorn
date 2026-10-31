use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct HttpProbe;

#[async_trait]
impl Tool for HttpProbe {
    fn name(&self) -> &'static str {
        "http_probe"
    }
    fn description(&self) -> &'static str {
        "Probe HTTP/HTTPS endpoints: status code, headers, title."
    }

    async fn run(&self, target: &str, _options: Value) -> Result<ToolOutput> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(crate::error::PupcornError::Http)?;
        let url = if target.starts_with("http") {
            target.to_string()
        } else {
            format!("https://{}", target)
        };
        let resp = client.get(&url).send().await;
        match resp {
            Ok(r) => {
                let status = r.status().as_u16();
                let headers: serde_json::Map<String, Value> = r
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), json!(v.to_str().unwrap_or(""))))
                    .collect();
                let body = r.text().await.unwrap_or_default();
                let title = extract_title(&body);
                Ok(ToolOutput {
                    tool: self.name().to_string(),
                    target: target.to_string(),
                    summary: format!("{} -> HTTP {}", url, status),
                    data: json!({
                        "url": url,
                        "status": status,
                        "title": title,
                        "headers": headers,
                    }),
                    findings_count: 1,
                })
            }
            Err(e) => Ok(ToolOutput {
                tool: self.name().to_string(),
                target: target.to_string(),
                summary: format!("request failed: {}", e),
                data: json!({ "error": e.to_string() }),
                findings_count: 0,
            }),
        }
    }
}

fn extract_title(html: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?i)<title[^>]*>(.*?)</title>").ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
}
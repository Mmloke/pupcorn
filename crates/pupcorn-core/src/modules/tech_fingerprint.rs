use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct TechFingerprint;

#[async_trait]
impl Tool for TechFingerprint {
    fn name(&self) -> &'static str { "tech_fingerprint" }
    fn description(&self) -> &'static str { "Fingerprint technologies from HTTP response headers/body." }

    async fn run(&self, target: &str, _options: Value) -> Result<ToolOutput> {
        let url = if target.starts_with("http") { target.to_string() } else { format!("https://{}", target) };
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(crate::error::PupcornError::Http)?;
        let mut techs: Vec<String> = Vec::new();
        if let Ok(resp) = client.get(&url).send().await {
            for (k, v) in resp.headers().iter() {
                let kl = k.as_str().to_lowercase();
                let vl = v.to_str().unwrap_or("").to_lowercase();
                if kl == "server" { techs.push(format!("server: {}", vl)); }
                if kl == "x-powered-by" { techs.push(format!("powered-by: {}", vl)); }
                if kl.contains("framework") { techs.push(format!("{}: {}", kl, vl)); }
            }
            let body = resp.text().await.unwrap_or_default().to_lowercase();
            let hints = [
                ("wp-content", "WordPress"), ("react", "React"), ("vue", "Vue.js"),
                ("angular", "Angular"), ("jquery", "jQuery"), ("bootstrap", "Bootstrap"),
                ("laravel", "Laravel"), ("django", "Django"), ("rails", "Ruby on Rails"),
                ("next", "Next.js"), ("nuxt", "Nuxt.js"),
            ];
            for (needle, name) in hints {
                if body.contains(needle) && !techs.iter().any(|t| t.contains(name)) {
                    techs.push(name.to_string());
                }
            }
        }
        Ok(ToolOutput {
            tool: self.name().to_string(),
            target: target.to_string(),
            summary: format!("Detected {} technologies", techs.len()),
            data: json!({ "technologies": techs }),
            findings_count: techs.len(),
        })
    }
}
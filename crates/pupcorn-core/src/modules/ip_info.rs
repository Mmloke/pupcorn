use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct IpInfo;

#[async_trait]
impl Tool for IpInfo {
    fn name(&self) -> &'static str {
        "ip_info"
    }
    fn description(&self) -> &'static str {
        "IP geolocation/ASN/org info via ipinfo.io (free, no key)."
    }

    async fn run(&self, target: &str, _options: Value) -> Result<ToolOutput> {
        let host = target.trim().trim_start_matches("http://").trim_start_matches("https://").split('/').next().unwrap_or(target).to_string();
        // Resolve to IP if needed.
        let ip = if host.chars().all(|c| c.is_ascii_digit() || c == '.') {
            host.clone()
        } else {
            match tokio::net::lookup_host(format!("{}:80", host)).await {
                Ok(mut addrs) => addrs.next().map(|a| a.ip().to_string()).unwrap_or(host.clone()),
                Err(_) => host.clone(),
            }
        };

        let client = reqwest::Client::builder()
            .user_agent("Pupcorn/0.1")
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(crate::error::PupcornError::Http)?;

        let url = format!("https://ipinfo.io/{}/json", ip);
        let mut data = json!({ "ip": ip });
        let mut summary = format!("IP info for {}", ip);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(v) = resp.json::<Value>().await {
                let city = v.get("city").and_then(|x| x.as_str()).unwrap_or("");
                let region = v.get("region").and_then(|x| x.as_str()).unwrap_or("");
                let country = v.get("country").and_then(|x| x.as_str()).unwrap_or("");
                let org = v.get("org").and_then(|x| x.as_str()).unwrap_or("");
                summary = format!("{} | {} {} {} | {}", ip, city, region, country, org);
                data = json!({ "ip": ip, "host": host, "info": v });
            }
        }

        Ok(ToolOutput {
            tool: self.name().to_string(),
            target: target.to_string(),
            summary,
            data,
            findings_count: 1,
        })
    }
}
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct WhoisRdap;

#[async_trait]
impl Tool for WhoisRdap {
    fn name(&self) -> &'static str {
        "whois_rdap"
    }
    fn description(&self) -> &'static str {
        "Domain registration info via RDAP (registrar, dates, nameservers, status)."
    }

    async fn run(&self, target: &str, _options: Value) -> Result<ToolOutput> {
        let domain = target.trim().trim_start_matches("http://").trim_start_matches("https://").split('/').next().unwrap_or(target).to_string();
        let client = reqwest::Client::builder()
            .user_agent("Pupcorn/0.1")
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(crate::error::PupcornError::Http)?;

        let url = format!("https://rdap.org/domain/{}", domain);
        let mut data = json!({});
        let mut summary = format!("RDAP lookup for {}", domain);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(v) = resp.json::<Value>().await {
                let registrar = v.get("registrar").and_then(|r| r.get("name")).and_then(|x| x.as_str()).unwrap_or("").to_string();
                let created = v.get("events").and_then(|e| e.as_array()).and_then(|arr| {
                    arr.iter().find(|e| e.get("eventAction").and_then(|x| x.as_str()) == Some("registration")).and_then(|e| e.get("eventDate").and_then(|x| x.as_str())).map(|s| s.to_string())
                }).unwrap_or_default();
                let expires = v.get("events").and_then(|e| e.as_array()).and_then(|arr| {
                    arr.iter().find(|e| e.get("eventAction").and_then(|x| x.as_str()) == Some("expiration")).and_then(|e| e.get("eventDate").and_then(|x| x.as_str())).map(|s| s.to_string())
                }).unwrap_or_default();
                let mut ns: Vec<String> = Vec::new();
                if let Some(n) = v.get("nameservers").and_then(|x| x.as_array()) {
                    for s in n { if let Some(ldh) = s.get("ldhName").and_then(|x| x.as_str()) { ns.push(ldh.to_string()); } }
                }
                let mut status: Vec<String> = Vec::new();
                if let Some(s) = v.get("status").and_then(|x| x.as_array()) {
                    for x in s { if let Some(t) = x.as_str() { status.push(t.to_string()); } }
                }
                summary = format!("{} | registrar: {} | created: {} | expires: {}", domain, registrar, created, expires);
                data = json!({
                    "domain": domain,
                    "registrar": registrar,
                    "created": created,
                    "expires": expires,
                    "nameservers": ns,
                    "status": status,
                    "raw": v,
                });
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
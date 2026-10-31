use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::BTreeSet;

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct SubdomainEnum;

#[async_trait]
impl Tool for SubdomainEnum {
    fn name(&self) -> &'static str {
        "subdomain_enum"
    }
    fn description(&self) -> &'static str {
        "Enumerate subdomains via certificate transparency (crt.sh) + DNS wordlist brute-force."
    }

    async fn run(&self, target: &str, options: Value) -> Result<ToolOutput> {
        let domain = target.trim().trim_start_matches("http://").trim_start_matches("https://").split('/').next().unwrap_or(target).to_string();
        let mut found: BTreeSet<String> = BTreeSet::new();

        // 1. Passive: crt.sh certificate transparency.
        let client = reqwest::Client::builder()
            .user_agent("Pupcorn/0.1")
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(crate::error::PupcornError::Http)?;
        let url = format!("https://crt.sh/?q=%25.{}&output=json", domain);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(text) = resp.text().await {
                if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                    for entry in arr {
                        if let Some(nv) = entry.get("name_value").and_then(|v| v.as_str()) {
                            for line in nv.lines() {
                                let name = line.trim().trim_start_matches("*.").to_lowercase();
                                if !name.is_empty() && name.ends_with(&domain) {
                                    found.insert(name);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. DNS brute-force with expanded wordlist.
        let wordlist: Vec<&str> = options
            .get("wordlist")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
            .unwrap_or_else(|| {
                vec![
                    "www", "api", "app", "dev", "staging", "test", "admin", "mail", "blog",
                    "cdn", "static", "assets", "auth", "login", "portal", "m", "mobile", "web",
                    "beta", "alpha", "demo", "docs", "help", "support", "status", "dashboard",
                    "panel", "manage", "manager", "console", "secure", "vpn", "remote", "git",
                    "gitlab", "github", "jenkins", "ci", "cd", "build", "deploy", "api2",
                    "v1", "v2", "rest", "graphql", "ws", "socket", "chat", "forum", "shop",
                    "store", "cart", "pay", "payment", "billing", "account", "accounts",
                    "user", "users", "profile", "my", "me", "old", "new", "legacy", "backup",
                    "bak", "tmp", "temp", "debug", "trace", "log", "logs", "monitor", "metrics",
                    "health", "ready", "live", "internal", "intranet", "corp", "corporate",
                    "prod", "production", "preprod", "qa", "uat", "sandbox", "lab", "labs",
                ]
            });

        for w in &wordlist {
            let host = format!("{}.{}", w, domain);
            if tokio::net::lookup_host(format!("{}:80", host)).await.is_ok() {
                found.insert(host);
            }
        }

        let list: Vec<String> = found.into_iter().collect();
        Ok(ToolOutput {
            tool: self.name().to_string(),
            target: target.to_string(),
            summary: format!("Found {} subdomains for {}", list.len(), domain),
            data: json!({ "subdomains": list, "checked": wordlist.len() }),
            findings_count: list.len(),
        })
    }
}
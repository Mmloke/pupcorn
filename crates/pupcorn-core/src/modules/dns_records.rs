use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct DnsRecords;

#[async_trait]
impl Tool for DnsRecords {
    fn name(&self) -> &'static str {
        "dns_records"
    }
    fn description(&self) -> &'static str {
        "Query DNS records (A, AAAA, MX, TXT, NS, CNAME) via DNS-over-HTTPS (Cloudflare)."
    }

    async fn run(&self, target: &str, _options: Value) -> Result<ToolOutput> {
        let domain = target.trim().trim_start_matches("http://").trim_start_matches("https://").split('/').next().unwrap_or(target).to_string();
        let client = reqwest::Client::builder()
            .user_agent("Pupcorn/0.1")
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(crate::error::PupcornError::Http)?;

        let mut records: serde_json::Map<String, Value> = serde_json::Map::new();
        let mut total = 0usize;
        for rtype in ["A", "AAAA", "MX", "TXT", "NS", "CNAME"] {
            let url = format!("https://cloudflare-dns.com/dns-query?name={}&type={}", domain, rtype);
            if let Ok(resp) = client.get(&url).header("Accept", "application/dns-json").send().await {
                if let Ok(v) = resp.json::<Value>().await {
                    let mut vals: Vec<String> = Vec::new();
                    if let Some(ans) = v.get("Answer").and_then(|a| a.as_array()) {
                        for a in ans {
                            if let Some(d) = a.get("data").and_then(|x| x.as_str()) {
                                vals.push(d.to_string());
                            }
                        }
                    }
                    total += vals.len();
                    records.insert(rtype.to_string(), json!(vals));
                }
            }
        }

        Ok(ToolOutput {
            tool: self.name().to_string(),
            target: target.to_string(),
            summary: format!("Resolved {} DNS records for {}", total, domain),
            data: json!({ "domain": domain, "records": records }),
            findings_count: total,
        })
    }
}
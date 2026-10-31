use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct DorkGen;

#[async_trait]
impl Tool for DorkGen {
    fn name(&self) -> &'static str {
        "dork_gen"
    }
    fn description(&self) -> &'static str {
        "Generate Google dork queries for a target (files, admin panels, exposed data)."
    }

    async fn run(&self, target: &str, _options: Value) -> Result<ToolOutput> {
        let domain = target.trim().trim_start_matches("http://").trim_start_matches("https://").split('/').next().unwrap_or(target).to_string();
        let dorks = vec![
            format!("site:{} filetype:pdf", domain),
            format!("site:{} filetype:xls OR filetype:xlsx OR filetype:csv", domain),
            format!("site:{} inurl:admin OR inurl:login OR inurl:dashboard", domain),
            format!("site:{} intitle:\"index of\"", domain),
            format!("site:{} ext:sql OR ext:bak OR ext:log OR ext:conf", domain),
            format!("site:{} inurl:api OR inurl:swagger OR inurl:graphql", domain),
            format!("site:{} \"password\" OR \"secret\" OR \"token\"", domain),
            format!("site:github.com \"{}\"", domain),
            format!("site:pastebin.com \"{}\"", domain),
            format!("\"{}\" filetype:env", domain),
        ];
        let count = dorks.len();
        Ok(ToolOutput {
            tool: self.name().to_string(),
            target: target.to_string(),
            summary: format!("Generated {} dork queries for {}", count, domain),
            data: json!({ "domain": domain, "dorks": dorks }),
            findings_count: count,
        })
    }
}
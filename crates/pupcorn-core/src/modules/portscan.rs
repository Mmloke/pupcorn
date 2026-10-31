use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::tool::{Tool, ToolOutput};

pub struct PortScan;

#[async_trait]
impl Tool for PortScan {
    fn name(&self) -> &'static str {
        "port_scan"
    }
    fn description(&self) -> &'static str {
        "Scan common ports on a host (TCP connect)."
    }

    async fn run(&self, target: &str, options: Value) -> Result<ToolOutput> {
        let ports: Vec<u16> = options
            .get("ports")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_u64().map(|n| n as u16)).collect())
            .unwrap_or_else(|| vec![80, 443, 8080, 8443, 3000, 5000, 8000, 9000]);

        let mut open = Vec::new();
        for port in &ports {
            let addr = format!("{}:{}", target, port);
            if tokio::net::TcpStream::connect(&addr).await.is_ok() {
                open.push(*port);
            }
        }

        Ok(ToolOutput {
            tool: self.name().to_string(),
            target: target.to_string(),
            summary: format!("{} open ports on {}", open.len(), target),
            data: json!({ "open_ports": open, "scanned": ports.len() }),
            findings_count: open.len(),
        })
    }
}
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A bug-bounty target within authorized scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub scope_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A scan session tied to one target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub target_id: Uuid,
    pub name: String,
    pub summary: Option<String>,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

/// A finding (endpoint, subdomain, vuln hint, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub target_id: Uuid,
    pub session_id: Option<Uuid>,
    pub kind: FindingKind,
    pub title: String,
    pub detail: Option<String>,
    pub severity: Option<Severity>,
    pub url: Option<String>,
    pub raw_request: Option<String>,
    pub raw_response: Option<String>,
    pub false_positive: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Subdomain,
    Endpoint,
    Port,
    Tech,
    Parameter,
    JsSecret,
    Anomaly,
    VulnerabilityHint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Screenshot evidence linked to a finding/target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub id: Uuid,
    pub target_id: Uuid,
    pub finding_id: Option<Uuid>,
    pub path: String,
    pub alt_text: Option<String>,
    pub captured_at: DateTime<Utc>,
}

/// Researcher decision (approve/reject/FP) feeding the memory loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: Uuid,
    pub target_id: Uuid,
    pub finding_id: Option<Uuid>,
    pub action: DecisionAction,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    Approved,
    Rejected,
    FalsePositive,
    Corrected,
}

/// Feedback entry used to weight future suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub id: Uuid,
    pub target_id: Uuid,
    pub pattern: String,
    pub weight_delta: f64,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
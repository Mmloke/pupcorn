use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::Result;
use crate::models::{Finding, FindingKind, Session, SessionStatus, Target};

pub struct Repo {
    pub pool: SqlitePool,
}

impl Repo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_target(&self, name: &str, base_url: &str, scope_notes: Option<&str>) -> Result<Target> {
        let now = Utc::now();
        let t = Target {
            id: Uuid::new_v4(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            scope_notes: scope_notes.map(String::from),
            created_at: now,
            updated_at: now,
        };
        sqlx::query(
            "INSERT INTO targets (id, name, base_url, scope_notes, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(t.id.to_string())
        .bind(&t.name)
        .bind(&t.base_url)
        .bind(&t.scope_notes)
        .bind(t.created_at.to_rfc3339())
        .bind(t.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(t)
    }

    pub async fn list_targets(&self) -> Result<Vec<Target>> {
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>, String, String)>(
            "SELECT id, name, base_url, scope_notes, created_at, updated_at FROM targets ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::new();
        for (id, name, base_url, scope_notes, ca, ua) in rows {
            out.push(Target {
                id: id.parse().unwrap(),
                name,
                base_url,
                scope_notes,
                created_at: ca.parse().unwrap(),
                updated_at: ua.parse().unwrap(),
            });
        }
        Ok(out)
    }

    pub async fn insert_finding(
        &self,
        target_id: Uuid,
        session_id: Option<Uuid>,
        kind: FindingKind,
        title: &str,
        detail: Option<&str>,
        url: Option<&str>,
    ) -> Result<Finding> {
        let now = Utc::now();
        let f = Finding {
            id: Uuid::new_v4(),
            target_id,
            session_id,
            kind,
            title: title.to_string(),
            detail: detail.map(String::from),
            severity: None,
            url: url.map(String::from),
            raw_request: None,
            raw_response: None,
            false_positive: false,
            created_at: now,
        };
        sqlx::query(
            "INSERT INTO findings (id, target_id, session_id, kind, title, detail, url, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(f.id.to_string())
        .bind(f.target_id.to_string())
        .bind(f.session_id.map(|u| u.to_string()))
        .bind(format!("{:?}", f.kind).to_lowercase())
        .bind(&f.title)
        .bind(&f.detail)
        .bind(&f.url)
        .bind(f.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(f)
    }

    pub async fn list_findings(&self, target_id: Uuid) -> Result<Vec<Finding>> {
        let rows = sqlx::query_as::<_, (String, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, i64, String)>(
            "SELECT id, target_id, session_id, kind, title, detail, url, severity, false_positive, created_at FROM findings WHERE target_id = ? ORDER BY created_at DESC",
        )
        .bind(target_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::new();
        for (id, tid, sid, kind, title, detail, url, _sev, fp, ca) in rows {
            out.push(Finding {
                id: id.parse().unwrap(),
                target_id: tid.parse().unwrap(),
                session_id: sid.and_then(|s| s.parse().ok()),
                kind: parse_kind(&kind),
                title,
                detail,
                severity: None,
                url,
                raw_request: None,
                raw_response: None,
                false_positive: fp != 0,
                created_at: ca.parse().unwrap(),
            });
        }
        Ok(out)
    }

    pub async fn insert_session(&self, target_id: Uuid, name: &str) -> Result<Session> {
        let now = Utc::now();
        let s = Session {
            id: Uuid::new_v4(),
            target_id,
            name: name.to_string(),
            summary: None,
            status: SessionStatus::Active,
            created_at: now,
            updated_at: now,
        };
        sqlx::query(
            "INSERT INTO sessions (id, target_id, name, summary, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(s.id.to_string())
        .bind(s.target_id.to_string())
        .bind(&s.name)
        .bind(&s.summary)
        .bind("active")
        .bind(s.created_at.to_rfc3339())
        .bind(s.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(s)
    }

    pub async fn insert_screenshot(
        &self,
        target_id: Uuid,
        finding_id: Option<Uuid>,
        path: &str,
        alt_text: Option<&str>,
    ) -> Result<crate::models::Screenshot> {
        use chrono::Utc;
        let now = Utc::now();
        let sc = crate::models::Screenshot {
            id: Uuid::new_v4(),
            target_id,
            finding_id,
            path: path.to_string(),
            alt_text: alt_text.map(String::from),
            captured_at: now,
        };
        sqlx::query(
            "INSERT INTO screenshots (id, target_id, finding_id, path, alt_text, captured_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(sc.id.to_string())
        .bind(sc.target_id.to_string())
        .bind(sc.finding_id.map(|u| u.to_string()))
        .bind(&sc.path)
        .bind(&sc.alt_text)
        .bind(sc.captured_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(sc)
    }

    pub async fn audit(&self, target_id: Option<Uuid>, action: &str, detail: Option<&str>) -> Result<()> {
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO audit_log (id, target_id, action, detail, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(target_id.map(|u| u.to_string()))
        .bind(action)
        .bind(detail)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
fn parse_kind(s: &str) -> FindingKind {
    match s.to_lowercase().as_str() {
        "subdomain" => FindingKind::Subdomain,
        "endpoint" => FindingKind::Endpoint,
        "port" => FindingKind::Port,
        "tech" => FindingKind::Tech,
        "parameter" => FindingKind::Parameter,
        "jssecret" | "js_secret" => FindingKind::JsSecret,
        "anomaly" => FindingKind::Anomaly,
        "vulnerabilityhint" | "vulnerability_hint" => FindingKind::VulnerabilityHint,
        _ => FindingKind::Anomaly,
    }
}

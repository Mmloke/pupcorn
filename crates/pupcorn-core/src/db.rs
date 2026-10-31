use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::Result;

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let opts = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;
        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(SCHEMA).execute(&self.pool).await?;
        Ok(())
    }
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS targets (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    base_url    TEXT NOT NULL,
    scope_notes TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    target_id   TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    summary     TEXT,
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS findings (
    id            TEXT PRIMARY KEY,
    target_id     TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    session_id    TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    kind          TEXT NOT NULL,
    title         TEXT NOT NULL,
    detail        TEXT,
    severity      TEXT,
    url           TEXT,
    raw_request   TEXT,
    raw_response  TEXT,
    false_positive INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS screenshots (
    id          TEXT PRIMARY KEY,
    target_id   TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    finding_id  TEXT REFERENCES findings(id) ON DELETE SET NULL,
    path        TEXT NOT NULL,
    alt_text    TEXT,
    captured_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS decisions (
    id          TEXT PRIMARY KEY,
    target_id   TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    finding_id  TEXT REFERENCES findings(id) ON DELETE SET NULL,
    action      TEXT NOT NULL,
    reason      TEXT,
    created_at  TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS feedback (
    id            TEXT PRIMARY KEY,
    target_id     TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    pattern       TEXT NOT NULL,
    weight_delta  REAL NOT NULL,
    reason        TEXT,
    created_at    TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS audit_log (
    id          TEXT PRIMARY KEY,
    target_id   TEXT,
    action      TEXT NOT NULL,
    detail      TEXT,
    created_at  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_findings_target ON findings(target_id);
CREATE INDEX IF NOT EXISTS idx_sessions_target ON sessions(target_id);
CREATE INDEX IF NOT EXISTS idx_screenshots_target ON screenshots(target_id);
CREATE INDEX IF NOT EXISTS idx_decisions_target ON decisions(target_id);
CREATE INDEX IF NOT EXISTS idx_feedback_target ON feedback(target_id);
"#;
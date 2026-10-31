use thiserror::Error;

#[derive(Error, Debug)]
pub enum PupcornError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("tool not found: {0}")]
    ToolNotFound(String),
    #[error("invalid scope: {0}")]
    InvalidScope(String),
    #[error("intrusive action requires approval: {0}")]
    ApprovalRequired(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, PupcornError>;
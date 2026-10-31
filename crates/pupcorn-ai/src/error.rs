use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("llm error: {0}")]
    Llm(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("core error: {0}")]
    Core(#[from] pupcorn_core::PupcornError),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AiError>;
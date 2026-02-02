use thiserror::Error;

#[derive(Error, Debug)]
pub enum KenError {
    #[error("Not initialized: run 'ken init' first")]
    NotInitialized,

    #[error("Already initialized: .ken directory exists")]
    AlreadyInitialized,

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

pub type Result<T> = std::result::Result<T, KenError>;

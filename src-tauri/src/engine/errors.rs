use thiserror::Error;

pub type EngineResult<T> = Result<T, EngineError>;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum EngineError {
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
    #[error("unsupported archive format: {0}")]
    UnsupportedFormat(String),
    #[error("invalid archive path: {0}")]
    InvalidArchivePath(String),
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("not enough free space for the current file")]
    InsufficientSpace,
    #[error("archive error: {0}")]
    Archive(String),
    #[error("password required")]
    PasswordRequired,
    #[error("archive is empty")]
    EmptyArchive,
    #[error("disk full")]
    DiskFull,
    #[error("corrupt entry: {path} - {detail}")]
    CorruptEntry { path: String, detail: String },
}

impl EngineError {
    pub fn user_message(&self) -> String {
        match self {
            EngineError::Io { context, source } => format!("{context}: {source}"),
            EngineError::UnsupportedFormat(format) => {
                format!("{format} archives are detected but are not yet extractable in this build.")
            }
            EngineError::InvalidArchivePath(path) => {
                format!("The archive entry path is not safe to extract: {path}")
            }
            EngineError::SessionNotFound(_) => {
                "This extraction session expired. Analyze the archive again.".to_string()
            }
            EngineError::InsufficientSpace => {
                "The target drive does not have enough free space for the current file.".to_string()
            }
            EngineError::Archive(message) => message.clone(),
            EngineError::PasswordRequired => "This archive requires a password to extract.".to_string(),
            EngineError::EmptyArchive => "The archive does not contain any extractable files.".to_string(),
            EngineError::DiskFull => "The target drive is full.".to_string(),
            EngineError::CorruptEntry { path, detail } => format!("Corrupt entry {path}: {detail}"),
        }
    }
}

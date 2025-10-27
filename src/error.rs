use std::fmt;

#[derive(Debug)]
pub enum ChaosError {
    Storage(String),
    Network(String),
    Parse(String),
    Generation(String),
    Analysis(String),
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    Rusqlite(rusqlite::Error),
    Serde(serde_json::Error),
}

impl fmt::Display for ChaosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(msg) => write!(f, "Storage error: {}", msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Parse(msg) => write!(f, "Parse error: {}", msg),
            Self::Generation(msg) => write!(f, "Generation error: {}", msg),
            Self::Analysis(msg) => write!(f, "Analysis error: {}", msg),
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Reqwest(err) => write!(f, "HTTP error: {}", err),
            Self::Rusqlite(err) => write!(f, "Database error: {}", err),
            Self::Serde(err) => write!(f, "Serialization error: {}", err),
        }
    }
}

impl std::error::Error for ChaosError {}

impl From<std::io::Error> for ChaosError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<reqwest::Error> for ChaosError {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<rusqlite::Error> for ChaosError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Rusqlite(err)
    }
}

impl From<serde_json::Error> for ChaosError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serde(err)
    }
}

impl From<anyhow::Error> for ChaosError {
    fn from(err: anyhow::Error) -> Self {
        Self::Network(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ChaosError>;

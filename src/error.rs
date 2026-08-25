use std::fmt;

#[derive(Debug)]
pub enum CruxError {
    Git(String),
    Io(String),
}

impl fmt::Display for CruxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CruxError::Git(msg) => write!(f, "git: {msg}"),
            CruxError::Io(msg) => write!(f, "io: {msg}"),
        }
    }
}

impl From<std::io::Error> for CruxError {
    fn from(e: std::io::Error) -> Self {
        CruxError::Io(e.to_string())
    }
}

impl From<String> for CruxError {
    fn from(s: String) -> Self {
        CruxError::Git(s)
    }
}

impl From<serde_json::Error> for CruxError {
    fn from(e: serde_json::Error) -> Self {
        CruxError::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CruxError>;

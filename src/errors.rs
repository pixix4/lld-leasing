#[derive(Debug, Clone)]
pub enum LldError {
    WrappedError(&'static str, String),
}

/// Helper for `ServiceError` result
pub type LldResult<T> = Result<T, LldError>;

impl From<hyper::Error> for LldError {
    fn from(error: hyper::Error) -> Self {
        LldError::WrappedError("hyper error", format!("{}", error))
    }
}

impl From<hyper::http::Error> for LldError {
    fn from(error: hyper::http::Error) -> Self {
        LldError::WrappedError("hyper http error", format!("{}", error))
    }
}

impl From<std::string::FromUtf8Error> for LldError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        LldError::WrappedError("utf8 decode error", format!("{}", error))
    }
}

impl From<serde_json::Error> for LldError {
    fn from(error: serde_json::Error) -> Self {
        LldError::WrappedError("serde json error", format!("{}", error))
    }
}

impl From<sqlite::Error> for LldError {
    fn from(error: sqlite::Error) -> Self {
        LldError::WrappedError("sqlite error", format!("{}", error))
    }
}

impl From<std::io::Error> for LldError {
    fn from(error: std::io::Error) -> Self {
        LldError::WrappedError("io error", format!("{}", error))
    }
}

impl From<tokio::task::JoinError> for LldError {
    fn from(error: tokio::task::JoinError) -> Self {
        LldError::WrappedError("tokio join error", format!("{}", error))
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for LldError {
    fn from(error: tokio::sync::mpsc::error::SendError<T>) -> Self {
        LldError::WrappedError("tokio mpsc send error", format!("{}", error))
    }
}

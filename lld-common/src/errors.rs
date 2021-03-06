#[derive(Debug, Clone)]
pub enum LldError {
    WrappedError(&'static str, String),
    DatabaseError {
        code: Option<isize>,
        message: Option<String>,
    },
}

pub type LldResult<T> = Result<T, LldError>;

impl From<reqwest::Error> for LldError {
    fn from(error: reqwest::Error) -> Self {
        LldError::WrappedError("Reqwest http error", format!("{}", error))
    }
}

impl From<openssl::error::ErrorStack> for LldError {
    fn from(error: openssl::error::ErrorStack) -> Self {
        LldError::WrappedError("OpenSSL error stack", format!("{}", error))
    }
}

impl From<openssl::ssl::Error> for LldError {
    fn from(error: openssl::ssl::Error) -> Self {
        LldError::WrappedError("OpenSSL ssl error", format!("{}", error))
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

impl From<std::io::Error> for LldError {
    fn from(error: std::io::Error) -> Self {
        LldError::WrappedError("io error", format!("{}", error))
    }
}

impl From<std::fmt::Error> for LldError {
    fn from(error: std::fmt::Error) -> Self {
        LldError::WrappedError("fmt error", format!("{}", error))
    }
}

impl From<tokio::task::JoinError> for LldError {
    fn from(error: tokio::task::JoinError) -> Self {
        LldError::WrappedError("tokio join error", format!("{}", error))
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for LldError {
    fn from(error: tokio::sync::oneshot::error::RecvError) -> Self {
        LldError::WrappedError("tokio oneshot receive errro", format!("{}", error))
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for LldError {
    fn from(error: tokio::sync::mpsc::error::SendError<T>) -> Self {
        LldError::WrappedError("tokio mpsc send error", format!("{}", error))
    }
}

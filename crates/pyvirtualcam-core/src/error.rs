use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidArgument(String),
    Runtime(String),
    Io {
        context: String,
        source: std::io::Error,
    },
}

impl Error {
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::InvalidArgument(message.into())
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime(message.into())
    }

    pub fn io(context: impl Into<String>) -> Self {
        Self::Io {
            context: context.into(),
            source: std::io::Error::last_os_error(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) | Self::Runtime(message) => f.write_str(message),
            Self::Io { context, source } => {
                write!(f, "{context}: {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

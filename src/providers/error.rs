#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error during network communication")]
    TransportError { source: reqwest::Error },
    #[error("invalid url")]
    InvalidUrl { source: url::ParseError },
}

impl From<reqwest::Error> for Error {
    fn from(source: reqwest::Error) -> Self {
        Self::TransportError { source }
    }
}

impl From<url::ParseError> for Error {
    fn from(source: url::ParseError) -> Self {
        Self::InvalidUrl { source }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

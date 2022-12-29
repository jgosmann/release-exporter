#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error during network communication")]
    TransportError { source: reqwest::Error },
}

impl From<reqwest::Error> for Error {
    fn from(source: reqwest::Error) -> Self {
        Self::TransportError { source }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

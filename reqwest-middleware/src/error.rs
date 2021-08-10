use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    /// There was an error running some middleware
    #[error("Middleware error: {0}")]
    Middleware(#[from] anyhow::Error),
    /// Error from the underlying reqwest client
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl Error {
    pub fn middleware<E>(err: E) -> Self
    where
        E: 'static + Send + Sync + std::error::Error,
    {
        Error::Middleware(err.into())
    }
}

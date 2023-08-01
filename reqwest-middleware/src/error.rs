use reqwest::{StatusCode, Url};
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

    /// Returns a possible URL related to this error.
    pub fn url(&self) -> Option<&Url> {
        match self {
            Error::Middleware(_) => None,
            Error::Reqwest(e) => e.url(),
        }
    }

    /// Returns a mutable reference to the URL related to this error
    ///
    /// This is useful if you need to remove sensitive information from the URL
    /// (e.g. an API key in the query), but do not want to remove the URL
    /// entirely.
    pub fn url_mut(&mut self) -> Option<&mut Url> {
        match self {
            Error::Middleware(_) => None,
            Error::Reqwest(e) => e.url_mut(),
        }
    }

    /// Add a url related to this error (overwriting any existing)
    pub fn with_url(self, url: Url) -> Self {
        match self {
            Error::Middleware(_) => self,
            Error::Reqwest(e) => e.with_url(url).into(),
        }
    }

    /// Strip the related url from this error (if, for example, it contains
    /// sensitive information)
    pub fn without_url(self) -> Self {
        match self {
            Error::Middleware(_) => self,
            Error::Reqwest(e) => e.without_url().into(),
        }
    }

    /// Returns true if the error is from a type Builder.
    pub fn is_builder(&self) -> bool {
        match self {
            Error::Middleware(_) => false,
            Error::Reqwest(e) => e.is_builder(),
        }
    }

    /// Returns true if the error is from a `RedirectPolicy`.
    pub fn is_redirect(&self) -> bool {
        match self {
            Error::Middleware(_) => false,
            Error::Reqwest(e) => e.is_redirect(),
        }
    }

    /// Returns true if the error is from `Response::error_for_status`.
    pub fn is_status(&self) -> bool {
        match self {
            Error::Middleware(_) => false,
            Error::Reqwest(e) => e.is_status(),
        }
    }

    /// Returns true if the error is related to a timeout.
    pub fn is_timeout(&self) -> bool {
        match self {
            Error::Middleware(_) => false,
            Error::Reqwest(e) => e.is_timeout(),
        }
    }

    /// Returns true if the error is related to the request
    pub fn is_request(&self) -> bool {
        match self {
            Error::Middleware(_) => false,
            Error::Reqwest(e) => e.is_request(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Returns true if the error is related to connect
    pub fn is_connect(&self) -> bool {
        match self {
            Error::Middleware(_) => false,
            Error::Reqwest(e) => e.is_connect(),
        }
    }

    /// Returns true if the error is related to the request or response body
    pub fn is_body(&self) -> bool {
        match self {
            Error::Middleware(_) => false,
            Error::Reqwest(e) => e.is_body(),
        }
    }

    /// Returns true if the error is related to decoding the response's body
    pub fn is_decode(&self) -> bool {
        match self {
            Error::Middleware(_) => false,
            Error::Reqwest(e) => e.is_decode(),
        }
    }

    /// Returns the status code, if the error was generated from a response.
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Error::Middleware(_) => None,
            Error::Reqwest(e) => e.status(),
        }
    }
}

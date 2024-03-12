use crate::retryable_strategy::{DefaultRetryableStrategy, RetryableStrategy};
use reqwest_middleware::Error;

/// Classification of an error/status returned by request.
#[derive(PartialEq, Eq)]
pub enum Retryable {
    /// The failure was due to something that might resolve in the future.
    Transient,
    /// Unresolvable error.
    Fatal,
}

impl Retryable {
    /// Try to map a `reqwest` response into `Retryable`.
    ///
    /// Returns `None` if the response object does not contain any errors.
    ///
    pub fn from_reqwest_response(res: &Result<reqwest::Response, Error>) -> Option<Self> {
        DefaultRetryableStrategy.handle(res)
    }
}

impl From<&reqwest::Error> for Retryable {
    fn from(_status: &reqwest::Error) -> Retryable {
        Retryable::Transient
    }
}

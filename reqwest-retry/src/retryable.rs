use http::StatusCode;
use reqwest_middleware::Error;

/// Classification of an error/status returned by request.
#[derive(PartialEq, Eq)]
pub enum Retryable {
    /// The failure was due to something tha might resolve in the future.
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
        match res {
            Ok(success) => {
                let status = success.status();
                if status.is_server_error() {
                    Some(Retryable::Transient)
                } else if status.is_client_error()
                    && status != StatusCode::REQUEST_TIMEOUT
                    && status != StatusCode::TOO_MANY_REQUESTS
                {
                    Some(Retryable::Fatal)
                } else if status.is_success() {
                    None
                } else if status == StatusCode::REQUEST_TIMEOUT
                    || status == StatusCode::TOO_MANY_REQUESTS
                {
                    Some(Retryable::Transient)
                } else {
                    Some(Retryable::Fatal)
                }
            }
            Err(error) => match error {
                // If something fails in the middleware we're screwed.
                Error::Middleware(_) => Some(Retryable::Fatal),
                Error::Reqwest(error) => {
                    if error.is_timeout() || error.is_connect() {
                        Some(Retryable::Transient)
                    } else if error.is_body()
                        || error.is_decode()
                        || error.is_request()
                        || error.is_builder()
                        || error.is_redirect()
                    {
                        Some(Retryable::Fatal)
                    } else {
                        // We omit checking if error.is_status() since we check that already.
                        // However, if Response::error_for_status is used the status will still
                        // remain in the response object.
                        None
                    }
                }
            },
        }
    }
}

impl From<&reqwest::Error> for Retryable {
    fn from(_status: &reqwest::Error) -> Retryable {
        Retryable::Transient
    }
}

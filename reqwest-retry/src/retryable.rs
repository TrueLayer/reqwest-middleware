use std::{error::Error as StdError, io};

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
                        || error.is_builder()
                        || error.is_redirect()
                    {
                        Some(Retryable::Fatal)
                    } else if error.is_request() {
                        // It seems that hyper::Error(IncompleteMessage) is not correctly handled by reqwest.
                        // Here we check if the Reqwest error was originated by hyper and map it consistently.
                        if let Some(hyper_error) = get_source_error_type::<hyper::Error>(&error) {
                            // The hyper::Error(IncompleteMessage) is raised if the HTTP response is well formatted but does not contain all the bytes.
                            // This can happen when the server has started sending back the response but the connection is cut halfway thorugh.
                            // We can safely retry the call, hence marking this error as [`Retryable::Transient`].
                            if hyper_error.is_incomplete_message() || hyper_error.is_canceled() {
                                Some(Retryable::Transient)

                            // Try and downcast the hyper error to io::Error if that is the
                            // underlying error, and try and classify it.
                            } else if let Some(io_error) = try_io_error(hyper_error) {
                                Some(classify_io_error(io_error))
                            } else {
                                Some(Retryable::Fatal)
                            }
                        } else {
                            Some(Retryable::Fatal)
                        }
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

fn classify_io_error(error: &io::Error) -> Retryable {
    match error.kind() {
        io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted => Retryable::Transient,
        _ => Retryable::Fatal,
    }
}

fn try_io_error(error: &hyper::Error) -> Option<&io::Error> {
    // We would prefer being able to check that the error kind
    // is IO here, but hyper::Error does not expose it.
    error
        .source()
        .and_then(|err| err.downcast_ref::<io::Error>())
}

/// Downcasts the given err source into T.
fn get_source_error_type<T: std::error::Error + 'static>(
    err: &dyn std::error::Error,
) -> Option<&T> {
    let mut source = err.source();

    while let Some(err) = source {
        if let Some(hyper_err) = err.downcast_ref::<T>() {
            return Some(hyper_err);
        }

        source = err.source();
    }
    None
}

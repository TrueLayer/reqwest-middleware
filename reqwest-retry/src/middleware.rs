//! `RetryTransientMiddleware` implements retrying requests on transient errors.

use std::pin::Pin;
use std::task::{ready, Context, Poll};

use crate::retryable::Retryable;
use chrono::Utc;
use futures::Future;
use pin_project_lite::pin_project;
use reqwest::Response;
use reqwest_middleware::{Error, MiddlewareRequest};
use retry_policies::RetryPolicy;
use task_local_extensions::Extensions;
use tokio::time::Sleep;
use tower::retry::{Policy, Retry};
use tower::{Layer, Service};

/// `RetryTransientMiddleware` offers retry logic for requests that fail in a transient manner
/// and can be safely executed again.
///
/// Currently, it allows setting a [RetryPolicy][retry_policies::RetryPolicy] algorithm for calculating the __wait_time__
/// between each request retry.
///
///```rust
///     use reqwest_middleware::ClientBuilder;
///     use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
///     use reqwest::Client;
///
///     // We create a ExponentialBackoff retry policy which implements `RetryPolicy`.
///     let retry_policy = ExponentialBackoff {
///         /// How many times the policy will tell the middleware to retry the request.
///         max_n_retries: 3,
///         max_retry_interval: std::time::Duration::from_millis(30),
///         min_retry_interval: std::time::Duration::from_millis(100),
///         backoff_exponent: 2,
///     };
///
///     let retry_transient_middleware = RetryTransientMiddleware::new_with_policy(retry_policy);
///     let client = ClientBuilder::new(Client::new()).with(retry_transient_middleware).build();
///```
///
/// # Note
///
/// This middleware always errors when given requests with streaming bodies, before even executing
/// the request. When this happens you'll get an [`Error::Middleware`] with the message
/// 'Request object is not clonable. Are you passing a streaming body?'.
///
/// Some workaround suggestions:
/// * If you can fit the data in memory, you can instead build static request bodies e.g. with
/// `Body`'s `From<String>` or `From<Bytes>` implementations.
/// * You can wrap this middleware in a custom one which skips retries for streaming requests.
/// * You can write a custom retry middleware that builds new streaming requests from the data
/// source directly, avoiding the issue of streaming requests not being clonable.
pub struct RetryTransientMiddleware<T: RetryPolicy + Send + Sync + 'static> {
    retry_policy: T,
}

impl<T: RetryPolicy + Send + Sync> RetryTransientMiddleware<T> {
    /// Construct `RetryTransientMiddleware` with  a [retry_policy][retry_policies::RetryPolicy].
    pub fn new_with_policy(retry_policy: T) -> Self {
        Self { retry_policy }
    }
}

impl<T: RetryPolicy + Clone + Send + Sync + 'static, Svc> Layer<Svc> for RetryTransientMiddleware<T>
where
    Svc: Service<MiddlewareRequest, Response = Response, Error = Error>,
{
    type Service = Retry<TowerRetryPolicy<T>, Svc>;

    fn layer(&self, inner: Svc) -> Self::Service {
        Retry::new(
            TowerRetryPolicy {
                n_past_retries: 0,
                retry_policy: self.retry_policy.clone(),
            },
            inner,
        )
    }
}

#[derive(Clone)]
pub struct TowerRetryPolicy<T> {
    n_past_retries: u32,
    retry_policy: T,
}

pin_project! {
    pub struct RetryFuture<T>
    {
        retry: Option<TowerRetryPolicy<T>>,
        #[pin]
        sleep: Sleep,
    }
}

impl<T> Future for RetryFuture<T> {
    type Output = TowerRetryPolicy<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        ready!(this.sleep.poll(cx));
        Poll::Ready(
            this.retry
                .take()
                .expect("poll should not be called more than once"),
        )
    }
}

impl<T: RetryPolicy + Clone> Policy<MiddlewareRequest, Response, Error> for TowerRetryPolicy<T> {
    type Future = RetryFuture<T>;

    fn retry(
        &self,
        _req: &MiddlewareRequest,
        result: std::result::Result<&Response, &Error>,
    ) -> Option<Self::Future> {
        // We classify the response which will return None if not
        // errors were returned.
        match Retryable::from_reqwest_response(result) {
            Some(Retryable::Transient) => {
                // If the response failed and the error type was transient
                // we can safely try to retry the request.
                let retry_decicion = self.retry_policy.should_retry(self.n_past_retries);
                if let retry_policies::RetryDecision::Retry { execute_after } = retry_decicion {
                    let duration = (execute_after - Utc::now()).to_std().ok()?;
                    // Sleep the requested amount before we try again.
                    tracing::warn!(
                        "Retry attempt #{}. Sleeping {:?} before the next attempt",
                        self.n_past_retries,
                        duration
                    );
                    let sleep = tokio::time::sleep(duration);
                    Some(RetryFuture {
                        retry: Some(TowerRetryPolicy {
                            n_past_retries: self.n_past_retries + 1,
                            retry_policy: self.retry_policy.clone(),
                        }),
                        sleep,
                    })
                } else {
                    None
                }
            }
            Some(_) | None => None,
        }
    }

    fn clone_request(&self, req: &MiddlewareRequest) -> Option<MiddlewareRequest> {
        Some(MiddlewareRequest {
            request: req.request.try_clone()?,
            extensions: Extensions::new(),
        })
    }
}

//! `RetryTransientMiddleware` implements retrying requests on transient errors.

use crate::retryable::Retryable;
use anyhow::anyhow;
use chrono::Utc;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Middleware, Next, Result};
use retry_policies::RetryPolicy;
use task_local_extensions::Extensions;

/// We limit the number of retries to a maximum of `10` to avoid stack-overflow issues due to the recursion.
static MAXIMUM_NUMBER_OF_RETRIES: u32 = 10;

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
pub struct RetryTransientMiddleware<T: RetryPolicy + Send + Sync + 'static> {
    retry_policy: T,
}

impl<T: RetryPolicy + Send + Sync> RetryTransientMiddleware<T> {
    /// Construct `RetryTransientMiddleware` with  a [retry_policy][retry_policies::RetryPolicy].
    pub fn new_with_policy(retry_policy: T) -> Self {
        Self { retry_policy }
    }
}

#[async_trait::async_trait]
impl<T: RetryPolicy + Send + Sync> Middleware for RetryTransientMiddleware<T> {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        // TODO: Ideally we should create a new instance of the `Extensions` map to pass
        // downstream. This will guard against previous retries poluting `Extensions`.
        // That is, we only return what's populated in the typemap for the last retry attempt
        // and copy those into the the `global` Extensions map.
        self.execute_with_retry(req, next, extensions).await
    }
}

impl<T: RetryPolicy + Send + Sync> RetryTransientMiddleware<T> {
    /// This function will try to execute the request, if it fails
    /// with an error classified as transient it will call itself
    /// to retry the request.
    async fn execute_with_retry<'a>(
        &'a self,
        req: Request,
        next: Next<'a>,
        ext: &'a mut Extensions,
    ) -> Result<Response> {
        let mut n_past_retries = 0;
        loop {
            // Cloning the request object before-the-fact is not ideal..
            // However, if the body of the request is not static, e.g of type `Bytes`,
            // the Clone operation should be of constant complexity and not O(N)
            // since the byte abstraction is a shared pointer over a buffer.
            let duplicate_request = req.try_clone().ok_or_else(|| {
                Error::Middleware(anyhow!(
                    "Request object is not clonable. Are you passing a streaming body?".to_string()
                ))
            })?;

            let result = next.clone().run(duplicate_request, ext).await;

            // We classify the response which will return None if not
            // errors were returned.
            break match Retryable::from_reqwest_response(&result) {
                Some(retryable)
                    if retryable == Retryable::Transient
                        && n_past_retries < MAXIMUM_NUMBER_OF_RETRIES =>
                {
                    // If the response failed and the error type was transient
                    // we can safely try to retry the request.
                    let retry_decicion = self.retry_policy.should_retry(n_past_retries);
                    if let retry_policies::RetryDecision::Retry { execute_after } = retry_decicion {
                        let duration = (execute_after - Utc::now())
                            .to_std()
                            .map_err(Error::middleware)?;
                        // Sleep the requested amount before we try again.
                        tracing::warn!(
                            "Retry attempt #{}. Sleeping {:?} before the next attempt",
                            n_past_retries,
                            duration
                        );
                        tokio::time::sleep(duration).await;

                        n_past_retries += 1;
                        continue;
                    } else {
                        result
                    }
                }
                Some(_) | None => result,
            };
        }
    }
}

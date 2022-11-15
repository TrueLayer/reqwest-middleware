//! `RetryTransientMiddleware` implements retrying requests on transient errors.

use std::pin::Pin;
use std::task::{ready, Context, Poll};

use crate::retryable::Retryable;
use chrono::Utc;
use futures::Future;
use pin_project_lite::pin_project;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Layer, Service};
use retry_policies::RetryPolicy;
use task_local_extensions::Extensions;
use tokio::time::Sleep;

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

impl<T, Svc> Layer<Svc> for RetryTransientMiddleware<T>
where
    T: RetryPolicy + Clone + Send + Sync + 'static,
{
    type Service = Retry<TowerRetryPolicy<T>, Svc>;

    fn layer(&self, inner: Svc) -> Self::Service {
        Retry {
            policy: TowerRetryPolicy {
                n_past_retries: 0,
                retry_policy: self.retry_policy.clone(),
            },
            service: inner,
        }
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

impl<T: RetryPolicy + Clone> Policy for TowerRetryPolicy<T> {
    type Future = RetryFuture<T>;

    fn retry(&self, _req: &Request, result: &Result<Response, Error>) -> Option<Self::Future> {
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

    fn clone_request(&self, req: &Request) -> Option<Request> {
        req.try_clone()
    }
}

pub trait Policy: Sized {
    /// The [`Future`] type returned by [`Policy::retry`].
    type Future: Future<Output = Self>;

    /// Check the policy if a certain request should be retried.
    ///
    /// This method is passed a reference to the original request, and either
    /// the [`Service::Response`] or [`Service::Error`] from the inner service.
    ///
    /// If the request should **not** be retried, return `None`.
    ///
    /// If the request *should* be retried, return `Some` future of a new
    /// policy that would apply for the next request attempt.
    ///
    /// [`Service::Response`]: crate::Service::Response
    /// [`Service::Error`]: crate::Service::Error
    fn retry(&self, req: &Request, result: &Result<Response, Error>) -> Option<Self::Future>;

    /// Tries to clone a request before being passed to the inner service.
    ///
    /// If the request cannot be cloned, return [`None`].
    fn clone_request(&self, req: &Request) -> Option<Request>;
}

pin_project! {
    /// Configure retrying requests of "failed" responses.
    ///
    /// A [`Policy`] classifies what is a "failed" response.
    #[derive(Clone, Debug)]
    pub struct Retry<P, S> {
        #[pin]
        policy: P,
        service: S,
    }
}

impl<P, S> Service for Retry<P, S>
where
    P: 'static + Policy + Clone,
    S: 'static + Service + Clone,
{
    type Future = ResponseFuture<P, S>;

    fn call(&mut self, request: Request, ext: &mut Extensions) -> Self::Future {
        let cloned = self.policy.clone_request(&request);
        let future = self.service.call(request, ext);

        ResponseFuture::new(cloned, self.clone(), future)
    }

    // fn call(&mut self, request: Request) -> Self::Future {
    //     let cloned = self.policy.clone_request(&request);
    //     let future = self.service.call(request);

    //     ResponseFuture::new(cloned, self.clone(), future)
    // }
}

pin_project! {
    /// The [`Future`] returned by a [`Retry`] service.
    #[derive(Debug)]
    pub struct ResponseFuture<P, S>
    where
        P: Policy,
        S: Service,
    {
        request: Option<Request>,
        #[pin]
        retry: Retry<P, S>,
        #[pin]
        state: State<S::Future, P::Future>,
    }
}

pin_project! {
    #[project = StateProj]
    #[derive(Debug)]
    enum State<F, P> {
        // Polling the future from [`Service::call`]
        Called {
            #[pin]
            future: F
        },
        // Polling the future from [`Policy::retry`]
        Checking {
            #[pin]
            checking: P
        },
        // Polling [`Service::poll_ready`] after [`Checking`] was OK.
        Retrying,
    }
}

impl<P, S> ResponseFuture<P, S>
where
    P: Policy,
    S: Service,
{
    pub(crate) fn new(
        request: Option<Request>,
        retry: Retry<P, S>,
        future: S::Future,
    ) -> ResponseFuture<P, S> {
        ResponseFuture {
            request,
            retry,
            state: State::Called { future },
        }
    }
}

impl<P, S> Future for ResponseFuture<P, S>
where
    P: Policy + Clone,
    S: Service + Clone,
{
    type Output = Result<Response, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            match this.state.as_mut().project() {
                StateProj::Called { future } => {
                    let result = ready!(future.poll(cx));
                    if let Some(ref req) = this.request {
                        match this.retry.policy.retry(req, &result) {
                            Some(checking) => {
                                this.state.set(State::Checking { checking });
                            }
                            None => return Poll::Ready(result),
                        }
                    } else {
                        // request wasn't cloned, so no way to retry it
                        return Poll::Ready(result);
                    }
                }
                StateProj::Checking { checking } => {
                    this.retry
                        .as_mut()
                        .project()
                        .policy
                        .set(ready!(checking.poll(cx)));
                    this.state.set(State::Retrying);
                }
                StateProj::Retrying => {
                    let req = this
                        .request
                        .take()
                        .expect("retrying requires cloned request");
                    *this.request = this.retry.policy.clone_request(&req);
                    this.state.set(State::Called {
                        future: this
                            .retry
                            .as_mut()
                            .project()
                            .service
                            .call(req, &mut Extensions::new()),
                    });
                }
            }
        }
    }
}

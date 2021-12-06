use futures::future::{BoxFuture, FutureExt, TryFutureExt};
use reqwest::{Client, Request, Response};
use std::sync::Arc;
use task_local_extensions::Extensions;

use crate::error::{Error, Result};

/// When attached to a [`ClientWithMiddleware`] (generally using [`with`]), middleware is run
/// whenever the client issues a request, in the order it was attached.
///
/// # Example
///
/// ```
/// use reqwest::{Client, Request, Response};
/// use reqwest_middleware::{ClientBuilder, Middleware, Next, Result};
/// use task_local_extensions::Extensions;
///
/// #[derive(Debug)]
/// struct TransparentMiddleware;
///
/// #[async_trait::async_trait]
/// impl Middleware for TransparentMiddleware {
///     async fn handle(
///         &self,
///         req: Request,
///         extensions: &mut Extensions,
///         next: Next<'_>,
///     ) -> Result<Response> {
///         next.run(req, extensions).await
///     }
/// }
/// ```
///
/// [`ClientWithMiddleware`]: crate::ClientWithMiddleware
/// [`with`]: crate::ClientBuilder::with
#[async_trait::async_trait]
pub trait Middleware: 'static + Send + Sync + std::fmt::Debug {
    /// Invoked with a request before sending it. If you want to continue processing the request,
    /// you should explicitly call `next.run(req, extensions)`.
    ///
    /// If you need to forward data down the middleware stack, you can use the `extensions`
    /// argument.
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response>;
}

#[async_trait::async_trait]
impl<F> Middleware for F
where
    F: Send
        + Sync
        + std::fmt::Debug
        + 'static
        + for<'a> Fn(Request, &'a mut Extensions, Next<'a>) -> BoxFuture<'a, Result<Response>>,
{
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        (self)(req, extensions, next).await
    }
}

/// Next encapsulates the remaining middleware chain to run in [`Middleware::handle`]. You can
/// forward the request down the chain with [`run`].
///
/// [`Middleware::handle`]: Middleware::handle
/// [`run`]: Self::run
#[derive(Clone)]
pub struct Next<'a> {
    client: &'a Client,
    middlewares: &'a [Arc<dyn Middleware>],
}

impl<'a> Next<'a> {
    pub(crate) fn new(client: &'a Client, middlewares: &'a [Arc<dyn Middleware>]) -> Self {
        Next {
            client,
            middlewares,
        }
    }

    pub fn run(
        mut self,
        req: Request,
        extensions: &'a mut Extensions,
    ) -> BoxFuture<'a, Result<Response>> {
        if let Some((current, rest)) = self.middlewares.split_first() {
            self.middlewares = rest;
            current.handle(req, extensions, self).boxed()
        } else {
            self.client.execute(req).map_err(Error::from).boxed()
        }
    }
}

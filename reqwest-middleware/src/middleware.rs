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
pub trait Middleware: 'static + Send + Sync {
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

/// A middleware that inserts the value into the [`Extensions`] during the call.
///
/// This is a good way to inject extensions to middleware deeper in the stack
///
/// ```
/// use reqwest::{Client, Request, Response};
/// use reqwest_middleware::{ClientBuilder, Middleware, Next, Result, Extension};
/// use task_local_extensions::Extensions;
///
/// struct LoggingMiddleware;
/// #[derive(Clone)]
/// struct LogName(String);
///
/// #[async_trait::async_trait]
/// impl Middleware for LoggingMiddleware {
///     async fn handle(
///         &self,
///         req: Request,
///         extensions: &mut Extensions,
///         next: Next<'_>,
///     ) -> Result<Response> {
///         // get the log name or default to "unknown"
///         let name = extensions
///             .get()
///             .map(|LogName(name)| name.as_str())
///             .unwrap_or("unknown")
///             .to_owned();
///         println!("[{name}] Request started {req:?}");
///         let res = next.run(req, extensions).await;
///         println!("[{name}] Result: {res:?}");
///         res
///     }
/// }
///
/// async fn run() {
///     let reqwest_client = Client::builder().build().unwrap();
///     let client = ClientBuilder::new(reqwest_client)
///         .with(Extension(LogName("my-client".into())))
///         .with(LoggingMiddleware)
///         .build();
///     let resp = client.get("https://truelayer.com").send().await.unwrap();
///     println!("TrueLayer page HTML: {}", resp.text().await.unwrap());
/// }
/// ```
pub struct Extension<T>(pub T);

#[async_trait::async_trait]
impl<T: Send + Sync + Clone + 'static> Middleware for Extension<T> {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        extensions.insert(self.0.clone());
        next.run(req, extensions).await
    }
}

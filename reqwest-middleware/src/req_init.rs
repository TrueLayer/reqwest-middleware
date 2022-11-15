use reqwest::RequestBuilder;
use task_local_extensions::Extensions;

use crate::Identity;

/// When attached to a [`ClientWithMiddleware`] (generally using [`with_init`]), it is run
/// whenever the client starts building a request, in the order it was attached.
///
/// # Example
///
/// ```
/// use reqwest::RequestBuilder;
/// use reqwest_middleware::RequestInitialiser;
/// use task_local_extensions::Extensions;
///
/// struct AuthInit;
///
/// impl RequestInitialiser for AuthInit {
///     fn init(&self, req: RequestBuilder, ext: &mut Extensions) -> RequestBuilder {
///         req.bearer_auth("my_auth_token")
///     }
/// }
/// ```
///
/// [`ClientWithMiddleware`]: crate::ClientWithMiddleware
/// [`with_init`]: crate::ClientBuilder::with_init
pub trait RequestInitialiser: 'static + Send + Sync {
    fn init(&self, req: RequestBuilder, ext: &mut Extensions) -> RequestBuilder;
}

impl RequestInitialiser for Identity {
    fn init(&self, req: RequestBuilder, _: &mut Extensions) -> RequestBuilder {
        req
    }
}

/// Two [`RequestInitialiser`]s chained together.
#[derive(Clone)]
pub struct RequestStack<Inner, Outer> {
    pub(crate) inner: Inner,
    pub(crate) outer: Outer,
}

impl<I, O> RequestInitialiser for RequestStack<I, O>
where
    I: RequestInitialiser,
    O: RequestInitialiser,
{
    fn init(&self, req: RequestBuilder, ext: &mut Extensions) -> RequestBuilder {
        let req = self.inner.init(req, ext);
        self.outer.init(req, ext)
    }
}

/// A middleware that inserts the value into the [`Extensions`](task_local_extensions::Extensions) during the call.
///
/// This is a good way to inject extensions to middleware deeper in the stack
///
/// ```
/// use reqwest::{Client, Request, Response};
/// use reqwest_middleware::{ClientBuilder, Error, Extension, Layer, Service};
/// use task_local_extensions::Extensions;
/// use futures::future::{BoxFuture, FutureExt};
/// use std::task::{Context, Poll};
///
/// #[derive(Clone)]
/// struct LogName(&'static str);
///
/// struct LoggingLayer;
/// struct LoggingService<S>(S);
///
/// impl<S> Layer<S> for LoggingLayer {
///     type Service = LoggingService<S>;
///
///     fn layer(&self, inner: S) -> Self::Service {
///         LoggingService(inner)
///     }
/// }
///
/// impl<S> Service for LoggingService<S>
/// where
///     S: Service,
///     S::Future: Send + 'static,
/// {
///     type Future = BoxFuture<'static, Result<Response, Error>>;
///     
///     fn call(&mut self, req: Request, ext: &mut Extensions) -> Self::Future {
///         // get the log name or default to "unknown"
///         let name = ext
///             .get()
///             .map(|&LogName(name)| name)
///             .unwrap_or("unknown");
///         println!("[{name}] Request started {req:?}");
///         let fut = self.0.call(req, ext);
///         async move {
///             let res = fut.await;
///             println!("[{name}] Result: {res:?}");
///             res
///         }.boxed()
///     }
/// }
///
/// async fn run() {
///     let reqwest_client = Client::builder().build().unwrap();
///     let client = ClientBuilder::new(reqwest_client)
///         .with_init(Extension(LogName("my-client")))
///         .with(LoggingLayer)
///         .build();
///     let resp = client.get("https://truelayer.com").send().await.unwrap();
///     println!("TrueLayer page HTML: {}", resp.text().await.unwrap());
/// }
/// ```
pub struct Extension<T>(pub T);

impl<T: Send + Sync + Clone + 'static> RequestInitialiser for Extension<T> {
    fn init(&self, req: RequestBuilder, ext: &mut Extensions) -> RequestBuilder {
        ext.insert(self.0.clone());
        req
    }
}

use reqwest::RequestBuilder;
use task_local_extensions::Extensions;

/// When attached to a [`ClientWithMiddleware`] (generally using [`with_init`]), it is run
/// whenever the client starts building a request, in the order it was attached.
///
/// # Example
///
/// ```
/// use reqwest_middleware::{RequestInitialiser, MiddlewareRequest};
///
/// struct AuthInit;
///
/// impl RequestInitialiser for AuthInit {
///     fn init(&self, req: MiddlewareRequest) -> MiddlewareRequest {
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

impl RequestInitialiser for () {
    fn init(&self, req: RequestBuilder, _: &mut Extensions) -> RequestBuilder {
        req
    }
}

// impl<F> RequestInitialiser for F
// where
//     F: Send + Sync + 'static + Fn(MiddlewareRequest) -> MiddlewareRequest,
// {
//     fn init(&self, req: RequestBuilder, ext: &mut Extensions) -> RequestBuilder {
//         (self)(req)
//     }
// }

/// A middleware that inserts the value into the [`Extensions`](task_local_extensions::Extensions) during the call.
///
/// This is a good way to inject extensions to middleware deeper in the stack
///
/// ```
/// use reqwest::{Client, Request, Response};
/// use reqwest_middleware::{ClientBuilder, Middleware, Next, Result, Extension};
/// use task_local_extensions::Extensions;
///
/// #[derive(Clone)]
/// struct LogName(&'static str);
/// struct LoggingMiddleware;
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
///             .map(|&LogName(name)| name)
///             .unwrap_or("unknown");
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
///         .with_init(Extension(LogName("my-client")))
///         .with(LoggingMiddleware)
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

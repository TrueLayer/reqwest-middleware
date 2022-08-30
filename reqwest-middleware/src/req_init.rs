use crate::RequestBuilder;

pub trait RequestInitialiser: 'static + Send + Sync {
    fn init(&self, req: RequestBuilder) -> RequestBuilder;
}

impl<F> RequestInitialiser for F
where
    F: Send + Sync + 'static + Fn(RequestBuilder) -> RequestBuilder,
{
    fn init(&self, req: RequestBuilder) -> RequestBuilder {
        (self)(req)
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
    fn init(&self, req: RequestBuilder) -> RequestBuilder {
        req.with_extension(self.0.clone())
    }
}

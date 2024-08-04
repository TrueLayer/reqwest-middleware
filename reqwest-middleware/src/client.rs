use http::Extensions;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Body, Client, IntoUrl, Method, Request, Response};
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::sync::Arc;

#[cfg(feature = "multipart")]
use reqwest::multipart;

use crate::error::Result;
use crate::middleware::{Middleware, Next};
use crate::RequestInitialiser;

/// A `ClientBuilder` is used to build a [`ClientWithMiddleware`].
///
/// [`ClientWithMiddleware`]: crate::ClientWithMiddleware
pub struct ClientBuilder {
    client: Client,
    middleware_stack: Vec<Arc<dyn Middleware>>,
    initialiser_stack: Vec<Arc<dyn RequestInitialiser>>,
}

impl ClientBuilder {
    pub fn new(client: Client) -> Self {
        ClientBuilder {
            client,
            middleware_stack: Vec::new(),
            initialiser_stack: Vec::new(),
        }
    }

    /// This method allows creating a ClientBuilder
    /// from an existing ClientWithMiddleware instance
    pub fn from_client(client_with_middleware: ClientWithMiddleware) -> Self {
        Self {
            client: client_with_middleware.inner,
            middleware_stack: client_with_middleware.middleware_stack.into_vec(),
            initialiser_stack: client_with_middleware.initialiser_stack.into_vec(),
        }
    }

    /// Convenience method to attach middleware.
    ///
    /// If you need to keep a reference to the middleware after attaching, use [`with_arc`].
    ///
    /// [`with_arc`]: Self::with_arc
    pub fn with<M>(self, middleware: M) -> Self
    where
        M: Middleware,
    {
        self.with_arc(Arc::new(middleware))
    }

    /// Add middleware to the chain. [`with`] is more ergonomic if you don't need the `Arc`.
    ///
    /// [`with`]: Self::with
    pub fn with_arc(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.middleware_stack.push(middleware);
        self
    }

    /// Convenience method to attach a request initialiser.
    ///
    /// If you need to keep a reference to the initialiser after attaching, use [`with_arc_init`].
    ///
    /// [`with_arc_init`]: Self::with_arc_init
    pub fn with_init<I>(self, initialiser: I) -> Self
    where
        I: RequestInitialiser,
    {
        self.with_arc_init(Arc::new(initialiser))
    }

    /// Add a request initialiser to the chain. [`with_init`] is more ergonomic if you don't need the `Arc`.
    ///
    /// [`with_init`]: Self::with_init
    pub fn with_arc_init(mut self, initialiser: Arc<dyn RequestInitialiser>) -> Self {
        self.initialiser_stack.push(initialiser);
        self
    }

    /// Returns a `ClientWithMiddleware` using this builder configuration.
    pub fn build(self) -> ClientWithMiddleware {
        ClientWithMiddleware {
            inner: self.client,
            middleware_stack: self.middleware_stack.into_boxed_slice(),
            initialiser_stack: self.initialiser_stack.into_boxed_slice(),
        }
    }
}

/// `ClientWithMiddleware` is a wrapper around [`reqwest::Client`] which runs middleware on every
/// request.
#[derive(Clone, Default)]
pub struct ClientWithMiddleware {
    inner: reqwest::Client,
    middleware_stack: Box<[Arc<dyn Middleware>]>,
    initialiser_stack: Box<[Arc<dyn RequestInitialiser>]>,
}

impl ClientWithMiddleware {
    /// See [`ClientBuilder`] for a more ergonomic way to build `ClientWithMiddleware` instances.
    pub fn new<T>(client: Client, middleware_stack: T) -> Self
    where
        T: Into<Box<[Arc<dyn Middleware>]>>,
    {
        ClientWithMiddleware {
            inner: client,
            middleware_stack: middleware_stack.into(),
            // TODO(conradludgate) - allow downstream code to control this manually if desired
            initialiser_stack: Box::new([]),
        }
    }

    /// Convenience method to make a `GET` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    /// Convenience method to make a `POST` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    /// Convenience method to make a `PUT` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn put<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::PUT, url)
    }

    /// Convenience method to make a `PATCH` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn patch<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::PATCH, url)
    }

    /// Convenience method to make a `DELETE` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn delete<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::DELETE, url)
    }

    /// Convenience method to make a `HEAD` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn head<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::HEAD, url)
    }

    /// Start building a `Request` with the `Method` and `Url`.
    ///
    /// Returns a `RequestBuilder`, which will allow setting headers and
    /// the request body before sending.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let req = RequestBuilder {
            inner: self.inner.request(method, url),
            extensions: Extensions::new(),
            middleware_stack: self.middleware_stack.clone(),
            initialiser_stack: self.initialiser_stack.clone(),
        };
        self.initialiser_stack
            .iter()
            .fold(req, |req, i| i.init(req))
    }

    /// Executes a `Request`.
    ///
    /// A `Request` can be built manually with `Request::new()` or obtained
    /// from a RequestBuilder with `RequestBuilder::build()`.
    ///
    /// You should prefer to use the `RequestBuilder` and
    /// `RequestBuilder::send()`.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending request,
    /// redirect loop was detected or redirect limit was exhausted.
    pub async fn execute(&self, req: Request) -> Result<Response> {
        let mut ext = Extensions::new();
        self.execute_with_extensions(req, &mut ext).await
    }

    /// Executes a `Request` with initial [`Extensions`].
    ///
    /// A `Request` can be built manually with `Request::new()` or obtained
    /// from a RequestBuilder with `RequestBuilder::build()`.
    ///
    /// You should prefer to use the `RequestBuilder` and
    /// `RequestBuilder::send()`.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending request,
    /// redirect loop was detected or redirect limit was exhausted.
    pub async fn execute_with_extensions(
        &self,
        req: Request,
        ext: &mut Extensions,
    ) -> Result<Response> {
        let next = Next::new(&self.inner, &self.middleware_stack);
        next.run(req, ext).await
    }
}

/// Create a `ClientWithMiddleware` without any middleware.
impl From<Client> for ClientWithMiddleware {
    fn from(client: Client) -> Self {
        ClientWithMiddleware {
            inner: client,
            middleware_stack: Box::new([]),
            initialiser_stack: Box::new([]),
        }
    }
}

impl fmt::Debug for ClientWithMiddleware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // skipping middleware_stack field for now
        f.debug_struct("ClientWithMiddleware")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

mod service {
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };

    use crate::Result;
    use http::Extensions;
    use reqwest::{Request, Response};

    use crate::{middleware::BoxFuture, ClientWithMiddleware, Next};

    // this is meant to be semi-private, same as reqwest's pending
    pub struct Pending {
        inner: BoxFuture<'static, Result<Response>>,
    }

    impl Unpin for Pending {}

    impl Future for Pending {
        type Output = Result<Response>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.inner.as_mut().poll(cx)
        }
    }

    impl tower_service::Service<Request> for ClientWithMiddleware {
        type Response = Response;
        type Error = crate::Error;
        type Future = Pending;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
            self.inner.poll_ready(cx).map_err(crate::Error::Reqwest)
        }

        fn call(&mut self, req: Request) -> Self::Future {
            let inner = self.inner.clone();
            let middlewares = self.middleware_stack.clone();
            let mut extensions = Extensions::new();
            Pending {
                inner: Box::pin(async move {
                    let next = Next::new(&inner, &middlewares);
                    next.run(req, &mut extensions).await
                }),
            }
        }
    }

    impl tower_service::Service<Request> for &'_ ClientWithMiddleware {
        type Response = Response;
        type Error = crate::Error;
        type Future = Pending;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
            (&self.inner).poll_ready(cx).map_err(crate::Error::Reqwest)
        }

        fn call(&mut self, req: Request) -> Self::Future {
            let inner = self.inner.clone();
            let middlewares = self.middleware_stack.clone();
            let mut extensions = Extensions::new();
            Pending {
                inner: Box::pin(async move {
                    let next = Next::new(&inner, &middlewares);
                    next.run(req, &mut extensions).await
                }),
            }
        }
    }
}

/// This is a wrapper around [`reqwest::RequestBuilder`] exposing the same API.
#[must_use = "RequestBuilder does nothing until you 'send' it"]
pub struct RequestBuilder {
    inner: reqwest::RequestBuilder,
    middleware_stack: Box<[Arc<dyn Middleware>]>,
    initialiser_stack: Box<[Arc<dyn RequestInitialiser>]>,
    extensions: Extensions,
}

impl RequestBuilder {
    /// Assemble a builder starting from an existing `Client` and a `Request`.
    pub fn from_parts(client: ClientWithMiddleware, request: Request) -> RequestBuilder {
        let inner = reqwest::RequestBuilder::from_parts(client.inner, request);
        RequestBuilder {
            inner,
            middleware_stack: client.middleware_stack,
            initialiser_stack: client.initialiser_stack,
            extensions: Extensions::new(),
        }
    }

    /// Add a `Header` to this Request.
    pub fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        RequestBuilder {
            inner: self.inner.header(key, value),
            ..self
        }
    }

    /// Add a set of Headers to the existing ones on this Request.
    ///
    /// The headers will be merged in to any already set.
    pub fn headers(self, headers: HeaderMap) -> Self {
        RequestBuilder {
            inner: self.inner.headers(headers),
            ..self
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn version(self, version: reqwest::Version) -> Self {
        RequestBuilder {
            inner: self.inner.version(version),
            ..self
        }
    }

    /// Enable HTTP basic authentication.
    ///
    /// ```rust
    /// # use anyhow::Error;
    ///
    /// # async fn run() -> Result<(), Error> {
    /// let client = reqwest_middleware::ClientWithMiddleware::from(reqwest::Client::new());
    /// let resp = client.delete("http://httpbin.org/delete")
    ///     .basic_auth("admin", Some("good password"))
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: Display,
        P: Display,
    {
        RequestBuilder {
            inner: self.inner.basic_auth(username, password),
            ..self
        }
    }

    /// Enable HTTP bearer authentication.
    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: Display,
    {
        RequestBuilder {
            inner: self.inner.bearer_auth(token),
            ..self
        }
    }

    /// Set the request body.
    pub fn body<T: Into<Body>>(self, body: T) -> Self {
        RequestBuilder {
            inner: self.inner.body(body),
            ..self
        }
    }

    /// Enables a request timeout.
    ///
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished. It affects only this request and overrides
    /// the timeout configured using `ClientBuilder::timeout()`.
    pub fn timeout(self, timeout: std::time::Duration) -> Self {
        RequestBuilder {
            inner: self.inner.timeout(timeout),
            ..self
        }
    }

    #[cfg(feature = "multipart")]
    #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
    pub fn multipart(self, multipart: multipart::Form) -> Self {
        RequestBuilder {
            inner: self.inner.multipart(multipart),
            ..self
        }
    }

    /// Modify the query string of the URL.
    ///
    /// Modifies the URL of this request, adding the parameters provided.
    /// This method appends and does not overwrite. This means that it can
    /// be called multiple times and that existing query parameters are not
    /// overwritten if the same key is used. The key will simply show up
    /// twice in the query string.
    /// Calling `.query(&[("foo", "a"), ("foo", "b")])` gives `"foo=a&foo=b"`.
    ///
    /// # Note
    /// This method does not support serializing a single key-value
    /// pair. Instead of using `.query(("key", "val"))`, use a sequence, such
    /// as `.query(&[("key", "val")])`. It's also possible to serialize structs
    /// and maps into a key-value pair.
    ///
    /// # Errors
    /// This method will fail if the object you provide cannot be serialized
    /// into a query string.
    pub fn query<T: Serialize + ?Sized>(self, query: &T) -> Self {
        RequestBuilder {
            inner: self.inner.query(query),
            ..self
        }
    }

    /// Send a form body.
    ///
    /// Sets the body to the url encoded serialization of the passed value,
    /// and also sets the `Content-Type: application/x-www-form-urlencoded`
    /// header.
    ///
    /// ```rust
    /// # use anyhow::Error;
    /// # use std::collections::HashMap;
    /// #
    /// # async fn run() -> Result<(), Error> {
    /// let mut params = HashMap::new();
    /// params.insert("lang", "rust");
    ///
    /// let client = reqwest_middleware::ClientWithMiddleware::from(reqwest::Client::new());
    /// let res = client.post("http://httpbin.org")
    ///     .form(&params)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This method fails if the passed value cannot be serialized into
    /// url encoded format
    pub fn form<T: Serialize + ?Sized>(self, form: &T) -> Self {
        RequestBuilder {
            inner: self.inner.form(form),
            ..self
        }
    }

    /// Send a JSON body.
    ///
    /// # Optional
    ///
    /// This requires the optional `json` feature enabled.
    ///
    /// # Errors
    ///
    /// Serialization can fail if `T`'s implementation of `Serialize` decides to
    /// fail, or if `T` contains a map with non-string keys.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn json<T: Serialize + ?Sized>(self, json: &T) -> Self {
        RequestBuilder {
            inner: self.inner.json(json),
            ..self
        }
    }

    /// Disable CORS on fetching the request.
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request mode][mdn] will be set to 'no-cors'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/mode
    pub fn fetch_mode_no_cors(self) -> Self {
        RequestBuilder {
            inner: self.inner.fetch_mode_no_cors(),
            ..self
        }
    }

    /// Build a `Request`, which can be inspected, modified and executed with
    /// `ClientWithMiddleware::execute()`.
    pub fn build(self) -> reqwest::Result<Request> {
        self.inner.build()
    }

    /// Build a `Request`, which can be inspected, modified and executed with
    /// `ClientWithMiddleware::execute()`.
    ///
    /// This is similar to [`RequestBuilder::build()`], but also returns the
    /// embedded `Client`.
    pub fn build_split(self) -> (ClientWithMiddleware, reqwest::Result<Request>) {
        let Self {
            inner,
            middleware_stack,
            initialiser_stack,
            ..
        } = self;
        let (inner, req) = inner.build_split();
        let client = ClientWithMiddleware {
            inner,
            middleware_stack,
            initialiser_stack,
        };
        (client, req)
    }

    /// Inserts the extension into this request builder
    pub fn with_extension<T: Send + Sync + Clone + 'static>(mut self, extension: T) -> Self {
        self.extensions.insert(extension);
        self
    }

    /// Returns a mutable reference to the internal set of extensions for this request
    pub fn extensions(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    /// Constructs the Request and sends it to the target URL, returning a
    /// future Response.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending request,
    /// redirect loop was detected or redirect limit was exhausted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Error;
    /// #
    /// # async fn run() -> Result<(), Error> {
    /// let response = reqwest_middleware::ClientWithMiddleware::from(reqwest::Client::new())
    ///     .get("https://hyper.rs")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(mut self) -> Result<Response> {
        let mut extensions = std::mem::take(self.extensions());
        let (client, req) = self.build_split();
        client.execute_with_extensions(req?, &mut extensions).await
    }

    /// Attempt to clone the RequestBuilder.
    ///
    /// `None` is returned if the RequestBuilder can not be cloned,
    /// i.e. if the request body is a stream.
    ///
    /// # Examples
    ///
    /// ```
    /// # use reqwest::Error;
    /// #
    /// # fn run() -> Result<(), Error> {
    /// let client = reqwest_middleware::ClientWithMiddleware::from(reqwest::Client::new());
    /// let builder = client.post("http://httpbin.org/post")
    ///     .body("from a &str!");
    /// let clone = builder.try_clone();
    /// assert!(clone.is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_clone(&self) -> Option<Self> {
        self.inner.try_clone().map(|inner| RequestBuilder {
            inner,
            middleware_stack: self.middleware_stack.clone(),
            initialiser_stack: self.initialiser_stack.clone(),
            extensions: self.extensions.clone(),
        })
    }
}

impl fmt::Debug for RequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // skipping middleware_stack field for now
        f.debug_struct("RequestBuilder")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

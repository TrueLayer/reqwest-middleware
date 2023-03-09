use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::multipart::Form;
use reqwest::{Body, Client, IntoUrl, Method, Request, Response};
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::sync::Arc;
use task_local_extensions::Extensions;

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
#[derive(Clone)]
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

    /// See [`Client::get`]
    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    /// See [`Client::post`]
    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    /// See [`Client::put`]
    pub fn put<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::PUT, url)
    }

    /// See [`Client::patch`]
    pub fn patch<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::PATCH, url)
    }

    /// See [`Client::delete`]
    pub fn delete<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::DELETE, url)
    }

    /// See [`Client::head`]
    pub fn head<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::HEAD, url)
    }

    /// See [`Client::request`]
    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let req = RequestBuilder {
            inner: self.inner.request(method, url),
            client: self.clone(),
            extensions: Extensions::new(),
        };
        self.initialiser_stack
            .iter()
            .fold(req, |req, i| i.init(req))
    }

    /// See [`Client::execute`]
    pub async fn execute(&self, req: Request) -> Result<Response> {
        let mut ext = Extensions::new();
        self.execute_with_extensions(req, &mut ext).await
    }

    /// Executes a request with initial [`Extensions`].
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

/// This is a wrapper around [`reqwest::RequestBuilder`] exposing the same API.
#[must_use = "RequestBuilder does nothing until you 'send' it"]
pub struct RequestBuilder {
    inner: reqwest::RequestBuilder,
    client: ClientWithMiddleware,
    extensions: Extensions,
}

impl RequestBuilder {
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

    pub fn headers(self, headers: HeaderMap) -> Self {
        RequestBuilder {
            inner: self.inner.headers(headers),
            ..self
        }
    }

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

    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: Display,
    {
        RequestBuilder {
            inner: self.inner.bearer_auth(token),
            ..self
        }
    }

    pub fn body<T: Into<Body>>(self, body: T) -> Self {
        RequestBuilder {
            inner: self.inner.body(body),
            ..self
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn timeout(self, timeout: std::time::Duration) -> Self {
        RequestBuilder {
            inner: self.inner.timeout(timeout),
            ..self
        }
    }

    pub fn multipart(self, multipart: Form) -> Self {
        RequestBuilder {
            inner: self.inner.multipart(multipart),
            ..self
        }
    }

    pub fn query<T: Serialize + ?Sized>(self, query: &T) -> Self {
        RequestBuilder {
            inner: self.inner.query(query),
            ..self
        }
    }

    pub fn form<T: Serialize + ?Sized>(self, form: &T) -> Self {
        RequestBuilder {
            inner: self.inner.form(form),
            ..self
        }
    }

    pub fn json<T: Serialize + ?Sized>(self, json: &T) -> Self {
        RequestBuilder {
            inner: self.inner.json(json),
            ..self
        }
    }

    pub fn build(self) -> reqwest::Result<Request> {
        self.inner.build()
    }

    /// Inserts the extension into this request builder
    pub fn with_extension<T: Send + Sync + 'static>(mut self, extension: T) -> Self {
        self.extensions.insert(extension);
        self
    }

    /// Returns a mutable reference to the internal set of extensions for this request
    pub fn extensions(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    pub async fn send(self) -> Result<Response> {
        let Self {
            inner,
            client,
            mut extensions,
        } = self;
        let req = inner.build()?;
        client.execute_with_extensions(req, &mut extensions).await
    }

    /// Attempt to clone the RequestBuilder.
    ///
    /// `None` is returned if the RequestBuilder can not be cloned,
    /// i.e. if the request body is a stream.
    ///
    /// # Extensions
    /// Note that extensions are not preserved through cloning.
    pub fn try_clone(&self) -> Option<Self> {
        self.inner.try_clone().map(|inner| RequestBuilder {
            inner,
            client: self.client.clone(),
            extensions: Extensions::new(),
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

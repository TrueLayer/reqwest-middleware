use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::multipart::Form;
use reqwest::{Body, Client, IntoUrl, Method, Request, Response};
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;
use task_local_extensions::Extensions;

use crate::error::Result;
use crate::middleware::{Middleware, Next};

/// A `ClientBuilder` is used to build a [`ClientWithMiddleware`].
///
/// [`ClientWithMiddleware`]: crate::ClientWithMiddleware
pub struct ClientBuilder {
    client: Client,
    middleware_stack: Vec<Arc<dyn Middleware>>,
}

impl ClientBuilder {
    pub fn new(client: Client) -> Self {
        ClientBuilder {
            client,
            middleware_stack: Vec::new(),
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

    /// Returns a `ClientWithMiddleware` using this builder configuration.
    pub fn build(self) -> ClientWithMiddleware {
        ClientWithMiddleware::new(self.client, self.middleware_stack)
    }
}

/// `ClientWithMiddleware` is a wrapper around [`reqwest::Client`] which runs middleware on every
/// request.
#[derive(Clone, Debug)]
pub struct ClientWithMiddleware {
    inner: reqwest::Client,
    middleware_stack: Box<[Arc<dyn Middleware>]>,
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
        RequestBuilder {
            inner: self.inner.request(method, url),
            client: self.clone(),
        }
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
        }
    }
}

/// This is a wrapper around [`reqwest::RequestBuilder`] exposing the same API.
#[must_use = "RequestBuilder does nothing until you 'send' it"]
pub struct RequestBuilder {
    inner: reqwest::RequestBuilder,
    client: ClientWithMiddleware,
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
            client: self.client,
        }
    }

    pub fn headers(self, headers: HeaderMap) -> Self {
        RequestBuilder {
            inner: self.inner.headers(headers),
            client: self.client,
        }
    }

    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: Display,
        P: Display,
    {
        RequestBuilder {
            inner: self.inner.basic_auth(username, password),
            client: self.client,
        }
    }

    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: Display,
    {
        RequestBuilder {
            inner: self.inner.bearer_auth(token),
            client: self.client,
        }
    }

    pub fn body<T: Into<Body>>(self, body: T) -> Self {
        RequestBuilder {
            inner: self.inner.body(body),
            client: self.client,
        }
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        RequestBuilder {
            inner: self.inner.timeout(timeout),
            client: self.client,
        }
    }

    pub fn multipart(self, multipart: Form) -> Self {
        RequestBuilder {
            inner: self.inner.multipart(multipart),
            client: self.client,
        }
    }

    pub fn query<T: Serialize + ?Sized>(self, query: &T) -> Self {
        RequestBuilder {
            inner: self.inner.query(query),
            client: self.client,
        }
    }

    pub fn form<T: Serialize + ?Sized>(self, form: &T) -> Self {
        RequestBuilder {
            inner: self.inner.form(form),
            client: self.client,
        }
    }

    pub fn json<T: Serialize + ?Sized>(self, json: &T) -> Self {
        RequestBuilder {
            inner: self.inner.json(json),
            client: self.client,
        }
    }

    pub fn build(self) -> reqwest::Result<Request> {
        self.inner.build()
    }

    pub async fn send(self) -> Result<Response> {
        let req = self.inner.build()?;
        self.client.execute(req).await
    }

    /// Sends a request with initial [`Extensions`].
    pub async fn send_with_extensions(self, ext: &mut Extensions) -> Result<Response> {
        let req = self.inner.build()?;
        self.client.execute_with_extensions(req, ext).await
    }

    pub fn try_clone(self) -> Option<Self> {
        let client = self.client;
        self.inner
            .try_clone()
            .map(|inner| RequestBuilder { inner, client })
    }
}

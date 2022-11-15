//! This crate provides [`ClientWithMiddleware`], a wrapper around [`reqwest::Client`] with the
//! ability to attach middleware which runs on every request.
//!
//! You'll want to instantiate [`ClientWithMiddleware`] using [`ClientBuilder`], then you can
//! attach your middleware using [`with`], finalize it with [`build`] and from then on sending
//! requests is the same as with reqwest:
//!
//! ```
//! use reqwest::{Client, Request, Response};
//! use reqwest_middleware::{ClientBuilder, Error, Extension, Layer, Service};
//! use task_local_extensions::Extensions;
//! use futures::future::{BoxFuture, FutureExt};
//! use std::task::{Context, Poll};
//!
//! struct LoggingLayer;
//! struct LoggingService<S>(S);
//!
//! impl<S> Layer<S> for LoggingLayer {
//!     type Service = LoggingService<S>;
//!
//!     fn layer(&self, inner: S) -> Self::Service {
//!         LoggingService(inner)
//!     }
//! }
//!
//! impl<S> Service for LoggingService<S>
//! where
//!     S: Service,
//!     S::Future: Send + 'static,
//! {
//!     type Future = BoxFuture<'static, Result<Response, Error>>;
//!     
//!     fn call(&mut self, req: Request, ext: &mut Extensions) -> Self::Future {
//!         println!("Request started {req:?}");
//!         let fut = self.0.call(req, ext);
//!         async {
//!             let res = fut.await;
//!             println!("Result: {res:?}");
//!             res
//!         }.boxed()
//!     }
//! }
//!
//! async fn run() {
//!     let reqwest_client = Client::builder().build().unwrap();
//!     let client = ClientBuilder::new(reqwest_client)
//!         .with(LoggingLayer)
//!         .build();
//!     let resp = client.get("https://truelayer.com").send().await.unwrap();
//!     println!("TrueLayer page HTML: {}", resp.text().await.unwrap());
//! }
//! ```
//!
//! [`build`]: ClientBuilder::build
//! [`ClientBuilder`]: ClientBuilder
//! [`ClientWithMiddleware`]: ClientWithMiddleware
//! [`with`]: ClientBuilder::with

// Test README examples without overriding module docs.
// We want to keep the in-code docs separate as those allow for automatic linking to crate
// documentation.
#[doc = include_str!("../../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

mod client;
mod error;
mod req_init;

pub use client::{ClientBuilder, ClientWithMiddleware, RequestBuilder, ReqwestService};
pub use error::Error;
pub use req_init::{Extension, RequestInitialiser, RequestStack};
use reqwest::{Request, Response};
use task_local_extensions::Extensions;

/// Two [`RequestInitialiser`]s or [`Service`]s chained together.
#[derive(Clone)]
pub struct Stack<Inner, Outer> {
    pub(crate) inner: Inner,
    pub(crate) outer: Outer,
}

pub trait Service {
    type Future: std::future::Future<Output = Result<Response, Error>>;
    fn call(&mut self, req: Request, ext: &mut Extensions) -> Self::Future;
}

pub struct Identity;

impl<S: Service> Layer<S> for Identity {
    type Service = S;

    fn layer(&self, inner: S) -> Self::Service {
        inner
    }
}

pub trait Layer<S> {
    /// The wrapped service
    type Service;
    /// Wrap the given service with the middleware, returning a new service
    /// that has been decorated with the middleware.
    fn layer(&self, inner: S) -> Self::Service;
}

impl<S, Inner, Outer> Layer<S> for Stack<Inner, Outer>
where
    Inner: Layer<S>,
    Outer: Layer<Inner::Service>,
{
    type Service = Outer::Service;

    fn layer(&self, service: S) -> Self::Service {
        let inner = self.inner.layer(service);

        self.outer.layer(inner)
    }
}

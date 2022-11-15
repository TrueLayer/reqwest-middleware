//! This crate provides [`ClientWithMiddleware`], a wrapper around [`reqwest::Client`] with the
//! ability to attach middleware which runs on every request.
//!
//! You'll want to instantiate [`ClientWithMiddleware`] using [`ClientBuilder`], then you can
//! attach your middleware using [`with`], finalize it with [`build`] and from then on sending
//! requests is the same as with reqwest:
//!
//! ```
//! use reqwest::{Client, Request, Response};
//! use reqwest_middleware::{ClientBuilder, Middleware, Next, Result};
//! use task_local_extensions::Extensions;
//! use futures::FutureExt;
//! use std::task::{Context, Poll};
//!
//! struct LoggingLayer;
//! struct LoggingService<S>(S);
//! 
//! impl<S> tower::Layer<S> for LoggingLayer {
//!     type Service = LoggingService<S>;
//! 
//!     fn layer(&self, inner: S) -> Self::Service {
//!         LoggingService(inner)
//!     }
//! }
//!
//! impl<S: tower::Service<MiddlewareRequest>> tower::Service<MiddlewareRequest> for LoggingService<S> {
//!     type Response = S::Response;
//!     type Error = S::Error;
//!     type Future = futures::BoxFuture<'static, Result<S::Response, S::Error>>;
//! 
//!     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//!         self.0.poll_ready(cx)
//!     }
//!     
//!     fn call(&mut self, req: MiddlewareRequest) -> Self::Future {
//!         println!("Request started {:?}", &req.request);
//!         let fut = self.0.call(req);
//!         async {
//!             let res = fut.await;
//!             println!("Result: {:?}", res);
//!             res
//!         }.boxed()
//!     }
//! }
//!
//! async fn run() {
//!     let reqwest_client = Client::builder().build().unwrap();
//!     let client = ClientBuilder::new(reqwest_client)
//!         .layer(LoggingLayer)
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

pub use client::{ClientBuilder, ClientWithMiddleware, ReqService, RequestBuilder};
pub use error::{Error, Result};
pub use req_init::{Extension, RequestInitialiser};

pub struct MiddlewareRequest {
    pub request: reqwest::Request,
    pub extensions: task_local_extensions::Extensions,
}

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
//!
//! struct LoggingMiddleware;
//!
//! #[async_trait::async_trait]
//! impl Middleware for LoggingMiddleware {
//!     async fn handle(
//!         &self,
//!         req: Request,
//!         extensions: &mut Extensions,
//!         next: Next<'_>,
//!     ) -> Result<Response> {
//!         println!("Request started {:?}", req);
//!         let res = next.run(req, extensions).await;
//!         println!("Result: {:?}", res);
//!         res
//!     }
//! }
//!
//! async fn run() {
//!     let reqwest_client = Client::builder().build().unwrap();
//!     let client = ClientBuilder::new(reqwest_client)
//!         .with(LoggingMiddleware)
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
mod middleware;
mod req_init;

pub use client::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
pub use error::{Error, Result};
pub use middleware::{Middleware, Next};
pub use req_init::{Extension, RequestInitialiser};

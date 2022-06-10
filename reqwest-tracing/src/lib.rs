//! Opentracing middleware implementation for [`reqwest-middleware`].
//!
//! Attach [`TracingMiddleware`] to your client to automatically trace HTTP requests.
//!
//! In this example we define a custom span builder to calculate the request time elapsed and we register the `TracingMiddleware`.
//! ```rust
//!
//! use reqwest_middleware::Result;
//! use task_local_extensions::Extensions;
//! use reqwest::{Request, Response};
//! use reqwest_middleware::ClientBuilder;
//! use reqwest_tracing::{DefaultRequestOtelSpanBuilder, RequestOtelSpanBuilder, TracingMiddleware};
//! use tracing::Span;
//! use std::time::{Duration, Instant};
//!
//! use reqwest_tracing::root_span;
//!
//! pub struct TimeTrace;
//!
//! impl RequestOtelSpanBuilder for TimeTrace {
//!     fn on_request_start(req: &Request, extension: &mut Extensions) -> Span {
//!         extension.insert(Instant::now());
//!         root_span!(req, time_elapsed = tracing::field::Empty)
//!     }
//!
//!     fn on_request_end(span: &Span, outcome: &Result<Response>, extension: &mut Extensions) {
//!         let time_elapsed = extension.get::<Instant>().unwrap().elapsed().as_millis() as i64;
//!         DefaultRequestOtelSpanBuilder::on_request_end(span, outcome, extension);
//!         span.record("time_elapsed", &time_elapsed);
//!     }
//! }
//!
//! let http = ClientBuilder::new(reqwest::Client::new())
//!     .with(TracingMiddleware::<TimeTrace>::default())
//!     .build();
//! ```

mod middleware;
#[cfg(any(
    feature = "opentelemetry_0_13",
    feature = "opentelemetry_0_14",
    feature = "opentelemetry_0_15",
    feature = "opentelemetry_0_16",
    feature = "opentelemetry_0_17",
))]
mod otel;
mod reqwest_otel_span_builder;
pub use middleware::TracingMiddleware;
pub use reqwest_otel_span_builder::{DefaultRequestOtelSpanBuilder, RequestOtelSpanBuilder};

#[doc(hidden)]
pub mod reqwest_otel_span_macro;

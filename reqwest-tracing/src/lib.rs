//! Opentracing middleware implementation for [`reqwest_middleware`].
//!
//! Attach [`TracingMiddleware`] to your client to automatically trace HTTP requests.
//!
//! In this example we define a custom span builder to calculate the request time elapsed and we register the [`TracingMiddleware`].
//!
//! Note that Opentelemetry tracks start and stop already, there is no need to have a custom builder like this.
//! ```rust
//! use reqwest_middleware::Result;
//! use task_local_extensions::Extensions;
//! use reqwest::{Request, Response};
//! use reqwest_middleware::ClientBuilder;
//! use reqwest_tracing::{
//!     default_on_request_end, reqwest_otel_span, ReqwestOtelSpanBackend, TracingMiddleware
//! };
//! use tracing::Span;
//! use std::time::{Duration, Instant};
//!
//! pub struct TimeTrace;
//!
//! impl ReqwestOtelSpanBackend for TimeTrace {
//!     fn on_request_start(req: &Request, extension: &mut Extensions) -> Span {
//!         extension.insert(Instant::now());
//!         reqwest_otel_span!(name="example-request", req, time_elapsed = tracing::field::Empty)
//!     }
//!
//!     fn on_request_end(span: &Span, outcome: &Result<Response>, extension: &mut Extensions) {
//!         let time_elapsed = extension.get::<Instant>().unwrap().elapsed().as_millis() as i64;
//!         default_on_request_end(span, outcome);
//!         span.record("time_elapsed", &time_elapsed);
//!     }
//! }
//!
//! let http = ClientBuilder::new(reqwest::Client::new())
//!     .with(TracingMiddleware::<TimeTrace>::new())
//!     .build();
//! ```

mod middleware;
#[cfg(any(
    feature = "opentelemetry_0_13",
    feature = "opentelemetry_0_14",
    feature = "opentelemetry_0_15",
    feature = "opentelemetry_0_16",
    feature = "opentelemetry_0_17",
    feature = "opentelemetry_0_18",
))]
mod otel;
mod reqwest_otel_span_builder;
pub use middleware::TracingMiddleware;
pub use reqwest_otel_span_builder::{
    default_on_request_end, default_on_request_failure, default_on_request_success,
    DefaultSpanBackend, ReqwestOtelSpanBackend, ERROR_CAUSE_CHAIN, ERROR_MESSAGE, HTTP_HOST,
    HTTP_METHOD, HTTP_SCHEME, HTTP_STATUS_CODE, HTTP_URL, HTTP_USER_AGENT, NET_HOST_PORT,
    OTEL_KIND, OTEL_NAME, OTEL_STATUS_CODE,
};

#[doc(hidden)]
pub mod reqwest_otel_span_macro;

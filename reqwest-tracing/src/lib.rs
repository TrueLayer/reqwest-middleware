//! Opentracing middleware implementation for [`reqwest-middleware`].
//!
//! Attach [`TracingMiddleware`] to your client to automatically trace HTTP requests.

mod middleware;
#[cfg(any(
    feature = "opentelemetry_0_13",
    feature = "opentelemetry_0_14",
    feature = "opentelemetry_0_15",
    feature = "opentelemetry_0_16",
))]
mod otel;

pub use middleware::TracingMiddleware;

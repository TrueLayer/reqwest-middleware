use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Request;
use std::str::FromStr;
use tracing::Span;

#[cfg(feature = "opentelemetry_0_13")]
use opentelemetry_0_13_pkg as opentelemetry;

#[cfg(feature = "opentelemetry_0_14")]
use opentelemetry_0_14_pkg as opentelemetry;

#[cfg(feature = "opentelemetry_0_15")]
use opentelemetry_0_15_pkg as opentelemetry;

#[cfg(feature = "opentelemetry_0_16")]
use opentelemetry_0_16_pkg as opentelemetry;

#[cfg(feature = "opentelemetry_0_13")]
pub use tracing_opentelemetry_0_12_pkg as tracing_opentelemetry;

#[cfg(feature = "opentelemetry_0_14")]
pub use tracing_opentelemetry_0_13_pkg as tracing_opentelemetry;

#[cfg(feature = "opentelemetry_0_15")]
pub use tracing_opentelemetry_0_14_pkg as tracing_opentelemetry;

#[cfg(feature = "opentelemetry_0_16")]
pub use tracing_opentelemetry_0_16_pkg as tracing_opentelemetry;

use opentelemetry::global;
use opentelemetry::propagation::Injector;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Injects the given Opentelemetry Context into a reqwest::Request headers to allow propagation downstream.
pub fn inject_opentracing_context_into_request(span: &Span, request: Request) -> Request {
    let context = span.context();
    let mut request = request;

    global::get_text_map_propagator(|injector| {
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    request
}

// "traceparent" => https://www.w3.org/TR/trace-context/#trace-context-http-headers-format

/// Injector used via opentelemetry propagator to tell the extractor how to insert the "traceparent" header value
/// This will allow the propagator to inject opentelemetry context into a standard data structure. Will basically
/// insert a "traceparent" string value "{version}-{trace_id}-{span_id}-{trace-flags}" of the spans context into the headers.
/// Listeners can then re-hydrate the context to add additional spans to the same trace.
struct RequestCarrier<'a> {
    request: &'a mut Request,
}

impl<'a> RequestCarrier<'a> {
    pub fn new(request: &'a mut Request) -> Self {
        RequestCarrier { request }
    }
}

impl<'a> Injector for RequestCarrier<'a> {
    fn set(&mut self, key: &str, value: String) {
        let header_name = HeaderName::from_str(key).expect("Must be header name");
        let header_value = HeaderValue::from_str(&value).expect("Must be a header value");
        self.request.headers_mut().insert(header_name, header_value);
    }
}

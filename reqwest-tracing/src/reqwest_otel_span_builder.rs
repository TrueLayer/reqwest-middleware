use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Request, Response, StatusCode as RequestStatusCode};
use reqwest_middleware::{Error, Result};
use task_local_extensions::Extensions;
use tracing::Span;

use crate::reqwest_otel_span;

/// The `http.method` field added to the span by [`reqwest_otel_span`]
pub const HTTP_METHOD: &str = "http.method";
/// The `http.scheme` field added to the span by [`reqwest_otel_span`]
pub const HTTP_SCHEME: &str = "http.scheme";
/// The `http.host` field added to the span by [`reqwest_otel_span`]
pub const HTTP_HOST: &str = "http.host";
/// The `http.url` field added to the span by [`reqwest_otel_span`]
pub const HTTP_URL: &str = "http.url";
/// The `host.port` field added to the span by [`reqwest_otel_span`]
pub const NET_HOST_PORT: &str = "net.host.port";
/// The `otel.kind` field added to the span by [`reqwest_otel_span`]
pub const OTEL_KIND: &str = "otel.kind";
/// The `otel.name` field added to the span by [`reqwest_otel_span`]
pub const OTEL_NAME: &str = "otel.name";
/// The `otel.status_code.code` field added to the span by [`reqwest_otel_span`]
pub const OTEL_STATUS_CODE: &str = "http.status_code";
/// The `error.message` field added to the span by [`reqwest_otel_span`]
pub const ERROR_MESSAGE: &str = "error.message";
/// The `error.cause_chain` field added to the span by [`reqwest_otel_span`]
pub const ERROR_CAUSE_CHAIN: &str = "error.cause_chain";
/// The `http.status_code` field added to the span by [`reqwest_otel_span`]
pub const HTTP_STATUS_CODE: &str = "http.status_code";
/// The `http.user_agent` added to the span by [`reqwest_otel_span`]
pub const HTTP_USER_AGENT: &str = "http.user_agent";

/// [`ReqwestOtelSpanBackend`] allows you to customise the span attached by
/// [`TracingMiddleware`] to incoming requests.
///
/// Check out [`reqwest_otel_span`] documentation for examples.
///
/// [`TracingMiddleware`]: crate::middleware::TracingMiddleware.
pub trait ReqwestOtelSpanBackend {
    /// Initalized a new span before the request is executed.
    fn on_request_start(req: &Request, extension: &mut Extensions) -> Span;

    /// Runs after the request call has executed.
    fn on_request_end(span: &Span, outcome: &Result<Response>, extension: &mut Extensions);
}

/// Populates default success/failure fields for a given [`reqwest_otel_span!`] span.
#[inline]
pub fn default_on_request_end(span: &Span, outcome: &Result<Response>) {
    match outcome {
        Ok(res) => default_on_request_success(span, res),
        Err(err) => default_on_request_failure(span, err),
    }
}

/// Populates default success fields for a given [`reqwest_otel_span!`] span.
#[inline]
pub fn default_on_request_success(span: &Span, response: &Response) {
    let span_status = get_span_status(response.status());
    let status_code = response.status().as_u16() as i64;
    let user_agent = get_header_value("user_agent", response.headers());
    if let Some(span_status) = span_status {
        span.record(OTEL_STATUS_CODE, span_status);
    }
    span.record(HTTP_STATUS_CODE, status_code);
    span.record(HTTP_USER_AGENT, user_agent.as_str());
}

/// Populates default failure fields for a given [`reqwest_otel_span!`] span.
#[inline]
pub fn default_on_request_failure(span: &Span, e: &Error) {
    let error_message = e.to_string();
    let error_cause_chain = format!("{:?}", e);
    span.record(OTEL_STATUS_CODE, "ERROR");
    span.record(ERROR_MESSAGE, error_message.as_str());
    span.record(ERROR_CAUSE_CHAIN, error_cause_chain.as_str());
    if let Error::Reqwest(e) = e {
        span.record(
            HTTP_STATUS_CODE,
            e.status()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string())
                .as_str(),
        );
    }
}

/// The default [`ReqwestOtelSpanBackend`] for [`TracingMiddleware`].
///
/// [`TracingMiddleware`]: crate::middleware::TracingMiddleware
pub struct DefaultSpanBackend;

impl ReqwestOtelSpanBackend for DefaultSpanBackend {
    fn on_request_start(req: &Request, _: &mut Extensions) -> Span {
        reqwest_otel_span!(name = "reqwest-http-client", req)
    }

    fn on_request_end(span: &Span, outcome: &Result<Response>, _: &mut Extensions) {
        default_on_request_end(span, outcome)
    }
}

fn get_header_value(key: &str, headers: &HeaderMap) -> String {
    let header_default = &HeaderValue::from_static("");
    format!("{:?}", headers.get(key).unwrap_or(header_default)).replace('"', "")
}

/// HTTP Mapping <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/http.md#status>
///
/// Maps the the http status to an Opentelemetry span status following the the specified convention above.
fn get_span_status(request_status: RequestStatusCode) -> Option<&'static str> {
    match request_status.as_u16() {
        // Span Status MUST be left unset if HTTP status code was in the 1xx, 2xx or 3xx ranges, unless there was
        // another error (e.g., network error receiving the response body; or 3xx codes with max redirects exceeded),
        // in which case status MUST be set to Error.
        100..=399 => None,
        // For HTTP status codes in the 4xx range span status MUST be left unset in case of SpanKind.SERVER and MUST be
        // set to Error in case of SpanKind.CLIENT.
        400..=499 => Some("ERROR"),
        // For HTTP status codes in the 5xx range, as well as any other code the client failed to interpret, span
        // status MUST be set to Error.
        _ => Some("ERROR"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_header_value_for_span_attribute() {
        let expect = "IMPORTANT_HEADER";
        let mut header_map = HeaderMap::new();
        header_map.insert("test", expect.parse().unwrap());

        let value = get_header_value("test", &header_map);
        assert_eq!(value, expect);
    }
}

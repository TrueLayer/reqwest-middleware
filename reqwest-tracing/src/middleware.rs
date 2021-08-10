use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Request, Response, StatusCode as RequestStatusCode};
use reqwest_middleware::{Error, Middleware, Next, Result};
use truelayer_extensions::Extensions;

/// Middleware for tracing requests using the current Opentelemetry Context.
pub struct TracingMiddleware;

#[async_trait::async_trait]
impl Middleware for TracingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let request_span = {
            let method = req.method();
            let scheme = req.url().scheme();
            let host = req.url().host_str().unwrap_or("");
            let host_port = req.url().port().unwrap_or(0) as i64;
            let path = req.url().path();
            let otel_name = format!("{} {}", method, path);

            tracing::info_span!(
                "HTTP request",
                http.method = %method,
                http.scheme = %scheme,
                http.host = %host,
                net.host.port = %host_port,
                otel.kind = "client",
                otel.name = %otel_name,
                otel.status_code = tracing::field::Empty,
                http.user_agent = tracing::field::Empty,
                http.status_code = tracing::field::Empty,
                error.message = tracing::field::Empty,
                error.cause_chain = tracing::field::Empty,
            )
        };

        // Adds tracing headers to the given request to propagate the OpenTracing context to downstream revivers of the request.
        // Spans added by downstream consumers will be part of the same trace.
        #[cfg(any(
            feature = "opentelemetry_0_13",
            feature = "opentelemetry_0_14",
            feature = "opentelemetry_0_15"
        ))]
        let req = crate::otel::inject_opentracing_context_into_request(&request_span, req);

        // Run the request
        let outcome = next.run(req, extensions).await;
        match &outcome {
            Ok(response) => {
                // The request ran successfully
                let span_status = get_span_status(response.status());
                let status_code = response.status().as_u16() as i64;
                let user_agent = get_header_value("user_agent", response.headers());
                if let Some(span_status) = span_status {
                    request_span.record("otel.status_code", &span_status);
                }
                request_span.record("http.status_code", &status_code);
                request_span.record("http.user_agent", &user_agent.as_str());
            }
            Err(e) => {
                // The request didn't run successfully
                let error_message = e.to_string();
                let error_cause_chain = format!("{:?}", e);
                request_span.record("otel.status_code", &"ERROR");
                request_span.record("error.message", &error_message.as_str());
                request_span.record("error.cause_chain", &error_cause_chain.as_str());
                if let Error::Reqwest(e) = e {
                    request_span.record(
                        "http.status_code",
                        &e.status()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "".to_string())
                            .as_str(),
                    );
                }
            }
        }
        outcome
    }
}

fn get_header_value(key: &str, headers: &HeaderMap) -> String {
    let header_default = &HeaderValue::from_static("");
    format!("{:?}", headers.get(key).unwrap_or(header_default)).replace("\"", "")
}

/// HTTP Mapping <https://github.com/open-telemetry/opentelemetry-specification/blob/c4b7f4307de79009c97b3a98563e91fee39b7ba3/work_in_progress/opencensus/HTTP.md#status>
// | HTTP code               | Span status code      |
// |-------------------------|-----------------------|
// | 100...299               | `Ok`                  |
// | 3xx redirect codes      | `DeadlineExceeded` in case of loop (see above) [1], otherwise `Ok` |
// | 401 Unauthorized ⚠      | `Unauthenticated` ⚠ (Unauthorized actually means unauthenticated according to [RFC 7235][rfc-unauthorized])  |
// | 403 Forbidden           | `PermissionDenied`    |
// | 404 Not Found           | `NotFound`            |
// | 429 Too Many Requests   | `ResourceExhausted`   |
// | Other 4xx code          | `InvalidArgument` [1] |
// | 501 Not Implemented     | `Unimplemented`       |
// | 503 Service Unavailable | `Unavailable`         |
// | 504 Gateway Timeout     | `DeadlineExceeded`    |
// | Other 5xx code          | `InternalError` [1]   |
// | Any status code the client fails to interpret (e.g., 093 or 573) | `UnknownError` |
///
/// Maps the the http status to an Opentelemetry span status following the the specified convention above.
fn get_span_status(request_status: RequestStatusCode) -> Option<&'static str> {
    match request_status.as_u16() {
        100..=399 => Some("OK"),
        400..=599 => Some("ERROR"),
        _ => None,
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

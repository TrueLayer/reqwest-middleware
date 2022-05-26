use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Request, Response, StatusCode as RequestStatusCode};
use reqwest_middleware::{Error, Middleware, Next, Result};
use task_local_extensions::Extensions;
use tracing::{Instrument, Span};

use crate::root_span;

#[derive(Default)]
/// Middleware for tracing requests using the current Opentelemetry Context.
pub struct TracingMiddleware<S> {
    root_span_builder: std::marker::PhantomData<S>,
}

/// ```rust
/// [derive(Default)]
/// pub struct TimeTrace;
/// impl RootSpanBuilder for TimeTrace {
///     fn on_request_start(state: &mut Extensions, req: &Request) -> Span {
///         state.insert(Time::now());
///         root_span!(req, time_elapsed = tracing::field::Empty)
///     }
///
///     fn on_request_end(state: &mut Extensions, span: &Span, outcome: &Result<Response>) {
///         let detla = state.get().unwrap() - Time::now();
///         DefaultRootSpanBuilder::on_request_end(state, span, outcome);
///         match outcome {
///             Ok(_response) => {
///                 span.record("time_elapsed", &detla);
///             }
///             Err(_e) => {}
///         }
///     }
/// }
///
/// let http = ClientBuilder::new(reqwest::Client::new())
///     .with(TimeTrace::default())
///     .build();
/// ```
pub trait RootSpanBuilder {
    fn on_request_start(state: &mut Extensions, req: &Request) -> Span;
    fn on_request_end(state: &mut Extensions, span: &Span, outcome: &Result<Response>) {
        DefaultRootSpanBuilder::on_request_end(state, span, outcome)
    }
}

#[derive(Default)]
pub struct DefaultRootSpanBuilder;

impl RootSpanBuilder for DefaultRootSpanBuilder {
    fn on_request_start(_state: &mut Extensions, req: &Request) -> Span {
        root_span!(req, time_elp = tracing::field::Empty)
    }
    fn on_request_end(_state: &mut Extensions, span: &Span, outcome: &Result<Response>) {
        match outcome {
            Ok(response) => {
                // The request ran successfully
                let span_status = get_span_status(response.status());
                let status_code = response.status().as_u16() as i64;
                let user_agent = get_header_value("user_agent", response.headers());
                if let Some(span_status) = span_status {
                    span.record("otel.status_code", &span_status);
                }
                span.record("http.status_code", &status_code);
                span.record("http.user_agent", &user_agent.as_str());
            }
            Err(e) => {
                // The request didn't run successfully
                let error_message = e.to_string();
                let error_cause_chain = format!("{:?}", e);
                span.record("otel.status_code", &"ERROR");
                span.record("error.message", &error_message.as_str());
                span.record("error.cause_chain", &error_cause_chain.as_str());
                if let Error::Reqwest(e) = e {
                    span.record(
                        "http.status_code",
                        &e.status()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "".to_string())
                            .as_str(),
                    );
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl<RootSpan> Middleware for TracingMiddleware<RootSpan>
where
    RootSpan: RootSpanBuilder + Sync + Send + 'static,
{
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let mut state = Extensions::new();
        let request_span = RootSpan::on_request_start(&mut state, &req);

        let outcome_future = async {
            // Adds tracing headers to the given request to propagate the OpenTelemetry context to downstream revivers of the request.
            // Spans added by downstream consumers will be part of the same trace.
            #[cfg(any(
                feature = "opentelemetry_0_13",
                feature = "opentelemetry_0_14",
                feature = "opentelemetry_0_15",
                feature = "opentelemetry_0_16",
                feature = "opentelemetry_0_17",
            ))]
            let req = crate::otel::inject_opentelemetry_context_into_request(req);

            // Run the request
            let outcome = next.run(req, extensions).await;
            RootSpan::on_request_end(&mut state, &request_span, &outcome);
            outcome
        };

        outcome_future.instrument(request_span.clone()).await
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

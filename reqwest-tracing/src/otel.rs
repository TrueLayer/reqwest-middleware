use reqwest::{
    header::{HeaderName, HeaderValue},
    Request,
};
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

#[cfg(feature = "opentelemetry_0_17")]
use opentelemetry_0_17_pkg as opentelemetry;

#[cfg(feature = "opentelemetry_0_13")]
pub use tracing_opentelemetry_0_12_pkg as tracing_opentelemetry;

#[cfg(feature = "opentelemetry_0_14")]
pub use tracing_opentelemetry_0_13_pkg as tracing_opentelemetry;

#[cfg(feature = "opentelemetry_0_15")]
pub use tracing_opentelemetry_0_14_pkg as tracing_opentelemetry;

#[cfg(feature = "opentelemetry_0_16")]
pub use tracing_opentelemetry_0_16_pkg as tracing_opentelemetry;

#[cfg(feature = "opentelemetry_0_17")]
pub use tracing_opentelemetry_0_17_pkg as tracing_opentelemetry;

use opentelemetry::{global, propagation::Injector};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Injects the given OpenTelemetry Context into a reqwest::Request headers to allow propagation downstream.
pub fn inject_opentelemetry_context_into_request(mut request: Request) -> Request {
    let context = Span::current().context();

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::TracingMiddleware;
    use opentelemetry::sdk::propagation::TraceContextPropagator;
    use reqwest_middleware::ClientBuilder;
    use tracing::{info_span, Instrument, Level};
    #[cfg(any(
        feature = "opentelemetry_0_13",
        feature = "opentelemetry_0_14",
        feature = "opentelemetry_0_15"
    ))]
    use tracing_subscriber_0_2::{filter, layer::SubscriberExt, Registry};
    #[cfg(not(any(
        feature = "opentelemetry_0_13",
        feature = "opentelemetry_0_14",
        feature = "opentelemetry_0_15"
    )))]
    use tracing_subscriber_0_3::{filter, layer::SubscriberExt, Registry};
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn tracing_middleware_propagates_otel_data_even_when_the_span_is_disabled() {
        let tracer = opentelemetry::sdk::export::trace::stdout::new_pipeline()
            .with_writer(std::io::sink())
            .install_simple();
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let subscriber = Registry::default()
            .with(filter::Targets::new().with_target("reqwest_tracing::otel::test", Level::DEBUG))
            .with(telemetry);
        tracing::subscriber::set_global_default(subscriber).unwrap();
        global::set_text_map_propagator(TraceContextPropagator::new());

        // Mock server - sends all request headers back in the response
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(|req: &wiremock::Request| {
                req.headers
                    .iter()
                    .fold(ResponseTemplate::new(200), |resp, (k, v)| {
                        resp.append_header(k.clone(), v.clone())
                    })
            })
            .mount(&server)
            .await;

        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware)
            .build();

        let resp = client
            .get(server.uri())
            .send()
            .instrument(info_span!("some_span"))
            .await
            .unwrap();

        assert!(resp.headers().contains_key("traceparent"));
    }
}

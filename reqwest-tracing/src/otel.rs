use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Request;
use std::str::FromStr;
use tracing::Span;

/// Injects the given OpenTelemetry Context into a reqwest::Request headers to allow propagation downstream.
pub fn inject_opentelemetry_context_into_request(mut request: Request) -> Request {
    #[cfg(feature = "opentelemetry_0_20")]
    opentelemetry_0_20_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_21_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_21")]
    opentelemetry_0_21_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_22_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_22")]
    opentelemetry_0_22_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_23_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_23")]
    opentelemetry_0_23_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_24_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_24")]
    opentelemetry_0_24_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_25_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_25")]
    opentelemetry_0_25_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_26_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_26")]
    opentelemetry_0_26_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_27_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_27")]
    opentelemetry_0_27_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_28_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_28")]
    opentelemetry_0_28_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_29_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_29")]
    opentelemetry_0_29_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_30_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_30")]
    opentelemetry_0_30_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_31_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
        injector.inject_context(&context, &mut RequestCarrier::new(&mut request))
    });

    #[cfg(feature = "opentelemetry_0_31")]
    opentelemetry_0_31_pkg::global::get_text_map_propagator(|injector| {
        use tracing_opentelemetry_0_32_pkg::OpenTelemetrySpanExt;
        let context = Span::current().context();
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

impl RequestCarrier<'_> {
    fn set_inner(&mut self, key: &str, value: String) {
        let header_name = HeaderName::from_str(key).expect("Must be header name");
        let header_value = HeaderValue::from_str(&value).expect("Must be a header value");
        self.request.headers_mut().insert(header_name, header_value);
    }
}

#[cfg(feature = "opentelemetry_0_20")]
impl opentelemetry_0_20_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_21")]
impl opentelemetry_0_21_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_22")]
impl opentelemetry_0_22_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_23")]
impl opentelemetry_0_23_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_24")]
impl opentelemetry_0_24_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_25")]
impl opentelemetry_0_25_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_26")]
impl opentelemetry_0_26_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_27")]
impl opentelemetry_0_27_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_28")]
impl opentelemetry_0_28_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_29")]
impl opentelemetry_0_29_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_30")]
impl opentelemetry_0_30_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(feature = "opentelemetry_0_31")]
impl opentelemetry_0_31_pkg::propagation::Injector for RequestCarrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.set_inner(key, value)
    }
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use crate::{DisableOtelPropagation, TracingMiddleware};
    use reqwest::Response;
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Extension};
    use tracing::{info_span, Instrument, Level};

    use tracing_subscriber::{filter, layer::SubscriberExt, Registry};
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    async fn make_echo_request_in_otel_context(client: ClientWithMiddleware) -> Response {
        static TELEMETRY: OnceLock<()> = OnceLock::new();

        TELEMETRY.get_or_init(|| {
            let subscriber = Registry::default().with(
                filter::Targets::new().with_target("reqwest_tracing::otel::test", Level::DEBUG),
            );

            #[cfg(feature = "opentelemetry_0_20")]
            let subscriber = {
                use opentelemetry_0_20_pkg::trace::TracerProvider;
                use opentelemetry_stdout_0_1::SpanExporterBuilder;

                let exporter = SpanExporterBuilder::default()
                    .with_writer(std::io::sink())
                    .build();

                let provider = opentelemetry_0_20_pkg::sdk::trace::TracerProvider::builder()
                    .with_simple_exporter(exporter)
                    .build();

                let tracer = provider.versioned_tracer("reqwest", None::<&str>, None::<&str>, None);
                let _ = opentelemetry_0_20_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_20_pkg::global::set_text_map_propagator(
                    opentelemetry_0_20_pkg::sdk::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_21_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_21")]
            let subscriber = {
                use opentelemetry_0_21_pkg::trace::TracerProvider;
                use opentelemetry_stdout_0_2::SpanExporterBuilder;

                let exporter = SpanExporterBuilder::default()
                    .with_writer(std::io::sink())
                    .build();

                let provider = opentelemetry_sdk_0_21::trace::TracerProvider::builder()
                    .with_simple_exporter(exporter)
                    .build();

                let tracer = provider.versioned_tracer("reqwest", None::<&str>, None::<&str>, None);
                let _ = opentelemetry_0_21_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_21_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_21::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_22_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_22")]
            let subscriber = {
                use opentelemetry_0_22_pkg::trace::TracerProvider;
                use opentelemetry_stdout_0_3::SpanExporterBuilder;

                let exporter = SpanExporterBuilder::default()
                    .with_writer(std::io::sink())
                    .build();

                let provider = opentelemetry_sdk_0_22::trace::TracerProvider::builder()
                    .with_simple_exporter(exporter)
                    .build();

                let tracer = provider.versioned_tracer("reqwest", None::<&str>, None::<&str>, None);
                let _ = opentelemetry_0_22_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_22_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_22::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_23_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_23")]
            let subscriber = {
                use opentelemetry_0_23_pkg::trace::TracerProvider;
                use opentelemetry_stdout_0_4::SpanExporterBuilder;

                let exporter = SpanExporterBuilder::default()
                    .with_writer(std::io::sink())
                    .build();

                let provider = opentelemetry_sdk_0_23::trace::TracerProvider::builder()
                    .with_simple_exporter(exporter)
                    .build();

                let tracer = provider.tracer_builder("reqwest").build();
                let _ = opentelemetry_0_23_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_23_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_23::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_24_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_24")]
            let subscriber = {
                use opentelemetry_0_24_pkg::trace::TracerProvider;
                use opentelemetry_stdout_0_5::SpanExporterBuilder;

                let exporter = SpanExporterBuilder::default()
                    .with_writer(std::io::sink())
                    .build();

                let provider = opentelemetry_sdk_0_24::trace::TracerProvider::builder()
                    .with_simple_exporter(exporter)
                    .build();

                let tracer = provider.tracer_builder("reqwest").build();
                let _ = opentelemetry_0_24_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_24_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_24::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_25_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_25")]
            let subscriber = {
                use opentelemetry_0_25_pkg::trace::TracerProvider;

                let provider = opentelemetry_sdk_0_25::trace::TracerProvider::builder().build();

                let tracer = provider.tracer_builder("reqwest").build();
                let _ = opentelemetry_0_25_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_25_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_25::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_26_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_26")]
            let subscriber = {
                use opentelemetry_0_26_pkg::trace::TracerProvider;

                let provider = opentelemetry_sdk_0_26::trace::TracerProvider::builder().build();

                let tracer = provider.tracer_builder("reqwest").build();
                let _ = opentelemetry_0_26_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_26_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_26::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_27_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_27")]
            let subscriber = {
                use opentelemetry_0_27_pkg::trace::TracerProvider;

                let provider = opentelemetry_sdk_0_27::trace::TracerProvider::builder().build();

                let tracer = provider.tracer("reqwest");
                let _ = opentelemetry_0_27_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_27_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_27::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_28_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_28")]
            let subscriber = {
                use opentelemetry_0_28_pkg::trace::TracerProvider;

                let provider = opentelemetry_sdk_0_28::trace::SdkTracerProvider::builder().build();

                let tracer = provider.tracer("reqwest");
                let _ = opentelemetry_0_28_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_28_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_28::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_29_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_29")]
            let subscriber = {
                use opentelemetry_0_29_pkg::trace::TracerProvider;

                let provider = opentelemetry_sdk_0_29::trace::SdkTracerProvider::builder().build();

                let tracer = provider.tracer("reqwest");
                let _ = opentelemetry_0_29_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_29_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_29::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_30_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_30")]
            let subscriber = {
                use opentelemetry_0_30_pkg::trace::TracerProvider;

                let provider = opentelemetry_sdk_0_30::trace::SdkTracerProvider::builder().build();

                let tracer = provider.tracer("reqwest");
                let _ = opentelemetry_0_30_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_30_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_30::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_31_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            #[cfg(feature = "opentelemetry_0_31")]
            let subscriber = {
                use opentelemetry_0_31_pkg::trace::TracerProvider;

                let provider = opentelemetry_sdk_0_31::trace::SdkTracerProvider::builder().build();

                let tracer = provider.tracer("reqwest");
                opentelemetry_0_31_pkg::global::set_tracer_provider(provider);
                opentelemetry_0_31_pkg::global::set_text_map_propagator(
                    opentelemetry_sdk_0_31::propagation::TraceContextPropagator::new(),
                );

                let telemetry = tracing_opentelemetry_0_32_pkg::layer().with_tracer(tracer);
                subscriber.with(telemetry)
            };

            tracing::subscriber::set_global_default(subscriber).unwrap();
        });

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

        client
            .get(server.uri())
            .send()
            .instrument(info_span!("some_span"))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn tracing_middleware_propagates_otel_data_even_when_the_span_is_disabled() {
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::default())
            .build();

        let resp = make_echo_request_in_otel_context(client).await;

        assert!(
            resp.headers().contains_key("traceparent"),
            "by default, the tracing middleware will propagate otel contexts"
        );
    }

    #[tokio::test]
    async fn context_no_propagated() {
        let client = ClientBuilder::new(reqwest::Client::new())
            .with_init(Extension(DisableOtelPropagation))
            .with(TracingMiddleware::default())
            .build();

        let resp = make_echo_request_in_otel_context(client).await;

        assert!(
            !resp.headers().contains_key("traceparent"),
            "request should not contain traceparent if context propagation is disabled"
        );
    }
}

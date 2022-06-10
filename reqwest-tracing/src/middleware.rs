use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use task_local_extensions::Extensions;
use tracing::Instrument;

use crate::{DefaultRequestOtelSpanBuilder, RequestOtelSpanBuilder};

/// Middleware for tracing requests using the current Opentelemetry Context.
pub struct TracingMiddleware<S: RequestOtelSpanBuilder> {
    root_span_builder: std::marker::PhantomData<S>,
}

impl<S: RequestOtelSpanBuilder> TracingMiddleware<S> {
    pub fn new() -> TracingMiddleware<S> {
        TracingMiddleware {
            root_span_builder: Default::default(),
        }
    }
}

impl Default for TracingMiddleware<DefaultRequestOtelSpanBuilder> {
    fn default() -> Self {
        TracingMiddleware::new()
    }
}

impl<S: RequestOtelSpanBuilder> Clone for TracingMiddleware<S> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl<RootSpan> Middleware for TracingMiddleware<RootSpan>
where
    RootSpan: RequestOtelSpanBuilder + Sync + Send + 'static,
{
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let request_span = RootSpan::on_request_start(&req, extensions);

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
            RootSpan::on_request_end(&request_span, &outcome, extensions);
            outcome
        };

        outcome_future.instrument(request_span.clone()).await
    }
}

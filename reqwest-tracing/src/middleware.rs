use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use task_local_extensions::Extensions;
use tracing::Instrument;

use crate::{DefaultSpanBackend, RequestOtelSpanBackend};

/// Middleware for tracing requests using the current Opentelemetry Context.
pub struct TracingMiddleware<S: RequestOtelSpanBackend> {
    span_backend: std::marker::PhantomData<S>,
}

impl<S: RequestOtelSpanBackend> TracingMiddleware<S> {
    pub fn new() -> TracingMiddleware<S> {
        TracingMiddleware {
            span_backend: Default::default(),
        }
    }
}

impl Default for TracingMiddleware<DefaultSpanBackend> {
    fn default() -> Self {
        TracingMiddleware::new()
    }
}

impl<S: RequestOtelSpanBackend> Clone for TracingMiddleware<S> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl<RequestOtelSpan> Middleware for TracingMiddleware<RequestOtelSpan>
where
    RequestOtelSpan: RequestOtelSpanBackend + Sync + Send + 'static,
{
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let request_span = RequestOtelSpan::on_request_start(&req, extensions);

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
            match &outcome {
                Ok(res) => RequestOtelSpan::on_request_success(&request_span, res, extensions),
                Err(err) => RequestOtelSpan::on_request_failure(&request_span, err, extensions),
            }
            RequestOtelSpan::on_request_end(&request_span, &outcome, extensions);
            outcome
        };

        outcome_future.instrument(request_span.clone()).await
    }
}

use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use task_local_extensions::Extensions;
use tracing::Instrument;

use crate::{DefaultSpanBackend, ReqwestOtelSpanBackend};

/// Middleware for tracing requests using the current Opentelemetry Context.
pub struct TracingMiddleware<S: ReqwestOtelSpanBackend> {
    span_backend: std::marker::PhantomData<S>,
}

impl<S: ReqwestOtelSpanBackend> TracingMiddleware<S> {
    pub fn new() -> TracingMiddleware<S> {
        TracingMiddleware {
            span_backend: Default::default(),
        }
    }
}

impl<S: ReqwestOtelSpanBackend> Clone for TracingMiddleware<S> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl Default for TracingMiddleware<DefaultSpanBackend> {
    fn default() -> Self {
        TracingMiddleware::new()
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl<ReqwestOtelSpan> Middleware for TracingMiddleware<ReqwestOtelSpan>
where
    ReqwestOtelSpan: ReqwestOtelSpanBackend + Sync + Send + 'static,
{
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let request_span = ReqwestOtelSpan::on_request_start(&req, extensions);

        let outcome_future = async {
            #[cfg(any(
                feature = "opentelemetry_0_13",
                feature = "opentelemetry_0_14",
                feature = "opentelemetry_0_15",
                feature = "opentelemetry_0_16",
                feature = "opentelemetry_0_17",
                feature = "opentelemetry_0_18",
                feature = "opentelemetry_0_19",
                feature = "opentelemetry_0_20",
                feature = "opentelemetry_0_21",
                feature = "opentelemetry_0_22",
            ))]
            let req = if !extensions.contains::<crate::DisableOtelPropagation>() {
                // Adds tracing headers to the given request to propagate the OpenTelemetry context to downstream revivers of the request.
                // Spans added by downstream consumers will be part of the same trace.
                crate::otel::inject_opentelemetry_context_into_request(req)
            } else {
                req
            };

            // Run the request
            let outcome = next.run(req, extensions).await;
            ReqwestOtelSpan::on_request_end(&request_span, &outcome, extensions);
            outcome
        };

        outcome_future.instrument(request_span.clone()).await
    }
}

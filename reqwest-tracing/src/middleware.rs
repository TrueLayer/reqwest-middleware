use std::marker::PhantomData;

use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use tracing::{Instrument, Span};

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
        struct CancelGuard<'s, ReqwestOtelSpan: ReqwestOtelSpanBackend> {
            armed: bool,
            span: &'s Span,
            _phantom: PhantomData<ReqwestOtelSpan>,
        }

        impl<'s, ReqwestOtelSpan: ReqwestOtelSpanBackend> CancelGuard<'s, ReqwestOtelSpan> {
            fn new(span: &'s Span) -> Self {
                Self {
                    armed: true,
                    span,
                    _phantom: PhantomData,
                }
            }
            fn disarm(mut self) {
                self.armed = false;
            }
        }

        impl<'s, ReqwestOtelSpan: ReqwestOtelSpanBackend> Drop for CancelGuard<'s, ReqwestOtelSpan> {
            fn drop(&mut self) {
                if self.armed {
                    ReqwestOtelSpan::on_request_cancelled(self.span);
                }
            }
        }

        let request_span = ReqwestOtelSpan::on_request_start(&req, extensions);

        let outcome_future = async {
            #[cfg(any(
                feature = "opentelemetry_0_20",
                feature = "opentelemetry_0_21",
                feature = "opentelemetry_0_22",
                feature = "opentelemetry_0_23",
            ))]
            let req = if extensions.get::<crate::DisableOtelPropagation>().is_none() {
                // Adds tracing headers to the given request to propagate the OpenTelemetry context to downstream revivers of the request.
                // Spans added by downstream consumers will be part of the same trace.
                crate::otel::inject_opentelemetry_context_into_request(req)
            } else {
                req
            };

            let guard = CancelGuard::<'_, ReqwestOtelSpan>::new(&request_span);

            // Run the request
            let outcome = next.run(req, extensions).await;

            guard.disarm();

            ReqwestOtelSpan::on_request_end(&request_span, &outcome, extensions);
            outcome
        };

        outcome_future.instrument(request_span.clone()).await
    }
}

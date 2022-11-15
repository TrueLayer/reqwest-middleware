use std::{
    future::Future,
    task::{ready, Context, Poll},
};

use pin_project_lite::pin_project;
use reqwest::Response;
use reqwest_middleware::{Error, MiddlewareRequest};
use tower::{Layer, Service};
use tracing::Span;

use crate::{DefaultSpanBackend, ReqwestOtelSpanBackend};

/// Middleware for tracing requests using the current Opentelemetry Context.
pub struct TracingMiddleware<S: ReqwestOtelSpanBackend> {
    span_backend: std::marker::PhantomData<S>,
}

impl<S: ReqwestOtelSpanBackend> Copy for TracingMiddleware<S> {}

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

impl<ReqwestOtelSpan, Svc> Layer<Svc> for TracingMiddleware<ReqwestOtelSpan>
where
    ReqwestOtelSpan: ReqwestOtelSpanBackend + Sync + Send + 'static,
{
    type Service = TracingMiddlewareService<ReqwestOtelSpan, Svc>;

    fn layer(&self, inner: Svc) -> Self::Service {
        TracingMiddlewareService {
            service: inner,
            layer: *self,
        }
    }
}

/// Middleware Service for tracing requests using the current Opentelemetry Context.
pub struct TracingMiddlewareService<S: ReqwestOtelSpanBackend, Svc> {
    layer: TracingMiddleware<S>,
    service: Svc,
}

impl<ReqwestOtelSpan, Svc> Service<MiddlewareRequest>
    for TracingMiddlewareService<ReqwestOtelSpan, Svc>
where
    ReqwestOtelSpan: ReqwestOtelSpanBackend + Sync + Send + 'static,
    Svc: Service<MiddlewareRequest, Response = Response, Error = Error>,
{
    type Response = Response;
    type Error = Error;
    type Future = TracingMiddlewareFuture<ReqwestOtelSpan, Svc::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: MiddlewareRequest) -> Self::Future {
        let MiddlewareRequest {
            request,
            mut extensions,
        } = req;
        let request_span = ReqwestOtelSpan::on_request_start(&request, &mut extensions);
        // Adds tracing headers to the given request to propagate the OpenTelemetry context to downstream revivers of the request.
        // Spans added by downstream consumers will be part of the same trace.
        #[cfg(any(
            feature = "opentelemetry_0_13",
            feature = "opentelemetry_0_14",
            feature = "opentelemetry_0_15",
            feature = "opentelemetry_0_16",
            feature = "opentelemetry_0_17",
            feature = "opentelemetry_0_18",
        ))]
        let request = crate::otel::inject_opentelemetry_context_into_request(request);

        let future = self.service.call(MiddlewareRequest {
            request,
            extensions,
        });

        TracingMiddlewareFuture {
            layer: self.layer,
            span: request_span,
            future,
        }
    }
}

pin_project!(
    pub struct TracingMiddlewareFuture<S: ReqwestOtelSpanBackend, F> {
        layer: TracingMiddleware<S>,
        span: Span,
        #[pin]
        future: F,
    }
);

impl<S: ReqwestOtelSpanBackend, F: Future<Output = Result<Response, Error>>> Future
    for TracingMiddlewareFuture<S, F>
{
    type Output = F::Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let outcome = {
            let _guard = this.span.enter();
            ready!(this.future.poll(cx))
        };
        S::on_request_end(this.span, &outcome);
        Poll::Ready(outcome)
    }
}

#[macro_export]
/// [`reqwest_otel_span!`](crate::reqwest_otel_span) creates a new [`tracing::Span`].
/// It empowers you to add custom properties to the span on top of the default properties provided by the macro
///
/// Default Fields:
/// - http.request.method
/// - url.scheme
/// - server.address
/// - server.port
/// - otel.kind
/// - otel.name
/// - otel.status_code
/// - user_agent.original
/// - http.response.status_code
/// - error.message
/// - error.cause_chain
///
/// Here are some convenient functions to checkout [`default_on_request_success`], [`default_on_request_failure`],
/// and [`default_on_request_end`].
///
/// # Why a macro?
///
/// [`tracing`] requires all the properties attached to a span to be declared upfront, when the span is created.
/// You cannot add new ones afterwards.
/// This makes it extremely fast, but it pushes us to reach for macros when we need some level of composition.
///
/// # Macro syntax
///
/// The first argument is a [span name](https://opentelemetry.io/docs/reference/specification/trace/api/#span).
/// The second argument passed to [`reqwest_otel_span!`](crate::reqwest_otel_span) is a reference to an [`reqwest::Request`].
///
/// ```rust
/// use reqwest_middleware::Result;
/// use http::Extensions;
/// use reqwest::{Request, Response};
/// use reqwest_tracing::{
///     default_on_request_end, reqwest_otel_span, ReqwestOtelSpanBackend
/// };
/// use tracing::Span;
///
/// pub struct CustomReqwestOtelSpanBackend;
///
/// impl ReqwestOtelSpanBackend for CustomReqwestOtelSpanBackend {
///     fn on_request_start(req: &Request, _extension: &mut Extensions) -> Span {
///         reqwest_otel_span!(name = "reqwest-http-request", req)
///     }
///
///     fn on_request_end(span: &Span, outcome: &Result<Response>, _extension: &mut Extensions) {
///         default_on_request_end(span, outcome)
///     }
/// }
/// ```
///
/// If nothing else is specified, the span generated by `reqwest_otel_span!` is identical to the one you'd
/// get by using [`DefaultSpanBackend`]. Note that to avoid leaking sensitive information, the
/// macro doesn't include `url.full`, even though it's required by opentelemetry. You can add the
/// URL attribute explicitly by using [`SpanBackendWithUrl`] instead of `DefaultSpanBackend` or
/// adding the field on your own implementation.
///
/// You can define new fields following the same syntax of [`tracing::info_span!`] for fields:
///
/// ```rust,should_panic
/// use reqwest_tracing::reqwest_otel_span;
/// # let request: &reqwest::Request = todo!();
///
/// // Define a `time_elapsed` field as empty. It might be populated later.
/// // (This example is just to show how to inject data - otel already tracks durations)
/// reqwest_otel_span!(name = "reqwest-http-request", request, time_elapsed = tracing::field::Empty);
///
/// // Define a `name` field with a known value, `AppName`.
/// reqwest_otel_span!(name = "reqwest-http-request", request, name = "AppName");
///
/// // Define an `app_id` field using the variable with the same name as value.
/// let app_id = "XYZ";
/// reqwest_otel_span!(name = "reqwest-http-request", request, app_id);
///
/// // All together
/// reqwest_otel_span!(name = "reqwest-http-request", request, time_elapsed = tracing::field::Empty, name = "AppName", app_id);
/// ```
///
/// You can also choose to customise the level of the generated span:
///
/// ```rust,should_panic
/// use reqwest_tracing::reqwest_otel_span;
/// use tracing::Level;
/// # let request: &reqwest::Request = todo!();
///
/// // Reduce the log level for service endpoints/probes
/// let level = if request.method().as_str() == "POST" {
///     Level::DEBUG
/// } else {
///     Level::INFO
/// };
///
/// // `level =` and name MUST come before the request, in this order
/// reqwest_otel_span!(level = level, name = "reqwest-http-request", request);
/// ```
///
///
/// [`DefaultSpanBackend`]: crate::reqwest_otel_span_builder::DefaultSpanBackend
/// [`SpanBackendWithUrl`]: crate::reqwest_otel_span_builder::DefaultSpanBackend
/// [`default_on_request_success`]: crate::reqwest_otel_span_builder::default_on_request_success
/// [`default_on_request_failure`]: crate::reqwest_otel_span_builder::default_on_request_failure
/// [`default_on_request_end`]: crate::reqwest_otel_span_builder::default_on_request_end
macro_rules! reqwest_otel_span {
    // Vanilla root span at default INFO level, with no additional fields
    (name=$name:expr, $request:ident) => {
        reqwest_otel_span!(name=$name, $request,)
    };
    // Vanilla root span, with no additional fields but custom level
    (level=$level:expr, name=$name:expr, $request:ident) => {
        reqwest_otel_span!(level=$level, name=$name, $request,)
    };
    // Root span with additional fields, default INFO level
    (name=$name:expr, $request:ident, $($field:tt)*) => {
        reqwest_otel_span!(level=$crate::reqwest_otel_span_macro::private::Level::INFO, name=$name, $request, $($field)*)
    };
    // Root span with additional fields and custom level
    (level=$level:expr, name=$name:expr, $request:ident, $($field:tt)*) => {
        {
            let method = $request.method();
            let url = $request.url();
            let scheme = url.scheme();
            let host = url.host_str().unwrap_or("");
            let host_port = url.port_or_known_default().unwrap_or(0) as i64;
            let otel_name = $name.to_string();
            let header_default = &::http::HeaderValue::from_static("");
            let user_agent = format!("{:?}", $request.headers().get("user-agent").unwrap_or(header_default)).replace('"', "");

            // The match here is necessary, because tracing expects the level to be static.
            match $level {
                $crate::reqwest_otel_span_macro::private::Level::TRACE => {
                    $crate::request_span!($crate::reqwest_otel_span_macro::private::Level::TRACE, method, scheme, host, host_port, user_agent, otel_name, $($field)*)
                },
                $crate::reqwest_otel_span_macro::private::Level::DEBUG => {
                    $crate::request_span!($crate::reqwest_otel_span_macro::private::Level::DEBUG, method, scheme, host, host_port, user_agent, otel_name, $($field)*)
                },
                $crate::reqwest_otel_span_macro::private::Level::INFO => {
                    $crate::request_span!($crate::reqwest_otel_span_macro::private::Level::INFO, method, scheme, host, host_port, user_agent, otel_name, $($field)*)
                },
                $crate::reqwest_otel_span_macro::private::Level::WARN => {
                    $crate::request_span!($crate::reqwest_otel_span_macro::private::Level::WARN, method, scheme, host, host_port, user_agent, otel_name, $($field)*)
                },
                $crate::reqwest_otel_span_macro::private::Level::ERROR => {
                    $crate::request_span!($crate::reqwest_otel_span_macro::private::Level::ERROR, method, scheme, host, host_port, user_agent, otel_name, $($field)*)
                },
            }
        }
    }
}

#[doc(hidden)]
pub mod private {
    #[doc(hidden)]
    pub use tracing::{span, Level};

    #[cfg(not(feature = "deprecated_attributes"))]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! request_span {
        ($level:expr, $method:expr, $scheme:expr, $host:expr, $host_port:expr, $user_agent:expr, $otel_name:expr, $($field:tt)*) => {
            $crate::reqwest_otel_span_macro::private::span!(
                $level,
                "HTTP request",
                http.request.method = %$method,
                url.scheme = %$scheme,
                server.address = %$host,
                server.port = %$host_port,
                user_agent.original = %$user_agent,
                otel.kind = "client",
                otel.name = %$otel_name,
                otel.status_code = tracing::field::Empty,
                http.response.status_code = tracing::field::Empty,
                error.message = tracing::field::Empty,
                error.cause_chain = tracing::field::Empty,
                $($field)*
            )
        }
    }

    // With the deprecated attributes flag enabled, we publish both the old and new attributes.
    #[cfg(feature = "deprecated_attributes")]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! request_span {
        ($level:expr, $method:expr, $scheme:expr, $host:expr, $host_port:expr, $user_agent:expr, $otel_name:expr, $($field:tt)*) => {
            $crate::reqwest_otel_span_macro::private::span!(
                $level,
                "HTTP request",
                http.request.method = %$method,
                url.scheme = %$scheme,
                server.address = %$host,
                server.port = %$host_port,
                user_agent.original = %$user_agent,
                otel.kind = "client",
                otel.name = %$otel_name,
                otel.status_code = tracing::field::Empty,
                http.response.status_code = tracing::field::Empty,
                error.message = tracing::field::Empty,
                error.cause_chain = tracing::field::Empty,
                // old attributes
                http.method = %$method,
                http.scheme = %$scheme,
                http.host = %$host,
                net.host.port = %$host_port,
                http.user_agent = tracing::field::Empty,
                http.status_code = tracing::field::Empty,
                $($field)*
            )
        }
    }
}

#[macro_export]
macro_rules! root_span {
    // Vanilla root span at default INFO level, with no additional fields
    ($request:ident) => {
        root_span!($request,)
    };
    // Vanilla root span, with no additional fields but custom level
    (level=$level:expr, $request:ident) => {
        root_span!(level=$level, $request,)
    };
    // Root span with additional fields, default INFO level
    ($request:ident, $($field:tt)*) => {
        root_span!(level=$crate::root_span::private::Level::INFO, $request, $($field)*)
    };
    // Root span with additional fields and custom level
    (level=$level:expr, $request:ident, $($field:tt)*) => {
        {
            let method = $request.method();
            let scheme = $request.url().scheme();
            let host = $request.url().host_str().unwrap_or("");
            let host_port = $request.url().port().unwrap_or(0) as i64;
            let path = $request.url().path();
            let otel_name = format!("{} {}", method, path);

            macro_rules! request_span {
                ($lvl:expr) => {
                    $crate::root_span::private::span!(
                        $lvl,
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
                        $($field)*
                    )
                }
            }

            let span = match $level {
                $crate::root_span::private::Level::TRACE => {
                    request_span!($crate::root_span::private::Level::TRACE)
                },
                $crate::root_span::private::Level::DEBUG => {
                    request_span!($crate::root_span::private::Level::DEBUG)
                },
                $crate::root_span::private::Level::INFO => {
                    request_span!($crate::root_span::private::Level::INFO)
                },
                $crate::root_span::private::Level::WARN => {
                    request_span!($crate::root_span::private::Level::WARN)
                },
                $crate::root_span::private::Level::ERROR => {
                    request_span!($crate::root_span::private::Level::ERROR)
                },
            };
            span
        }
    }
}

#[doc(hidden)]
pub mod private {
    #[doc(hidden)]
    pub use tracing::{span, Level};
}

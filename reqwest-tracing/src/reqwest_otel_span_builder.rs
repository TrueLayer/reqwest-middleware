use std::borrow::Cow;

use matchit::Router;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Request, Response, StatusCode as RequestStatusCode, Url};
use reqwest_middleware::{Error, Result};
use task_local_extensions::Extensions;
use tracing::{warn, Span};

use crate::reqwest_otel_span;

/// The `http.method` field added to the span by [`reqwest_otel_span`]
pub const HTTP_METHOD: &str = "http.method";
/// The `http.scheme` field added to the span by [`reqwest_otel_span`]
pub const HTTP_SCHEME: &str = "http.scheme";
/// The `http.host` field added to the span by [`reqwest_otel_span`]
pub const HTTP_HOST: &str = "http.host";
/// The `http.url` field added to the span by [`reqwest_otel_span`]
pub const HTTP_URL: &str = "http.url";
/// The `host.port` field added to the span by [`reqwest_otel_span`]
pub const NET_HOST_PORT: &str = "net.host.port";
/// The `otel.kind` field added to the span by [`reqwest_otel_span`]
pub const OTEL_KIND: &str = "otel.kind";
/// The `otel.name` field added to the span by [`reqwest_otel_span`]
pub const OTEL_NAME: &str = "otel.name";
/// The `otel.status_code.code` field added to the span by [`reqwest_otel_span`]
pub const OTEL_STATUS_CODE: &str = "http.status_code";
/// The `error.message` field added to the span by [`reqwest_otel_span`]
pub const ERROR_MESSAGE: &str = "error.message";
/// The `error.cause_chain` field added to the span by [`reqwest_otel_span`]
pub const ERROR_CAUSE_CHAIN: &str = "error.cause_chain";
/// The `http.status_code` field added to the span by [`reqwest_otel_span`]
pub const HTTP_STATUS_CODE: &str = "http.status_code";
/// The `http.user_agent` added to the span by [`reqwest_otel_span`]
pub const HTTP_USER_AGENT: &str = "http.user_agent";

/// [`ReqwestOtelSpanBackend`] allows you to customise the span attached by
/// [`TracingMiddleware`] to incoming requests.
///
/// Check out [`reqwest_otel_span`] documentation for examples.
///
/// [`TracingMiddleware`]: crate::middleware::TracingMiddleware.
pub trait ReqwestOtelSpanBackend {
    /// Initalized a new span before the request is executed.
    fn on_request_start(req: &Request, extension: &mut Extensions) -> Span;

    /// Runs after the request call has executed.
    fn on_request_end(span: &Span, outcome: &Result<Response>, extension: &mut Extensions);
}

/// Populates default success/failure fields for a given [`reqwest_otel_span!`] span.
#[inline]
pub fn default_on_request_end(span: &Span, outcome: &Result<Response>) {
    match outcome {
        Ok(res) => default_on_request_success(span, res),
        Err(err) => default_on_request_failure(span, err),
    }
}

/// Populates default success fields for a given [`reqwest_otel_span!`] span.
#[inline]
pub fn default_on_request_success(span: &Span, response: &Response) {
    let span_status = get_span_status(response.status());
    let status_code = response.status().as_u16() as i64;
    let user_agent = get_header_value("user_agent", response.headers());
    if let Some(span_status) = span_status {
        span.record(OTEL_STATUS_CODE, span_status);
    }
    span.record(HTTP_STATUS_CODE, status_code);
    span.record(HTTP_USER_AGENT, user_agent.as_str());
}

/// Populates default failure fields for a given [`reqwest_otel_span!`] span.
#[inline]
pub fn default_on_request_failure(span: &Span, e: &Error) {
    let error_message = e.to_string();
    let error_cause_chain = format!("{:?}", e);
    span.record(OTEL_STATUS_CODE, "ERROR");
    span.record(ERROR_MESSAGE, error_message.as_str());
    span.record(ERROR_CAUSE_CHAIN, error_cause_chain.as_str());
    if let Error::Reqwest(e) = e {
        span.record(
            HTTP_STATUS_CODE,
            e.status()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string())
                .as_str(),
        );
    }
}

/// The default [`ReqwestOtelSpanBackend`] for [`TracingMiddleware`]. Note that it doesn't include
/// the `http.url` field in spans, you can use [`SpanBackendWithUrl`] to add it.
///
/// [`TracingMiddleware`]: crate::middleware::TracingMiddleware
pub struct DefaultSpanBackend;

impl ReqwestOtelSpanBackend for DefaultSpanBackend {
    fn on_request_start(req: &Request, ext: &mut Extensions) -> Span {
        let name = if let Some(name) = ext.get::<OtelName>() {
            Cow::Borrowed(name.0.as_ref())
        } else if let Some(path_names) = ext.get::<OtelPathNames>() {
            path_names
                .find(req.url().path())
                .map(|path| Cow::Owned(format!("{} {}", req.method(), path)))
                .unwrap_or_else(|| {
                    warn!(target: "No OTEL path name found", path = %req.url().path());
                    Cow::Owned(format!("{} UNKNOWN", req.method().as_str()))
                })
        } else {
            Cow::Borrowed(req.method().as_str())
        };

        reqwest_otel_span!(name = name, req)
    }

    fn on_request_end(span: &Span, outcome: &Result<Response>, _: &mut Extensions) {
        default_on_request_end(span, outcome)
    }
}

fn get_header_value(key: &str, headers: &HeaderMap) -> String {
    let header_default = &HeaderValue::from_static("");
    format!("{:?}", headers.get(key).unwrap_or(header_default)).replace('"', "")
}

/// Similar to [`DefaultSpanBackend`] but also adds the `http.url` attribute to request spans.
///
/// [`TracingMiddleware`]: crate::middleware::TracingMiddleware
pub struct SpanBackendWithUrl;

impl ReqwestOtelSpanBackend for SpanBackendWithUrl {
    fn on_request_start(req: &Request, ext: &mut Extensions) -> Span {
        let name = if let Some(name) = ext.get::<OtelName>() {
            Cow::Borrowed(name.0.as_ref())
        } else if let Some(path_names) = ext.get::<OtelPathNames>() {
            path_names
                .find(req.url().path())
                .map(|path| Cow::Owned(format!("{} {}", req.method(), path)))
                .unwrap_or_else(|| {
                    warn!(target: "No OTEL path name found", path = req.url().path());
                    Cow::Owned(format!("{} UNKNOWN", req.method().as_str()))
                })
        } else {
            Cow::Borrowed(req.method().as_str())
        };

        reqwest_otel_span!(name = name, req, http.url = %remove_credentials(req.url()))
    }

    fn on_request_end(span: &Span, outcome: &Result<Response>, _: &mut Extensions) {
        default_on_request_end(span, outcome)
    }
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

/// [`OtelName`] allows customisation of the name of the spans created by
/// [`DefaultSpanBackend`] and [`SpanBackendWithUrl`].
///
/// Usage:
/// ```no_run
/// # use reqwest_middleware::Result;
/// use reqwest_middleware::{ClientBuilder, Extension};
/// use reqwest_tracing::{
///     TracingMiddleware, OtelName
/// };
/// # async fn example() -> Result<()> {
/// let reqwest_client = reqwest::Client::builder().build().unwrap();
/// let client = ClientBuilder::new(reqwest_client)
///    // Inserts the extension before the request is started
///    .with_init(Extension(OtelName("my-client".into())))
///    // Makes use of that extension to specify the otel name
///    .with(TracingMiddleware::default())
///    .build();
///
/// let resp = client.get("https://truelayer.com").send().await.unwrap();
///
/// // Or specify it on the individual request (will take priority)
/// let resp = client.post("https://api.truelayer.com/payment")
///     .with_extension(OtelName("POST /payment".into()))
///    .send()
///    .await
///    .unwrap();
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct OtelName(pub Cow<'static, str>);

/// [`OtelPathNames`] allows including templated paths in the spans created by
/// [`DefaultSpanBackend`] and [`SpanBackendWithUrl`].
///
/// When creating spans this can be used to try to match the path against some
/// known paths. If the path matches value returned is the templated path. This
/// can be used in span names as it will not contain values that would
/// increase the cardinality.
///
/// ```
/// /// # use reqwest_middleware::Result;
/// use reqwest_middleware::{ClientBuilder, Extension};
/// use reqwest_tracing::{
///     TracingMiddleware, OtelPathNames
/// };
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let reqwest_client = reqwest::Client::builder().build()?;
/// let client = ClientBuilder::new(reqwest_client)
///    // Inserts the extension before the request is started
///    .with_init(Extension(OtelPathNames::known_paths(["/payment/:paymentId"])))
///    // Makes use of that extension to specify the otel name
///    .with(TracingMiddleware::default())
///    .build();
///
/// let resp = client.get("https://truelayer.com/payment/id-123").send().await?;
///
/// // Or specify it on the individual request (will take priority)
/// let resp = client.post("https://api.truelayer.com/payment/id-123/authorization-flow")
///     .with_extension(OtelPathNames::known_paths(["/payment/:paymentId/authorization-flow"]))
///    .send()
///    .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct OtelPathNames(matchit::Router<String>);

impl OtelPathNames {
    /// Create a new [`OtelPathNames`] from a set of known paths.
    ///
    /// Paths in this set will be found with `find`.
    ///
    /// Paths can have different parameters:
    /// - Named parameters like `:paymentId` match anything until the next `/` or the end of the path.
    /// - Catch-all parameters start with `*` and match everything after the `/`. They must be at the end of the route.
    /// ```
    /// # use reqwest_tracing::OtelPathNames;
    /// OtelPathNames::known_paths([
    ///     "/",
    ///     "/payment",
    ///     "/payment/:paymentId",
    ///     "/payment/:paymentId/*action",
    /// ]);
    /// ```
    pub fn known_paths<Paths, Path>(paths: Paths) -> Self
    where
        Paths: IntoIterator<Item = Path>,
        Path: Into<String>,
    {
        let router = paths.into_iter().fold(Router::new(), |mut router, path| {
            let path = path.into();
            if let Err(error) = router.insert(path.clone(), path.clone()) {
                warn!(
                    target: "Invalid path cannot be added to known paths",
                    path = %path,
                    error = %error
                );
            }

            router
        });

        Self(router)
    }

    /// Find the templated path from the actual path.
    ///
    /// Returns the templated path if a match is found.
    ///
    /// ```
    /// # use reqwest_tracing::OtelPathNames;
    /// let path_names = OtelPathNames::known_paths(["/payment/:paymentId"]);
    /// let path = path_names.find("/payment/payment-id-123");
    /// assert_eq!(path, Some("/payment/:paymentId"));
    /// ```
    pub fn find(&self, path: &str) -> Option<&str> {
        self.0.at(path).map(|mtch| mtch.value.as_str()).ok()
    }
}

/// Removes the username and/or password parts of the url, if present.
fn remove_credentials(url: &Url) -> Cow<'_, str> {
    if !url.username().is_empty() || url.password().is_some() {
        let mut url = url.clone();
        // Errors settings username/password are set when the URL can't have credentials, so
        // they're just ignored.
        url.set_username("")
            .and_then(|_| url.set_password(None))
            .ok();
        url.to_string().into()
    } else {
        url.as_ref().into()
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

    #[test]
    fn remove_credentials_from_url_without_credentials_is_noop() {
        let url = "http://nocreds.com/".parse().unwrap();
        let clean = remove_credentials(&url);
        assert_eq!(clean, "http://nocreds.com/");
    }

    #[test]
    fn remove_credentials_removes_username_only() {
        let url = "http://user@withuser.com/".parse().unwrap();
        let clean = remove_credentials(&url);
        assert_eq!(clean, "http://withuser.com/");
    }

    #[test]
    fn remove_credentials_removes_password_only() {
        let url = "http://:123@withpwd.com/".parse().unwrap();
        let clean = remove_credentials(&url);
        assert_eq!(clean, "http://withpwd.com/");
    }

    #[test]
    fn remove_credentials_removes_username_and_password() {
        let url = "http://user:123@both.com/".parse().unwrap();
        let clean = remove_credentials(&url);
        assert_eq!(clean, "http://both.com/");
    }
}

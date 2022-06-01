use paste::paste;
use reqwest::{Client, StatusCode};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, Respond, ResponseTemplate,
};

use crate::helpers::SimpleServer;
pub struct RetryResponder(Arc<AtomicU32>, u32, u16);

impl RetryResponder {
    fn new(retries: u32, status_code: u16) -> Self {
        Self(Arc::new(AtomicU32::new(0)), retries, status_code)
    }
}

impl Respond for RetryResponder {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        let mut retries = self.0.load(Ordering::SeqCst);
        retries += 1;
        self.0.store(retries, Ordering::SeqCst);

        if retries + 1 >= self.1 {
            ResponseTemplate::new(200)
        } else {
            ResponseTemplate::new(self.2)
        }
    }
}

macro_rules! assert_retry_succeeds_inner {
    ($x:tt, $name:ident, $status:expr, $retry:tt, $exact:tt, $responder:expr) => {
        #[tokio::test]
        async fn $name() {
            let server = MockServer::start().await;
            let retry_amount: u32 = $retry;
            Mock::given(method("GET"))
                .and(path("/foo"))
                .respond_with($responder)
                .expect($exact)
                .mount(&server)
                .await;

            let reqwest_client = Client::builder().build().unwrap();
            let client = ClientBuilder::new(reqwest_client)
                .with(RetryTransientMiddleware::new_with_policy(
                    ExponentialBackoff {
                        max_n_retries: retry_amount,
                        max_retry_interval: std::time::Duration::from_millis(30),
                        min_retry_interval: std::time::Duration::from_millis(100),
                        backoff_exponent: 2,
                    },
                ))
                .build();

            let resp = client
                .get(&format!("{}/foo", server.uri()))
                .send()
                .await
                .expect("call failed");

            assert_eq!(resp.status(), $status);
        }
    };
}

macro_rules! assert_retry_succeeds {
    ($x:tt, $status:expr) => {
        paste! {
            assert_retry_succeeds_inner!($x, [<assert_retry_succeds_on_ $x>], $status, 3, 2, RetryResponder::new(3 as u32, $x));
        }
    };
}

macro_rules! assert_no_retry {
    ($x:tt, $status:expr) => {
        paste! {
            assert_retry_succeeds_inner!($x, [<assert_no_retry_on_ $x>], $status, 1, 1, ResponseTemplate::new($x));
        }
    };
}

// 2xx.
assert_no_retry!(200, StatusCode::OK);
assert_no_retry!(201, StatusCode::CREATED);
assert_no_retry!(202, StatusCode::ACCEPTED);
assert_no_retry!(203, StatusCode::NON_AUTHORITATIVE_INFORMATION);
assert_no_retry!(204, StatusCode::NO_CONTENT);
assert_no_retry!(205, StatusCode::RESET_CONTENT);
assert_no_retry!(206, StatusCode::PARTIAL_CONTENT);
assert_no_retry!(207, StatusCode::MULTI_STATUS);
assert_no_retry!(226, StatusCode::IM_USED);

// 3xx.
assert_no_retry!(300, StatusCode::MULTIPLE_CHOICES);
assert_no_retry!(301, StatusCode::MOVED_PERMANENTLY);
assert_no_retry!(302, StatusCode::FOUND);
assert_no_retry!(303, StatusCode::SEE_OTHER);
assert_no_retry!(304, StatusCode::NOT_MODIFIED);
assert_no_retry!(307, StatusCode::TEMPORARY_REDIRECT);
assert_no_retry!(308, StatusCode::PERMANENT_REDIRECT);

// 5xx.
assert_retry_succeeds!(500, StatusCode::OK);
assert_retry_succeeds!(501, StatusCode::OK);
assert_retry_succeeds!(502, StatusCode::OK);
assert_retry_succeeds!(503, StatusCode::OK);
assert_retry_succeeds!(504, StatusCode::OK);
assert_retry_succeeds!(505, StatusCode::OK);
assert_retry_succeeds!(506, StatusCode::OK);
assert_retry_succeeds!(507, StatusCode::OK);
assert_retry_succeeds!(508, StatusCode::OK);
assert_retry_succeeds!(510, StatusCode::OK);
assert_retry_succeeds!(511, StatusCode::OK);
// 4xx.
assert_no_retry!(400, StatusCode::BAD_REQUEST);
assert_no_retry!(401, StatusCode::UNAUTHORIZED);
assert_no_retry!(402, StatusCode::PAYMENT_REQUIRED);
assert_no_retry!(403, StatusCode::FORBIDDEN);
assert_no_retry!(404, StatusCode::NOT_FOUND);
assert_no_retry!(405, StatusCode::METHOD_NOT_ALLOWED);
assert_no_retry!(406, StatusCode::NOT_ACCEPTABLE);
assert_no_retry!(407, StatusCode::PROXY_AUTHENTICATION_REQUIRED);
assert_retry_succeeds!(408, StatusCode::OK);
assert_no_retry!(409, StatusCode::CONFLICT);
assert_no_retry!(410, StatusCode::GONE);
assert_no_retry!(411, StatusCode::LENGTH_REQUIRED);
assert_no_retry!(412, StatusCode::PRECONDITION_FAILED);
assert_no_retry!(413, StatusCode::PAYLOAD_TOO_LARGE);
assert_no_retry!(414, StatusCode::URI_TOO_LONG);
assert_no_retry!(415, StatusCode::UNSUPPORTED_MEDIA_TYPE);
assert_no_retry!(416, StatusCode::RANGE_NOT_SATISFIABLE);
assert_no_retry!(417, StatusCode::EXPECTATION_FAILED);
assert_no_retry!(418, StatusCode::IM_A_TEAPOT);
assert_no_retry!(421, StatusCode::MISDIRECTED_REQUEST);
assert_no_retry!(422, StatusCode::UNPROCESSABLE_ENTITY);
assert_no_retry!(423, StatusCode::LOCKED);
assert_no_retry!(424, StatusCode::FAILED_DEPENDENCY);
assert_no_retry!(426, StatusCode::UPGRADE_REQUIRED);
assert_no_retry!(428, StatusCode::PRECONDITION_REQUIRED);
assert_retry_succeeds!(429, StatusCode::OK);
assert_no_retry!(431, StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE);
assert_no_retry!(451, StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS);

// We assert that we cap retries at 10, which means that we will
// get 11 calls to the RetryResponder.
assert_retry_succeeds_inner!(
    500,
    assert_maximum_retries_is_not_exceeded,
    StatusCode::INTERNAL_SERVER_ERROR,
    100,
    11,
    RetryResponder::new(100_u32, 500)
);

pub struct RetryTimeoutResponder(Arc<AtomicU32>, u32, std::time::Duration);

impl RetryTimeoutResponder {
    fn new(retries: u32, initial_timeout: std::time::Duration) -> Self {
        Self(Arc::new(AtomicU32::new(0)), retries, initial_timeout)
    }
}

impl Respond for RetryTimeoutResponder {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        let mut retries = self.0.load(Ordering::SeqCst);
        retries += 1;
        self.0.store(retries, Ordering::SeqCst);

        if retries + 1 >= self.1 {
            ResponseTemplate::new(200)
        } else {
            ResponseTemplate::new(500).set_delay(self.2)
        }
    }
}

#[tokio::test]
async fn assert_retry_on_request_timeout() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/foo"))
        .respond_with(RetryTimeoutResponder::new(
            3,
            std::time::Duration::from_millis(1000),
        ))
        .expect(2)
        .mount(&server)
        .await;

    let reqwest_client = Client::builder().build().unwrap();
    let client = ClientBuilder::new(reqwest_client)
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff {
                max_n_retries: 3,
                max_retry_interval: std::time::Duration::from_millis(100),
                min_retry_interval: std::time::Duration::from_millis(30),
                backoff_exponent: 2,
            },
        ))
        .build();

    let resp = client
        .get(&format!("{}/foo", server.uri()))
        .timeout(std::time::Duration::from_millis(10))
        .send()
        .await
        .expect("call failed");

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn assert_retry_on_incomplete_message() {
    // Following the HTTP/1.1 specification (https://en.wikipedia.org/wiki/HTTP_message_body) a valid response contains:
    // - status line
    // - headers
    // - empty line
    // - optional message body
    //
    // After a few tries we have noticed that:
    // - "message_that_makes_no_sense" triggers a hyper::ParseError because the format is completely wrong
    // - "HTTP/1.1" triggers a hyper::IncompleteMessage because the format is correct until that point but misses mandatory parts
    let incomplete_message = "HTTP/1.1";
    let complete_message = "HTTP/1.1 200 OK\r\n\r\n";

    // create a SimpleServer that returns the correct response after 3 attempts.
    // the first 3 attempts are incomplete http response and internally they result in a [`hyper::Error(IncompleteMessage)`] error.
    let simple_server = SimpleServer::new(
        "127.0.0.1",
        None,
        vec![
            incomplete_message.to_string(),
            incomplete_message.to_string(),
            incomplete_message.to_string(),
            complete_message.to_string(),
        ],
    )
    .await
    .expect("Error when creating a simple server");

    let uri = simple_server.uri();

    tokio::spawn(simple_server.start());

    let reqwest_client = Client::builder().build().unwrap();
    let client = ClientBuilder::new(reqwest_client)
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff {
                max_n_retries: 3,
                max_retry_interval: std::time::Duration::from_millis(100),
                min_retry_interval: std::time::Duration::from_millis(30),
                backoff_exponent: 2,
            },
        ))
        .build();

    let resp = client
        .get(&format!("{}/foo", uri))
        .timeout(std::time::Duration::from_millis(100))
        .send()
        .await
        .expect("call failed");

    assert_eq!(resp.status(), 200);
}

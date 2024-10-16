# reqwest-tracing

Opentracing middleware implementation for
[`reqwest-middleware`](https://crates.io/crates/reqwest-middleware).

[![Crates.io](https://img.shields.io/crates/v/reqwest-tracing.svg)](https://crates.io/crates/reqwest-tracing)
[![Docs.rs](https://docs.rs/reqwest-tracing/badge.svg)](https://docs.rs/reqwest-tracing)
[![CI](https://github.com/TrueLayer/reqwest-middleware/workflows/CI/badge.svg)](https://github.com/TrueLayer/reqwest-middleware/actions)
[![Coverage Status](https://coveralls.io/repos/github/TrueLayer/reqwest-middleware/badge.svg?branch=main&t=UWgSpm)](https://coveralls.io/github/TrueLayer/reqwest-middleware?branch=main)

## Overview

Attach `TracingMiddleware` to your client to automatically trace HTTP requests:

```toml
# Cargo.toml
# ...
[dependencies]
opentelemetry = "0.22"
reqwest = { version = "0.12", features = ["rustls-tls"] }
reqwest-middleware = "0.3"
reqwest-retry = "0.5"
reqwest-tracing = { version = "0.5", features = ["opentelemetry_0_22"] }
tokio = { version = "1.12.0", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-opentelemetry = "0.23"
tracing-subscriber = "0.3"
http = "1"
```

```rust,skip
use reqwest_tracing::{default_on_request_end, reqwest_otel_span, ReqwestOtelSpanBackend, TracingMiddleware};
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, Result};
use std::time::Instant;
use http::Extensions;
use tracing::Span;
use tracing_subscriber::FmtSubscriber;
use tracing::Level;

pub struct TimeTrace;

impl ReqwestOtelSpanBackend for TimeTrace {
    fn on_request_start(req: &Request, extension: &mut Extensions) -> Span {
        extension.insert(Instant::now());
        reqwest_otel_span!(name="example-request", req, time_elapsed = tracing::field::Empty)
    }

    fn on_request_end(span: &Span, outcome: &Result<Response>, extension: &mut Extensions) {
        let time_elapsed = extension.get::<Instant>().unwrap().elapsed().as_millis() as i64;
        default_on_request_end(span, outcome);
        span.record("time_elapsed", &time_elapsed);
    }
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    run().await;
}

async fn run() {
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware::<TimeTrace>::new())
        .build();

    client.get("https://truelayer.com").send().await.unwrap();
}
```

```terminal
$ cargo run
2024-09-10T13:19:52.520194Z TRACE HTTP request{http.request.method=GET url.scheme=https server.address=truelayer.com server.port=443 user_agent.original= otel.kind="client" otel.name=example-request}: hyper_util::client::legacy::pool: checkout waiting for idle connection: ("https", truelayer.com)
2024-09-10T13:19:52.520303Z TRACE HTTP request{http.request.method=GET url.scheme=https server.address=truelayer.com server.port=443 user_agent.original= otel.kind="client" otel.name=example-request}: hyper_util::client::legacy::connect::http: Http::connect; scheme=Some("https"), host=Some("truelayer.com"), port=None
2024-09-10T13:19:52.520686Z DEBUG HTTP request{http.request.method=GET url.scheme=https server.address=truelayer.com server.port=443 user_agent.original= otel.kind="client" otel.name=example-request}:resolve{host=truelayer.com}: hyper_util::client::legacy::connect::dns: resolving host="truelayer.com"
2024-09-10T13:19:52.521847Z DEBUG HTTP request{http.request.method=GET url.scheme=https server.address=truelayer.com server.port=443 user_agent.original= otel.kind="client" otel.name=example-request}: hyper_util::client::legacy::connect::http: connecting to 104.18.24.12:443
2024-09-10T13:19:52.532045Z DEBUG HTTP request{http.request.method=GET url.scheme=https server.address=truelayer.com server.port=443 user_agent.original= otel.kind="client" otel.name=example-request}: hyper_util::client::legacy::connect::http: connected to 104.18.24.12:443
2024-09-10T13:19:52.548050Z TRACE HTTP request{http.request.method=GET url.scheme=https server.address=truelayer.com server.port=443 user_agent.original= otel.kind="client" otel.name=example-request}: hyper_util::client::legacy::client: http1 handshake complete, spawning background dispatcher task
2024-09-10T13:19:52.548651Z TRACE HTTP request{http.request.method=GET url.scheme=https server.address=truelayer.com server.port=443 user_agent.original= otel.kind="client" otel.name=example-request}: hyper_util::client::legacy::pool: checkout dropped for ("https", truelayer.com)
```

See the [`tracing`](https://crates.io/crates/tracing) crate for more information on how to set up a
tracing subscriber to make use of the spans.

## How to install

Add `reqwest-tracing` to your dependencies. Optionally enable opentelemetry integration by enabling
an opentelemetry version feature:

```toml
[dependencies]
# ...
reqwest-tracing = { version = "0.5.0", features = ["opentelemetry_0_22"] }
```

Available opentelemetry features are `opentelemetry_0_22`, `opentelemetry_0_21`, and `opentelemetry_0_20`,

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>

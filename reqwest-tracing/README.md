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
opentelemetry = "0.18"
reqwest = "0.11"
reqwest-middleware = "0.1.1"
reqwest-retry = "0.1.1"
reqwest-tracing = { version = "0.3.1", features = ["opentelemetry_0_18"] }
tokio = { version = "1.25", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-opentelemetry = "0.18"
tracing-subscriber = "0.3"
task-local-extensions = "0.1.3"
```

```rust,skip
use reqwest_tracing::{default_on_request_end, reqwest_otel_span, ReqwestOtelSpanBackend, TracingMiddleware};
use opentelemetry::sdk::export::trace::stdout;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, Result};
use std::time::Instant;
use task_local_extensions::Extensions;
use tracing::Span;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

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
    let tracer = stdout::new_pipeline().install_simple();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber).unwrap();

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
SpanData { span_context: SpanContext { trace_id: ...
```

See the [`tracing`](https://crates.io/crates/tracing) crate for more information on how to set up a
tracing subscriber to make use of the spans.

## How to install

Add `reqwest-tracing` to your dependencies. Optionally enable opentelemetry integration by enabling
an opentelemetry version feature:

```toml
[dependencies]
# ...
reqwest-tracing = { version = "0.3.1", features = ["opentelemetry_0_18"] }
```

Available opentelemetry features are `opentelemetry_0_18`, `opentelemetry_0_17`, `opentelemetry_0_16`, `opentelemetry_0_15`, `opentelemetry_0_14` and
`opentelemetry_0_13`.

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

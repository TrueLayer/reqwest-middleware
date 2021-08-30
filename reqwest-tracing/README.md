# reqwest-tracing

Opentracing middleware implementation for
[`reqwest-middleware`](https://crates.io/crates/reqwest-middleware).

[![Crates.io](https://img.shields.io/crates/v/reqwest-tracing.svg)](https://crates.io/crates/reqwest-tracing)
[![Docs.rs](https://docs.rs/reqwest-tracing/badge.svg)](https://docs.rs/reqwest-tracing)
[![CI](https://github.com/TrueLayer/reqwest-middleware/workflows/CI/badge.svg)](https://github.com/TrueLayer/reqwest-middleware/actions)
[![Coverage Status](https://coveralls.io/repos/github/TrueLayer/reqwest-middleware/badge.svg?branch=main&t=UWgSpm)](https://coveralls.io/github/TrueLayer/reqwest-middleware?branch=main)

## Overview

Attach `TracingMiddleware` to your client to automatically trace HTTP requests:

```rust
use opentelemetry::exporter::trace::stdout;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

#[tokio::main]
async fn main() {
  let (tracer, _) = stdout::new_pipeline().install();
  let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
  let subscriber = Registry::default().with(telemetry);
  tracing::subscriber::set_global_default(subscriber).unwrap();

  run().await;
}

async fun run() {
  let client = ClientBuilder::new(reqwest::Client::new())
    .with(TracingMiddleware)
    .build();`

  client.get("https://truelayer.com").send().await.unwrap();
}
```

See the [`tracing`](https://crates.io/crates/tracing) crate for more information on how to set up a
tracing subscriber to make use of the spans.

## How to install

Add `reqwest-tracing` to your dependencies. Optionally enable opentelemetry integration by enabling
an opentelemetry version feature:

```toml
[dependencies]
# ...
reqwest-tracing = { version = "0.1.0", features = ["opentelemetry_0_16"] }
```

Available opentelemetry features are `opentelemetry_0_16`, `opentelemetry_0_15`, `opentelemetry_0_14` and
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

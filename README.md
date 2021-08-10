# reqwest-middleware

A crate implementing a wrapper around [reqwest](https://crates.io/crates/reqwest)
to allow for client middleware chains.

[![Crates.io](https://img.shields.io/crates/v/reqwest-middleware.svg)](https://crates.io/crates/reqwest-middleware)
[![Docs.rs](https://docs.rs/reqwest-middleware/badge.svg)](https://docs.rs/reqwest-middleware)
[![CI](https://github.com/TrueLayer/reqwest-middleware/workflows/CI/badge.svg)](https://github.com/TrueLayer/reqwest-middleware/actions)
[![Coverage Status](https://coveralls.io/repos/github/TrueLayer/reqwest-middleware/badge.svg?branch=main&t=UWgSpm)](https://coveralls.io/github/TrueLayer/reqwest-middleware?branch=main)

## Overview

The `reqwest-middleware` client exposes the same interface as a plain `reqwest` client, but
`ClientBuilder` exposes functionality to attach middleware:

```rust
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;

#[tokio::main]
async fn main() {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    run(client).await;
}

async fn run(client: ClientWithMiddleware) {
    // free retries!
    client
        .get("https://some-external-service.com")
        .header("foo", "bar")
        .send()
        .await
        .unwrap();
}
```

## How to install

Add `reqwest-middleware` to your dependencies

```toml
[dependencies]
# ...
reqwest-middleware = "0.1.0"
```

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

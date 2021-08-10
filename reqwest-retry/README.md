# reqwest-retry

Retry middleware implementation for
[`reqwest-middleware`](https://crates.io/crates/reqwest-middleware).

[![Crates.io](https://img.shields.io/crates/v/reqwest-retry.svg)](https://crates.io/crates/reqwest-retry)
[![Docs.rs](https://docs.rs/reqwest-retry/badge.svg)](https://docs.rs/reqwest-retry)
[![CI](https://github.com/TrueLayer/reqwest-middleware/workflows/CI/badge.svg)](https://github.com/TrueLayer/reqwest-middleware/actions)
[![Coverage Status](https://coveralls.io/repos/github/TrueLayer/reqwest-middleware/badge.svg?branch=main&t=UWgSpm)](https://coveralls.io/github/TrueLayer/reqwest-middleware?branch=main)

## Overview

Build `RetryTransientMiddleware` from a `RetryPolicy`, then attach it to a
`reqwest_middleware::ClientBuilder`.
[`retry-policies::policies`](https://crates.io/crates/retry-policies) is reexported under
`reqwest_retry::policies` for convenience.

## How to install

Add `reqwest-retry` to your dependencies

```toml
[dependencies]
# ...
reqwest-retry = "0.1.0"
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

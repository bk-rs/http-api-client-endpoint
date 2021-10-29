## Dev

```
cargo clippy --all-features --tests -p http-api-endpoint -p http-api-client -p http-api-reqwest-client -p http-api-reqwest-client-demo -- -D clippy::all
cargo +nightly clippy --all-features --tests -p http-api-endpoint -p http-api-client -p http-api-reqwest-client -p http-api-reqwest-client-demo -- -D clippy::all

cargo clippy --features with-sleep-via-futures-timer --tests -p http-api-isahc-client -p http-api-isahc-client-demo -- -D clippy::all
cargo +nightly clippy --features with-sleep-via-futures-timer --tests -p http-api-isahc-client -p http-api-isahc-client-demo -- -D clippy::all

cargo fmt -- --check

cargo build-all-features
cargo test-all-features -- --nocapture
```

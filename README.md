## Dev

```
cargo clippy --all-features --tests --examples -p http-api-client-endpoint -p http-api-client -p http-api-reqwest-client -p http-api-reqwest-client-demo -- -D clippy::all
cargo +nightly clippy --all-features --tests --examples -p http-api-client-endpoint -p http-api-client -p http-api-reqwest-client -p http-api-reqwest-client-demo -- -D clippy::all

cargo clippy --features with-sleep-via-async-timer --tests --examples -p http-api-isahc-client -p http-api-isahc-client-demo -- -D clippy::all
cargo +nightly clippy --features with-sleep-via-async-timer --tests --examples -p http-api-isahc-client -p http-api-isahc-client-demo -- -D clippy::all

cargo fmt -- --check

cargo test-all-features -- --nocapture
```

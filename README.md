## Dev

```
cargo clippy --all-features --tests -p http-api-endpoint -p http-api-client -- -D clippy::all
cargo +nightly clippy --all-features --tests -p http-api-endpoint -p http-api-client -- -D clippy::all

cargo clippy --features sleep-via-futures-timer --tests -p http-api-isahc-client -p http-api-endpoint-demo -- -D clippy::all
cargo +nightly clippy --features sleep-via-futures-timer --tests -p http-api-isahc-client -p http-api-endpoint-demo -- -D clippy::all

cargo fmt -- --check

cargo build-all-features
cargo test-all-features -- --nocapture
```

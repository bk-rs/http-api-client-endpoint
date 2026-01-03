## Dev

```
cargo clippy --all-features --tests -- -D clippy::all
cargo +nightly clippy --all-features --tests -- -D clippy::all

cargo fmt -- --check

cargo test-all-features -- --nocapture
```

## Publish order

http-api-client-endpoint

http-api-client

http-api-reqwest-client

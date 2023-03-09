## Dev

```
cargo clippy --all-features --tests -- -D clippy::all
cargo +nightly clippy --all-features --tests -- -D clippy::all

cargo fmt -- --check

cargo test-all-features -- --nocapture
```

## Publish order

icmp-packet

icmp-client

async-ping

async-ping/cli

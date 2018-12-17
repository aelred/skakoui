set -e
cargo test --features strict
cargo clippy --features strict -- -D clippy::all

set -e
cargo test
cargo clippy --features strict -- -D clippy::all

set -e

cargo test

touch src/lib.rs  # this is a trick to force clippy to re-check
cargo clippy -- -D clippy::all

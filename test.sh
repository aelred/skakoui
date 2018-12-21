set -e
cargo --color=always check
cargo --color=always test
cargo --color=always clippy --features strict -- -D clippy::all

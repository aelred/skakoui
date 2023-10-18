image=aelred/skakoui

check:
	cargo check --all-targets

lint: check
	cargo clippy --features strict -- -D clippy::all

test:
	cargo test

test-all:
	cargo test --benches --release

bench:
	# Explicitly specify benchmarks because of a conflict between libtest and criterion
	# https://bheisler.github.io/criterion.rs/book/faq.html#cargo-bench-gives-unrecognized-option-errors-for-valid-command-line-options
	cargo bench --bench searcher --bench perft -- --output-format bencher

build:
	docker build --tag $(image) .

run: build
	docker compose up
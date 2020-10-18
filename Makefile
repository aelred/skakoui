tag=$(shell git rev-parse HEAD)
image=aelred/skakoui
tagged=$(image):$(tag)

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
	docker build --tag $(image) --tag $(tagged) .

run: build
	docker run -t -i -e LICHESS_API_TOKEN $(image)

deploy: deploy-docker deploy-ecs

deploy-docker: build
	docker push $(image)
	docker push $(tagged)

deploy-ecs:
	aws ecs update-service --service skakoui-service --force-new-deployment --region us-east-2

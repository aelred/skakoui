tag=$(shell git rev-parse HEAD)
image=aelred/skakoui:$(tag)

lint:
	cargo clippy --features strict -- -D clippy::all

test:
	cargo test

build:
	docker build --tag $(image) .

run: build
	docker run -t -i -e LICHESS_API_TOKEN $(image)

deploy: build
	docker push $(image)
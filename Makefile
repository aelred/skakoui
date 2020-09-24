tag=$(shell git rev-parse HEAD)
image=aelred/skakoui
tagged=$(image):$(tag)

lint:
	cargo clippy --features strict -- -D clippy::all

test:
	cargo test

build:
	docker build --tag $(image) --tag $(tagged) .

run: build
	docker run -t -i -e LICHESS_API_TOKEN $(image)

deploy: build
	docker push $(image)
	docker push $(tagged)
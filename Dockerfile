FROM rust:1.43 as chess-builder
WORKDIR /usr/src/chess
ENV USER docker
RUN cargo init
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo build --release
COPY ./src src
RUN cargo install --path .

FROM python:3 as python3-venv
RUN apt-get update &&\
    apt-get install -y python3-venv
RUN python3 -m venv /venv
ENV PATH="/venv/bin:$PATH"

FROM python3-venv as lichess-bot-builder
RUN git clone https://github.com/careless25/lichess-bot.git /lcbot
WORKDIR /lcbot
RUN pip install -r requirements.txt

FROM python3-venv as skaki
COPY lcbot-config.yml /lcbot/config.yml
COPY --from=chess-builder /usr/local/cargo/bin/uci /chess/bin/skaki
COPY --from=lichess-bot-builder /venv /venv
COPY --from=lichess-bot-builder /lcbot /lcbot
WORKDIR /lcbot
ENTRYPOINT ["python", "lichess-bot.py"]

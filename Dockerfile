FROM rust:1.47-alpine as skakoui-builder
RUN apk add musl-dev
WORKDIR /usr/src/skakoui
ENV USER docker
RUN cargo init
COPY Cargo.lock .
COPY Cargo.toml .
COPY ./benches benches
RUN cargo build --release
RUN rm src/main.rs
COPY ./src src
RUN cargo install --path .

FROM python:3-alpine as python3-venv
RUN python3 -m venv /venv
ENV PATH="/venv/bin:$PATH"

FROM python3-venv as lichess-bot-builder
RUN apk add git
RUN git clone https://github.com/ShailChoksi/lichess-bot.git /lcbot
WORKDIR /lcbot
# lichess-bot 1.2.0, but it's not tagged
RUN git reset dfa846b376814e5dcd278729e8c6aa935a3de957 --hard
RUN pip install -r requirements.txt

FROM python3-venv as skakoui
COPY --from=lichess-bot-builder /venv /venv
COPY --from=lichess-bot-builder /lcbot /lcbot
COPY lcbot-config.yml /lcbot/configtmp.yml
COPY --from=skakoui-builder /usr/local/cargo/bin/uci /lcbot/engines/skakoui
WORKDIR /lcbot
ENTRYPOINT : ${LICHESS_API_TOKEN?"Need to set LICHESS_API_TOKEN"} &&\
    sed "s/LICHESS_API_TOKEN/$LICHESS_API_TOKEN/" configtmp.yml > config.yml &&\
    python lichess-bot.py

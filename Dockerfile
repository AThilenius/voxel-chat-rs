FROM rust AS base

RUN cargo install --locked trunk

CMD trunk serve --release --port $PORT

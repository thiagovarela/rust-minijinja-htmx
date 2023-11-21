FROM rust:1 AS chef 

RUN cargo install cargo-chef 

WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

ENV SQLX_OFFLINE=true
RUN cargo build -p server --release

FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates

RUN update-ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/server ./
COPY templates ./templates
COPY public ./public

EXPOSE 8000

CMD ["/app/server"]

FROM rust:slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /usr/src/kromer

FROM chef AS planner
WORKDIR /usr/src/kromer
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
WORKDIR /usr/src/kromer
COPY --from=planner /usr/src/kromer/recipe.json recipe.json
RUN apt-get update && apt-get install -y pkg-config openssl libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin kromer-economy

FROM debian:bookworm-slim AS runtime
WORKDIR /kromer
COPY --from=builder /usr/src/kromer/target/release/kromer-economy /usr/local/bin
CMD ["kromer-economy"]

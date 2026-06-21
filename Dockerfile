FROM rust:1.87-slim AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    docker.io \
    docker-compose-v2 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/cookest /usr/local/bin/cookest

ENTRYPOINT ["cookest"]

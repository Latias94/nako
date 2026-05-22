# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.85

FROM rust:${RUST_VERSION}-bookworm AS builder
WORKDIR /workspace

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libsqlite3-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --locked --release -p nako-server

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        ffmpeg \
        libsqlite3-0 \
        tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 nako \
    && useradd --system --uid 10001 --gid nako --home-dir /nonexistent --shell /usr/sbin/nologin nako \
    && mkdir -p /config /data/artwork /cache/remux /media \
    && chown -R nako:nako /config /data /cache

COPY --from=builder /workspace/target/release/nako-server /usr/local/bin/nako-server

USER nako:nako
EXPOSE 3000
VOLUME ["/config", "/data", "/cache"]

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["nako-server", "--config", "/config/nako.toml", "serve"]

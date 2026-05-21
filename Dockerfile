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

RUN cargo build --locked --release -p taru-server

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
    && groupadd --system --gid 10001 taru \
    && useradd --system --uid 10001 --gid taru --home-dir /nonexistent --shell /usr/sbin/nologin taru \
    && mkdir -p /config /data/artwork /cache/remux /media \
    && chown -R taru:taru /config /data /cache

COPY --from=builder /workspace/target/release/taru-server /usr/local/bin/taru-server

USER taru:taru
EXPOSE 3000
VOLUME ["/config", "/data", "/cache"]

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["taru-server", "--config", "/config/taru.toml", "serve"]

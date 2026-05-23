# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.95
ARG CARGO_CHEF_VERSION=0.1.77
ARG DEBIAN_MIRROR=http://deb.debian.org/debian
ARG DEBIAN_SECURITY_MIRROR=http://deb.debian.org/debian-security

FROM rust:${RUST_VERSION}-bookworm AS chef
WORKDIR /workspace
ARG CARGO_CHEF_VERSION
ARG DEBIAN_MIRROR
ARG DEBIAN_SECURITY_MIRROR

RUN set -eux; \
    sed -i \
        -e "s|^URIs: http://deb.debian.org/debian-security$|URIs: ${DEBIAN_SECURITY_MIRROR}|g" \
        -e "s|^URIs: http://deb.debian.org/debian$|URIs: ${DEBIAN_MIRROR}|g" \
        /etc/apt/sources.list.d/debian.sources; \
    rm -f /etc/apt/apt.conf.d/docker-clean; \
    apt_packages="ca-certificates libsqlite3-dev pkg-config"; \
    for attempt in 1 2 3 4 5; do \
        apt-get -o Acquire::Retries=5 update \
        && apt-get -o Acquire::Retries=5 install -y --download-only --no-install-recommends \
            $apt_packages \
        && break; \
        if [ "$attempt" = "5" ]; then exit 1; fi; \
        rm -rf /var/lib/apt/lists/*; \
        sleep "$((attempt * 5))"; \
    done; \
    apt-get install -y --no-download --no-install-recommends $apt_packages; \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*.deb /var/cache/apt/archives/partial/*

RUN cargo install cargo-chef --version "${CARGO_CHEF_VERSION}" --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin nako-server

FROM chef AS builder
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin nako-server
COPY . .
RUN cargo build --locked --release -p nako-server --bin nako-server

FROM debian:bookworm-slim AS runtime
WORKDIR /app
ARG DEBIAN_MIRROR
ARG DEBIAN_SECURITY_MIRROR

RUN set -eux; \
    sed -i \
        -e "s|^URIs: http://deb.debian.org/debian-security$|URIs: ${DEBIAN_SECURITY_MIRROR}|g" \
        -e "s|^URIs: http://deb.debian.org/debian$|URIs: ${DEBIAN_MIRROR}|g" \
        /etc/apt/sources.list.d/debian.sources; \
    rm -f /etc/apt/apt.conf.d/docker-clean; \
    apt_packages="ca-certificates curl ffmpeg libsqlite3-0 tini"; \
    for attempt in 1 2 3 4 5 6 7 8; do \
        apt-get -o Acquire::Retries=5 update \
        && apt-get -o Acquire::Retries=5 install -y --download-only --no-install-recommends \
            $apt_packages \
        && break; \
        if [ "$attempt" = "8" ]; then exit 1; fi; \
        rm -rf /var/lib/apt/lists/*; \
        sleep "$((attempt * 5))"; \
    done; \
    apt-get install -y --no-download --no-install-recommends $apt_packages; \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*.deb /var/cache/apt/archives/partial/* \
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

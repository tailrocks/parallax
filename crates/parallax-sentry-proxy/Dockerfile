# syntax=docker/dockerfile:1.7
# parallax-sentry-proxy — Selene Sentry fan-out ingress (§11 scaffold).
#
# Build context: parallax repo root (same pattern as selene-parallax compose).
#   docker build -f crates/parallax-sentry-proxy/Dockerfile -t selene-sentry-proxy .

ARG PARALLAX_GIT_URL=https://github.com/tailrocks/parallax.git
ARG PARALLAX_GIT_REF=HEAD

FROM alpine:3.20 AS src
ARG PARALLAX_GIT_URL
ARG PARALLAX_GIT_REF
RUN apk add --no-cache git ca-certificates
WORKDIR /src
RUN --mount=type=bind,from=parallax-checkout,source=.,target=/checkout \
    if [ -f /checkout/Cargo.toml ]; then \
      cp -a /checkout/. .; \
    else \
      git clone --filter=blob:none --no-checkout "${PARALLAX_GIT_URL}" . && \
      git checkout "${PARALLAX_GIT_REF}"; \
    fi

FROM rust:1.97-bookworm AS rust
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev mold \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY --from=src /src/ ./
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"
RUN cargo build --release --locked -p parallax-sentry-proxy

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=rust /src/target/release/parallax-sentry-proxy /usr/local/bin/parallax-sentry-proxy
USER 65534:65534
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/parallax-sentry-proxy"]
CMD ["--config", "/etc/parallax-sentry-proxy/config.toml"]

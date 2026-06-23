# Maple in LOCAL mode (chDB embedded, NO Tinybird), containerized.
#
# Maple DOES ship prebuilt local-mode bundles for Linux (x86_64 & arm64) via its
# manual installer (maple.dev/cli/install → GitHub Releases): the bundle is the
# `maple` binary + `libchdb.so` (embedded ClickHouse). We install that prebuilt
# bundle rather than building from source. Pin MAPLE_VERSION to a release tag.
#
# `maple start` runs OTLP/HTTP ingest + query API + UI on a SINGLE --port, and it
# binds that port to **127.0.0.1 only** (verified 2026-06-23 — there is no
# --host/bind flag). In the container-only lab topology Rotel lives in another
# container, so we run Maple on an internal loopback port and front it with a
# tiny `socat` forwarder on 0.0.0.0:4318 so `maple:4318` is reachable on the lab
# network. `--offline` serves the bundled UI from loopback.
FROM debian:bookworm-slim
ARG MAPLE_VERSION=""
ENV MAPLE_INSTALL_DIR=/opt/maple/bin \
    MAPLE_BIN_DIR=/usr/local/bin \
    MAPLE_DATA_DIR=/data \
    MAPLE_NO_UPDATE_CHECK=1 \
    MAPLE_INTERNAL_PORT=9000
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates libstdc++6 socat bash \
    && rm -rf /var/lib/apt/lists/* \
    && MAPLE_VERSION="${MAPLE_VERSION}" sh -c 'curl -fsSL https://maple.dev/cli/install | sh' \
    && maple --version
COPY entrypoint.sh /usr/local/bin/maple-entrypoint
RUN chmod +x /usr/local/bin/maple-entrypoint
VOLUME /data
EXPOSE 4318
CMD ["maple-entrypoint"]

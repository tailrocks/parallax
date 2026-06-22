# Maple in LOCAL mode (chDB embedded, NO Tinybird), containerized.
#
# Maple ships its local single-binary via a Homebrew tap (macOS only) and does
# not publish a Linux image or binary for local mode (verified 2026-06-23), so we
# build it from source: Bun compile + libchdb. This is the lab's known risk —
# verify against the current Maple repo at run time; pin MAPLE_REF.
#
# Expected result: a container that runs `maple start`, exposing OTLP/HTTP on
# 4318 and the dashboard, storing chDB data under /data.
FROM oven/bun:1 AS build
ARG MAPLE_REF=main
RUN apt-get update && apt-get install -y --no-install-recommends git ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
RUN git clone --depth 1 --branch "${MAPLE_REF}" https://github.com/Makisuo/maple.git .
RUN bun install --frozen-lockfile || bun install
# Build the local-mode binary. The exact target script is Maple-version-specific;
# consult docs/local-mode.md in the repo. Common forms (uncomment the right one):
# RUN bun run build:local
# RUN bun build apps/cli/src/index.ts --compile --outfile /out/maple

FROM oven/bun:1
WORKDIR /app
ENV MAPLE_DATA_DIR=/data
VOLUME /data
EXPOSE 4318
# COPY --from=build /out/maple /usr/local/bin/maple
# libchdb (embedded ClickHouse) must be present alongside the binary — copy it
# from the build stage once the build step above is finalized.
# CMD ["maple", "start", "--host", "0.0.0.0", "--otlp-port", "4318", "--data", "/data"]
CMD ["bash", "-lc", "echo 'Maple local-mode container: finalize build/CMD per docs/local-mode.md' && sleep infinity"]

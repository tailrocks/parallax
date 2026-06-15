#!/usr/bin/env bash
# Build a single-binary Parallax release: web UI compiled in (embed-ui),
# Zig/cargo-zigbuild binary, tarball + sha256 ready for GitHub/Homebrew.
#
# Usage: scripts/release.sh [target-triple]
#   default target: the host (macOS arm64 first per the V1 build plan).
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo"

command -v mise >/dev/null || {
  echo "mise is required; install tool dependencies through mise" >&2
  exit 1
}

mise install

target="${1:-$(mise exec -- rustc -vV | sed -n 's/^host: //p')}"
zig_target="$target"
case "$target" in
  *-unknown-linux-gnu) zig_target="${target}.2.17" ;;
esac
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"

echo "==> UI build (bun)"
(cd ui && mise exec -- bun install --frozen-lockfile --ignore-scripts && mise exec -- bun run build)
test -f ui/dist/client/_shell.html || {
  echo "ui/dist/client/_shell.html missing after build" >&2
  exit 1
}

echo "==> cargo zigbuild --release --features embed-ui (${zig_target})"
mise exec -- cargo zigbuild --release --locked -p parallax-cli --features embed-ui --target "$zig_target"

bin="target/${target}/release/parallax"
test -x "$bin"

echo "==> package"
dist="target/dist"
name="parallax-v${version}-${target}"
mkdir -p "$dist"
tar -C "$(dirname "$bin")" -czf "${dist}/${name}.tar.gz" parallax
(cd "$dist" && shasum -a 256 "${name}.tar.gz" | tee "${name}.tar.gz.sha256")

echo "==> done: ${dist}/${name}.tar.gz"
echo "    update tailrocks/homebrew-parallax with the url + sha256 above"

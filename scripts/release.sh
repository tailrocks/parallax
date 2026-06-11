#!/usr/bin/env bash
# Build a single-binary Parallax release: web UI compiled in (embed-ui),
# tarball + sha256 ready for a GitHub release / Homebrew formula.
#
# Usage: scripts/release.sh [target-triple]
#   default target: the host (macOS arm64 first per the V1 build plan).
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo"

target="${1:-$(rustc -vV | sed -n 's/^host: //p')}"
version="$(cargo metadata --no-deps --format-version 1 |
  python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')"

echo "==> UI build (pnpm)"
(cd ui && pnpm install --frozen-lockfile && pnpm build)
test -f ui/dist/client/_shell.html || {
  echo "ui/dist/client/_shell.html missing after build" >&2
  exit 1
}

echo "==> cargo build --release --features embed-ui (${target})"
cargo build --release -p parallax-cli --features embed-ui --target "$target"

bin="target/${target}/release/parallax"
test -x "$bin"

echo "==> package"
dist="target/dist"
name="parallax-v${version}-${target}"
mkdir -p "$dist"
tar -C "$(dirname "$bin")" -czf "${dist}/${name}.tar.gz" parallax
(cd "$dist" && shasum -a 256 "${name}.tar.gz" | tee "${name}.tar.gz.sha256")

echo "==> done: ${dist}/${name}.tar.gz"
echo "    update packaging/homebrew/parallax.rb with the url + sha256 above"

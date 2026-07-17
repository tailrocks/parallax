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
base_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
source_sha="$(git rev-parse HEAD)"
short_sha="$(printf '%s' "$source_sha" | cut -c1-7)"
source_epoch="$(git show -s --format=%ct "$source_sha")"
version="${base_version}+${short_sha}"

echo "==> UI build (bun)"
(cd ui && mise exec -- bun install --frozen-lockfile --ignore-scripts && mise exec -- bun run build)
test -f ui/dist/client/_shell.html || {
  echo "ui/dist/client/_shell.html missing after build" >&2
  exit 1
}

case "$target" in
  *-apple-darwin)
    # Native Apple ld + dsymutil + codesign: release-package embeds __DWARF.
    echo "==> cargo build --release --features embed-ui (${target})"
    PARALLAX_VERSION_OVERRIDE="$version" mise exec -- cargo build --release --locked -p parallax-cli --features embed-ui --target "$target"
    ;;
  *-unknown-linux-gnu)
    zig_target="${target}.2.17"
    echo "==> cargo zigbuild --release --features embed-ui,cross-release-vendored (${zig_target})"
    PARALLAX_VERSION_OVERRIDE="$version" mise exec -- cargo zigbuild --release --locked -p parallax-cli --features embed-ui,cross-release-vendored --target "$zig_target"
    ;;
  *)
    echo "unsupported release target: ${target}" >&2
    exit 1
    ;;
esac

bin="target/${target}/release/parallax"
test -x "$bin"

echo "==> deterministic release rehearsal"
dist="target/dist"
mkdir -p "$dist"
mise exec -- cargo xtask release-rehearse \
  --binary "$bin" \
  --target "$target" \
  --version "$version" \
  --channel rehearsal \
  --source-epoch "$source_epoch" \
  --output-dir "$dist"

archive="${dist}/parallax-${version}-${target}.tar.gz"
echo "==> done: ${archive}"
echo "    update tailrocks/homebrew-parallax with the url + sha256 above"

#!/usr/bin/env bash
# Generate the current supported SigNoz Compose deployment with Foundry.
# Generated output is disposable and stays under ./vendor.
set -euo pipefail
cd "$(dirname "$0")"

FOUNDRY_REF="${FOUNDRY_REF:-v0.2.17}"
FOUNDRY_SHA="273dec4a6f6bb8a70b4db9dc975b958d0e2a2944"
CASTING="signoz/casting.yaml"
POURS_DIR="${SIGNOZ_POURS_DIR:-vendor/signoz-pours}"

if [ ! -f "$CASTING" ]; then
  echo "ERROR: missing $CASTING" >&2
  exit 1
fi

if command -v foundryctl >/dev/null 2>&1; then
  FOUNDRYCTL="$(command -v foundryctl)"
else
  OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
  ARCH="$(uname -m | sed 's/x86_64/amd64/; s/aarch64/arm64/')"
  case "$OS/$ARCH" in
    darwin/amd64|darwin/arm64|linux/amd64|linux/arm64) ;;
    *) echo "ERROR: unsupported Foundry host $OS/$ARCH" >&2; exit 1 ;;
  esac
  TMP_DIR="$(mktemp -d /tmp/signoz-foundry.XXXXXX)"
  ARCHIVE="foundry_${OS}_${ARCH}.tar.gz"
  FOUNDRY_VERSION="${FOUNDRY_REF#v}"
  curl -fsSL "https://github.com/SigNoz/foundry/releases/download/${FOUNDRY_REF}/${ARCHIVE}" -o "$TMP_DIR/$ARCHIVE"
  curl -fsSL "https://github.com/SigNoz/foundry/releases/download/${FOUNDRY_REF}/foundry_${FOUNDRY_VERSION}_checksums.txt" -o "$TMP_DIR/checksums.txt"
  (cd "$TMP_DIR" && grep "${ARCHIVE}$" checksums.txt | shasum -a 256 -c -)
  tar -xzf "$TMP_DIR/$ARCHIVE" -C "$TMP_DIR"
  FOUNDRYCTL="$TMP_DIR/foundry_${OS}_${ARCH}/bin/foundryctl"
fi

VERSION="$($FOUNDRYCTL version 2>/dev/null || true)"
case "$VERSION" in
  *"${FOUNDRY_REF#v}"*) ;;
  *) echo "ERROR: expected Foundry ${FOUNDRY_REF} (commit ${FOUNDRY_SHA}), got: $VERSION" >&2; exit 1 ;;
esac

echo "Generating SigNoz Compose with Foundry ${FOUNDRY_REF} (commit ${FOUNDRY_SHA}) ..."
mkdir -p "$POURS_DIR"
"$FOUNDRYCTL" forge --file "$CASTING" --pours "$POURS_DIR" --no-ledger --no-updater
echo "done. Compose: $POURS_DIR/deployment/compose.yaml"

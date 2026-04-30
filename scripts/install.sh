#!/usr/bin/env bash
set -euo pipefail

REPO="${VAULTSHIP_REPO:-cyberxdefend/vaultship}"
INSTALL_DIR="${VAULTSHIP_INSTALL_DIR:-/usr/local/bin}"
VERSION="${1:-latest}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required command: $1" >&2
    exit 1
  }
}

need_cmd curl
need_cmd tar

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux) OS_TARGET="unknown-linux-gnu" ;;
  Darwin) OS_TARGET="apple-darwin" ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT) OS_TARGET="pc-windows-msvc" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_TARGET="x86_64" ;;
  arm64|aarch64) ARCH_TARGET="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"

if [[ "$VERSION" == "latest" ]]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
fi

if [[ -z "${VERSION}" ]]; then
  echo "Could not resolve release version." >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

if [[ "$OS_TARGET" == "pc-windows-msvc" ]]; then
  ASSET="vaultship-${VERSION}-${TARGET}.zip"
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
  need_cmd unzip
  curl -fL "$URL" -o "${TMP_DIR}/${ASSET}"
  unzip -q "${TMP_DIR}/${ASSET}" -d "${TMP_DIR}"
  BIN_SRC="${TMP_DIR}/vaultship.exe"
  BIN_DST="${INSTALL_DIR}/vaultship.exe"
else
  ASSET="vaultship-${VERSION}-${TARGET}.tar.gz"
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
  curl -fL "$URL" -o "${TMP_DIR}/${ASSET}" || {
    if [[ "$TARGET" == "x86_64-unknown-linux-gnu" ]]; then
      # Fallback for static Linux assets if GNU missing
      ASSET="vaultship-${VERSION}-x86_64-unknown-linux-musl.tar.gz"
      URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
      curl -fL "$URL" -o "${TMP_DIR}/${ASSET}"
    else
      exit 1
    fi
  }
  tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}"
  BIN_SRC="${TMP_DIR}/vaultship"
  BIN_DST="${INSTALL_DIR}/vaultship"
fi

mkdir -p "$INSTALL_DIR"
if [[ -w "$INSTALL_DIR" ]]; then
  cp "$BIN_SRC" "$BIN_DST"
else
  sudo cp "$BIN_SRC" "$BIN_DST"
fi
chmod +x "$BIN_DST"

echo "VaultShip installed: $BIN_DST (${VERSION})"

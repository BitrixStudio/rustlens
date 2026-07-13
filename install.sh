#!/usr/bin/env bash
set -euo pipefail

REPO="BitrixStudio/rustlens"
BIN_DIR="${BIN_DIR:-/usr/local/bin}"

usage() {
  cat <<EOF
rustlens installer

Usage:
  install.sh [--bin-dir DIR] [--only rustlens|rustlensmanager] [--version vX.Y.Z]

Examples:
  ./install.sh
  ./install.sh --only rustlens
  BIN_DIR=\$HOME/.local/bin ./install.sh
  ./install.sh --version v0.1.0

EOF
}

ONLY=""
VERSION="latest"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin-dir) BIN_DIR="$2"; shift 2 ;;
    --only) ONLY="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1"; usage; exit 1 ;;
  esac
done

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "Missing dependency: $1"; exit 1; }
}

need curl
need tar
need uname

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)   OS_TAG="unknown-linux-gnu" ;;
  Darwin)  OS_TAG="apple-darwin" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_TAG="x86_64" ;;
  arm64|aarch64) ARCH_TAG="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${ARCH_TAG}-${OS_TAG}"

if [[ "$VERSION" == "latest" ]]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | \
    awk -F'"' '/"tag_name":/ {print $4; exit}')"
  if [[ -z "${VERSION}" ]]; then
    echo "Failed to resolve latest version."
    exit 1
  fi
fi

ASSET="rustlens-${VERSION}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
SUMS_URL="https://github.com/${REPO}/releases/download/${VERSION}/SHA256SUMS.txt"

TMP="$(mktemp -d)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

echo "Downloading ${ASSET}..."
curl -fL "$URL" -o "${TMP}/${ASSET}"

# Optional checksum verification if tools exist and SHA256SUMS is present
if command -v shasum >/dev/null 2>&1; then
  if curl -fsSL "$SUMS_URL" -o "${TMP}/SHA256SUMS.txt"; then
    EXPECTED="$(awk -v f="$ASSET" '$2==f {print $1}' "${TMP}/SHA256SUMS.txt" | head -n1)"
    if [[ -n "$EXPECTED" ]]; then
      ACTUAL="$(shasum -a 256 "${TMP}/${ASSET}" | awk '{print $1}')"
      if [[ "$EXPECTED" != "$ACTUAL" ]]; then
        echo "Checksum mismatch for ${ASSET}"
        exit 1
      fi
    fi
  fi
fi

tar -xzf "${TMP}/${ASSET}" -C "$TMP"
PKG_DIR="${TMP}/rustlens-${VERSION}-${TARGET}"

install_one() {
  local name="$1"
  local src="${PKG_DIR}/${name}"
  local dst="${BIN_DIR}/${name}"

  if [[ ! -f "$src" ]]; then
    echo "Missing binary in archive: $name"
    exit 1
  fi

  mkdir -p "$BIN_DIR"

  if [[ -w "$BIN_DIR" ]]; then
    install -m 0755 "$src" "$dst"
  else
    sudo install -m 0755 "$src" "$dst"
  fi

  echo "Installed: $dst"
}

if [[ -z "$ONLY" ]]; then
  install_one rustlens
  install_one rustlensmanager
else
  install_one "$ONLY"
fi

cat <<EOF

Done.
If '${BIN_DIR}' is not on your PATH, add it:
  export PATH="${BIN_DIR}:\$PATH"

EOF

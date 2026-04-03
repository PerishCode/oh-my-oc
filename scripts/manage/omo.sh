#!/bin/sh
set -eu

NAME=${OH_MY_OC_NAME:-oh-my-oc}
REPO=${OH_MY_OC_REPO:-PerishCode/oh-my-oc}
BASE_URL=${OH_MY_OC_BASE_URL:-https://github.com/$REPO/releases}
INSTALL_ROOT=${OH_MY_OC_INSTALL_ROOT:-"$HOME/.local/share/$NAME"}
LOCAL_BIN_DIR=${OH_MY_OC_LOCAL_BIN_DIR:-"$HOME/.local/bin"}
COMMAND=${1:-install}

if [ "$#" -gt 0 ]; then
  shift
fi

VERSION=${OH_MY_OC_VERSION:-}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      shift
      VERSION=${1:-}
      [ -n "$VERSION" ] || { printf '%s\n' "missing value for --version" >&2; exit 1; }
      ;;
    *)
      printf '%s\n' "unknown argument: $1" >&2
      exit 1
      ;;
  esac
  shift
done

case "$COMMAND" in
  install|upgrade)
    ;;
  uninstall)
    if [ -L "$LOCAL_BIN_DIR/$NAME" ] || [ -e "$LOCAL_BIN_DIR/$NAME" ]; then
      rm -f "$LOCAL_BIN_DIR/$NAME"
    fi
    if [ -n "$VERSION" ] && [ -e "$INSTALL_ROOT/$VERSION" ]; then
      rm -rf "$INSTALL_ROOT/$VERSION"
    elif [ -z "$VERSION" ] && [ -d "$INSTALL_ROOT" ]; then
      rm -rf "$INSTALL_ROOT"
    fi
    exit 0
    ;;
  *)
    printf '%s\n' "unknown command: $COMMAND" >&2
    exit 1
    ;;
esac

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)
    TARGET=aarch64-apple-darwin
    ;;
  Darwin-x86_64)
    TARGET=x86_64-apple-darwin
    ;;
  Linux-x86_64)
    TARGET=x86_64-unknown-linux-gnu
    ;;
  *)
    printf '%s\n' "unsupported host target: $(uname -s)-$(uname -m)" >&2
    exit 1
    ;;
esac

tarball="$NAME-$TARGET.tar.gz"
if [ -n "$VERSION" ]; then
  release_path="$VERSION"
  checksums_url="$BASE_URL/download/$release_path/checksums.txt"
  tarball_url="$BASE_URL/download/$release_path/$tarball"
else
  release_path="latest"
  checksums_url="$BASE_URL/$release_path/download/checksums.txt"
  tarball_url="$BASE_URL/$release_path/download/$tarball"
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM

curl -fsSL "$checksums_url" -o "$tmpdir/checksums.txt"
curl -fsSL "$tarball_url" -o "$tmpdir/$tarball"

if [ -z "$VERSION" ]; then
  VERSION=$(sed -n 's/^VERSION: *//p' "$tmpdir/checksums.txt" | head -n 1)
fi

[ -n "$VERSION" ] || { printf '%s\n' "could not resolve release version" >&2; exit 1; }

INSTALL_DIR="$INSTALL_ROOT/$VERSION"
mkdir -p "$INSTALL_DIR" "$LOCAL_BIN_DIR"

expected=$(awk -v file="$tarball" '$2 == file { print $1 }' "$tmpdir/checksums.txt")
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$tmpdir/$tarball" | awk '{ print $1 }')
elif command -v shasum >/dev/null 2>&1; then
  actual=$(shasum -a 256 "$tmpdir/$tarball" | awk '{ print $1 }')
else
  printf '%s\n' "missing checksum tool: sha256sum or shasum" >&2
  exit 1
fi

if [ -z "$expected" ]; then
  printf '%s\n' "artifact unavailable: $tarball" >&2
  exit 1
fi

if [ "$expected" != "$actual" ]; then
  printf '%s\n' "checksum verification failed" >&2
  exit 1
fi

tar -xzf "$tmpdir/$tarball" -C "$tmpdir"
install -m 755 "$tmpdir/$NAME" "$INSTALL_DIR/$NAME"
ln -sf "$INSTALL_DIR/$NAME" "$LOCAL_BIN_DIR/$NAME"

printf '%s\n' "$LOCAL_BIN_DIR/$NAME"
